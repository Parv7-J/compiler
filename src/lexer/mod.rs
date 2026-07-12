pub mod token;
use token::*;

pub const EOF_CHAR: char = '\0';

pub struct Lexer<'a, I> {
    input: &'a str,
    at: u32,
    chars: I,
    state: State,
    pub newlines: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq)]
enum State {
    Idle,
    Started(Started),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Started {
    Operator(Operator),
    String(u32),
    Ident(u32),
    Number(u32),
}

impl<'a> Lexer<'a, std::str::Chars<'a>> {
    pub fn new(input: &'a str) -> Result<Self, ()> {
        if input.len() > u32::MAX as usize {
            return Err(());
        }
        Ok(Self {
            input,
            at: 0,
            chars: input.chars(),
            state: State::Idle,
            newlines: Vec::new(),
        })
    }

    ///Constructs a Token from a TokenKind
    ///We can optionally provide an start value, for Tokens that are of not known length (for ex:
    ///Number, String)
    ///This fn also consumes the token by calling 'self.bump()', so no need to call consume or bump for the char at 'self.at'
    ///As it may call self.bump(), we need to adhere by its requirements
    fn produce_token(&mut self, kind: TokenKind, start_opt: Option<u32>) -> Token {
        let start = start_opt.unwrap_or_else(|| self.bump());
        match kind {
            TokenKind::Delimiter(_) => Token::new(kind, start, start + 1),
            TokenKind::Punctuation(_) => Token::new(kind, start, start + 1),
            TokenKind::String => Token::new(kind, start, self.bump() + 1),
            TokenKind::Keyword(_) => Token::new(kind, start, self.at),
            TokenKind::Ident => Token::new(kind, start, self.at),
            TokenKind::Number => Token::new(kind, start, self.at),
            TokenKind::Operator(_) => match start_opt {
                Some(_) => Token::new(kind, start, start + 1),
                None => Token::new(kind, start - 1, self.bump() + 1),
            },
            _ => unimplemented!(),
        }
    }

    ///Returns the next char in the input, without consuming it, or 'EOF_CHAR' if no next char
    fn peek(&mut self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    ///Checks if the input stream has ended, and we are not getting an instream 'EOF_CHAR'
    fn is_eof(&mut self) -> bool {
        (self.input.len() - (self.at as usize)) == 0 && (self.peek() == EOF_CHAR)
    }

    ///Returns the next char in the input, consuming it from the input
    ///Should only be called if we are certain that calling 'next' on the input wont return None
    ///Thus, a call to peek + is_eof(if peek returns 'EOF_CHAR') is necessary before this
    fn consume(&mut self) -> char {
        self.chars.next().unwrap()
    }

    ///Returns the current position of the character it is going to consume, and moving 'self.at'
    ///forward
    ///Should only be called if we are certain that calling 'next' on the input wont return None
    ///Thus, a call to peek + is_eof(if peek returns 'EOF_CHAR') is necessary before this
    fn bump(&mut self) -> u32 {
        let at = self.at;
        self.chars.next();
        self.at += 1;
        at
    }

    ///Returns the next token, and returns TokenKind::EOF when cannot yield more tokens
    ///an 'EOF_CHAR' in the stream is skipped, thus only the end of input stream is considered as
    ///the end of file
    ///Errors are handled using the Unknown TokenKind, but are not reported, and thus its the job of
    ///the Parser to handle lexing errors
    pub fn advance_token(&mut self) -> Token {
        loop {
            let ch = self.peek();

            if ch == EOF_CHAR {
                println!("EOF Reached");
                if self.is_eof() {
                    match self.state {
                        State::Started(Started::Operator(op)) => {
                            self.state = State::Idle;
                            return self.produce_token(TokenKind::Operator(op), Some(self.at - 1));
                        }
                        State::Started(Started::String(start)) => {
                            self.state = State::Idle;
                            return self.produce_token(TokenKind::Unknown, Some(start));
                        }
                        State::Idle => {
                            return Token::new(TokenKind::Eof, self.at, self.at);
                        }
                        _ => unimplemented!(),
                    }
                } else {
                    continue;
                }
            }

            if self.state == State::Idle {
                if ch == '\n' {
                    let at = self.bump();
                    self.newlines.push(at);
                    continue;
                } else if ch.is_ascii_whitespace() {
                    self.bump();
                    continue;
                }
            }

            match self.state {
                State::Idle => match ch {
                    '&' | '|' | '+' | '-' | '*' | '/' | '<' | '>' | '=' | '!' => {
                        self.bump();
                        self.state = State::Started(Started::Operator(ch.try_into().unwrap()))
                    }
                    '"' => {
                        self.state = State::Started(Started::String(self.bump()));
                    }
                    'a'..='z' | 'A'..='Z' | '_' => {
                        self.state = State::Started(Started::Ident(self.bump()));
                    }
                    '0'..='9' => {
                        self.state = State::Started(Started::Number(self.bump()));
                    }

                    ';' | ',' | ':' => {
                        return self
                            .produce_token(TokenKind::Punctuation(ch.try_into().unwrap()), None);
                    }
                    '(' | ')' | '{' | '}' | '[' | ']' => {
                        return self
                            .produce_token(TokenKind::Delimiter(ch.try_into().unwrap()), None);
                    }
                    '.' => return self.produce_token(TokenKind::Operator(Operator::Dot), None),

                    _ => {
                        //TODO: batch unknowns
                        let at = self.bump();
                        return Token::new(TokenKind::Unknown, at, at + 1);
                    }
                },
                //now correct
                State::Started(Started::Operator(first)) => match ch {
                    '=' => {
                        self.state = State::Idle;
                        if let Some(op) = first.suffix_equal() {
                            return self.produce_token(TokenKind::Operator(op), None);
                        }
                        if let Ok(op) = CompoundAssignOperator::try_from(first) {
                            return self.produce_token(
                                TokenKind::Operator(Operator::CompoundAssign(op)),
                                None,
                            );
                        }
                        return self.produce_token(TokenKind::Operator(first), Some(self.at - 1));
                    }
                    '&' | '|' => {
                        self.state = State::Idle;
                        if (first == Operator::BitwiseAnd && ch == '&')
                            || (first == Operator::BitwiseOr && ch == '|')
                        {
                            return self.produce_token(
                                TokenKind::Operator(Operator::Logical(
                                    LogicalOperator::try_from(first).unwrap(),
                                )),
                                None,
                            );
                        }
                        return self.produce_token(TokenKind::Operator(first), Some(self.at - 1));
                    }
                    _ => {
                        self.state = State::Idle;
                        return self.produce_token(TokenKind::Operator(first), Some(self.at - 1));
                    }
                },
                //correct
                State::Started(Started::String(start)) if ch == '"' => {
                    self.state = State::Idle;
                    self.produce_token(TokenKind::String, Some(start));
                }
                //correct
                State::Started(Started::Ident(start)) => match ch {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                        self.bump();
                    }
                    _ => {
                        self.state = State::Idle;
                        let ident = &self.input[start as usize..(self.at) as usize];
                        if let Ok(keyword) = Keyword::try_from(ident) {
                            return self.produce_token(TokenKind::Keyword(keyword), Some(start));
                        } else {
                            return self.produce_token(TokenKind::Ident, Some(start));
                        }
                    }
                },
                //correct
                State::Started(Started::Number(start)) if !ch.is_ascii_digit() => {
                    self.state = State::Idle;
                    return self.produce_token(TokenKind::Number, Some(start));
                }
                //TODO: move them in their arms only , will you
                //correct -> started number and a digit, or started string and a valid char
                _ => {
                    self.bump();
                }
            }
        }
    }
}
