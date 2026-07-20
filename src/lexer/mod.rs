pub mod token;
use token::*;

pub const EOF_CHAR: char = '\0';

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    pub input: &'a str,
    pub at: u32,
    pub chars: std::str::Chars<'a>,
    pub state: State,
    pub newlines: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    Idle,
    Started(Started),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Started {
    Operator(Operator),
    String(u32),
    Ident(u32),
    Number(u32, bool),
    Char(u32, bool),
    SingleLineComment,
    MultiLineComment,
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Token> {
        match self.advance_token() {
            t if t.kind == TokenKind::Eof => None,
            t => Some(t),
        }
    }
}

impl<'a> Lexer<'a> {
    #[allow(clippy::result_unit_err)]
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
        //TODO: improve this messy logic
        match kind {
            TokenKind::Delimiter(_) => Token::new(kind, start, start + 1),
            TokenKind::Punctuation(_) => Token::new(kind, start, start + 1),
            TokenKind::String => Token::new(kind, start, self.bump() + 1),
            TokenKind::Keyword(_) => Token::new(kind, start, self.at),
            TokenKind::Ident => Token::new(kind, start, self.at),
            TokenKind::Number => Token::new(kind, start, self.at),
            TokenKind::Char => Token::new(kind, start, self.at),
            TokenKind::Operator(_) => match start_opt {
                Some(_) => Token::new(kind, start, start + 1),
                None => Token::new(kind, self.at - 2, self.at),
            },
            TokenKind::Unknown => Token::new(kind, start, self.at),
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
    fn _consume(&mut self) -> char {
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

            if ch == EOF_CHAR && self.is_eof() {
                match self.state {
                    State::Started(Started::Operator(op)) => {
                        self.state = State::Idle;
                        return self.produce_token(TokenKind::Operator(op), Some(self.at - 1));
                    }
                    State::Started(Started::String(start)) => {
                        self.state = State::Idle;
                        return self.produce_token(TokenKind::Unknown, Some(start));
                    }
                    State::Started(Started::Char(start, _)) => {
                        self.state = State::Idle;
                        return self.produce_token(TokenKind::Unknown, Some(start));
                    }
                    State::Idle
                    | State::Started(Started::SingleLineComment | Started::MultiLineComment) => {
                        self.state = State::Idle;
                        return Token::new(TokenKind::Eof, self.at, self.at);
                    }
                    _ => unimplemented!(),
                }
            }

            //TODO:  no need for this now
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
                    '&' | '|' | '+' | '*' | '<' | '>' | '=' | '!' | '-' => {
                        self.bump();
                        //TODO: remove started operator
                        self.state = State::Started(Started::Operator(ch.try_into().unwrap()))
                    }
                    '/' => {
                        self.bump();
                        if self.peek() == '/' {
                            self.bump();
                            self.state = State::Started(Started::SingleLineComment);
                        } else if self.peek() == '*' {
                            self.bump();
                            self.state = State::Started(Started::MultiLineComment);
                        } else {
                            self.state = State::Started(Started::Operator(ch.try_into().unwrap()))
                        }
                    }
                    '\'' => self.state = State::Started(Started::Char(self.bump(), false)),
                    '"' => {
                        self.state = State::Started(Started::String(self.bump()));
                    }
                    'a'..='z' | 'A'..='Z' | '_' => {
                        self.state = State::Started(Started::Ident(self.bump()));
                    }
                    '0'..='9' => {
                        self.state = State::Started(Started::Number(self.bump(), false));
                    }
                    ';' | ',' | ':' => {
                        return self
                            .produce_token(TokenKind::Punctuation(ch.try_into().unwrap()), None);
                    }
                    '(' | ')' | '{' | '}' | '[' | ']' => {
                        return self
                            .produce_token(TokenKind::Delimiter(ch.try_into().unwrap()), None);
                    }
                    '.' => {
                        let start = self.bump();
                        return self.produce_token(TokenKind::Operator(Operator::Dot), Some(start));
                    }

                    _ => {
                        //TODO: batch unknowns
                        let at = self.bump();
                        return Token::new(TokenKind::Unknown, at, at + 1);
                    }
                },
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
                State::Started(Started::String(start)) => match ch {
                    '"' => {
                        self.state = State::Idle;
                        return self.produce_token(TokenKind::String, Some(start));
                    }
                    _ => {
                        self.bump();
                    }
                },
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
                State::Started(Started::Number(start, ref mut dot_encountered)) => {
                    if ch.is_ascii_digit() {
                        self.bump();
                        continue;
                    }
                    //CHOICE: . trails or not
                    if ch == '.' && !*dot_encountered {
                        *dot_encountered = true;
                        self.bump();
                        continue;
                    }
                    self.state = State::Idle;
                    return self.produce_token(TokenKind::Number, Some(start));
                }
                State::Started(Started::Char(start, ref mut got)) => {
                    if ch == '\'' && *got {
                        self.state = State::Idle;
                        self.bump();
                        return self.produce_token(TokenKind::Char, Some(start));
                    } else if ch == '\'' {
                        self.state = State::Idle;
                        self.bump();
                        return self.produce_token(TokenKind::Unknown, Some(start));
                    }
                    if *got {
                        self.state = State::Idle;
                        return self.produce_token(TokenKind::Unknown, Some(start));
                    }
                    *got = true;
                    self.bump();
                }
                State::Started(Started::SingleLineComment) => {
                    self.bump();
                    if ch != '\n' {
                        continue;
                    }
                    self.state = State::Idle;
                }
                State::Started(Started::MultiLineComment) => {
                    self.bump();

                    if ch != '*' {
                        continue;
                    }

                    if self.peek() == '/' {
                        self.bump();
                        self.state = State::Idle;
                    }
                }
            }
        }
    }
}
