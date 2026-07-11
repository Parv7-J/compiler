pub mod token;
use token::*;

pub const EOF_CHAR: char = '\0';

pub struct Lexer<'a, I> {
    input: &'a str,
    at: u32,
    chars: I,
    state: State,
    scratch: Option<char>,
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
            scratch: None,
        })
    }

    fn peek(&self) -> char {
        self.chars.clone().next().unwrap_or(EOF_CHAR)
    }

    fn advance_token(&mut self) -> Token {
        let ch = match self.scratch.take().or_else(|| self.chars.next()) {
            Some(tup) => tup,
            None => match &self.state {
                State::Started(Started::Operator(op)) => {
                    let op = *op;
                    self.state = State::Idle;
                    return Token::Operator(op);
                }
                State::Started(Started::String(_)) => {
                    panic!("Unclosed quotation for string");
                }
                _ => return None,
            },
        };

        if let State::Started(ref s) = self.state {
            if let Started::Operator(_) = s
                && ch.is_ascii_whitespace()
            {
                continue;
            }
        } else {
            if ch.is_ascii_whitespace() {
                continue;
            }
        }

        match self.state {
            State::Idle => match ch {
                '&' | '|' | '+' | '-' | '*' | '/' | '<' | '>' | '=' | '!' => {
                    self.state = State::Started(Started::Operator(ch.try_into().unwrap()))
                }
                '.' => {
                    return Some(Token::new(TokenKind::Operator(Operator::Dot), pos, pos + 1));
                }
                '"' => {
                    self.state = State::Started(Started::String(pos));
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.state = State::Started(Started::Ident(pos));
                }
                ';' | ',' | ':' => {
                    return Some(Token::new(
                        TokenKind::Punctuation(ch.try_into().unwrap()),
                        pos,
                        pos + 1,
                    ));
                }
                '(' | ')' | '{' | '}' | '[' | ']' => {
                    return Some(Token::new(
                        TokenKind::Delimiter(ch.try_into().unwrap()),
                        pos,
                        pos + 1,
                    ));
                }
                '0'..='9' => {
                    self.state = State::Started(Started::Number(pos));
                }
                _ => {
                    return Some(Token::new(TokenKind::Unknown, pos, pos + 1));
                }
            },
            State::Started(Started::Operator(ref mut first)) => {
                let first = *first;
                let tok = match ch {
                    '=' => {
                        let op = match first {
                            Operator::Assign => Operator::Comparision(ComparisionOperator::Equal),
                            Operator::Not => Operator::Comparision(ComparisionOperator::NotEqual),
                            Operator::Comparision(ComparisionOperator::LessThan) => {
                                Operator::Comparision(ComparisionOperator::LessEqual)
                            }
                            Operator::Comparision(ComparisionOperator::GreaterThan) => {
                                Operator::Comparision(ComparisionOperator::GreaterEqual)
                            }
                            _ => {
                                let op = match CompoundAssignOperator::try_from(first) {
                                    Ok(op) => op,
                                    Err(op) => {
                                        self.scratch = Some((pos, ch));
                                        self.state = State::Idle;
                                        return Some(Token::Operator(op));
                                    }
                                };
                                Operator::CompoundAssign(op)
                            }
                        };

                        Some(Token::Operator(op))
                    }
                    '&' | '|' => {
                        self.state = State::Idle;
                        if (first == Operator::BitwiseAnd && ch == '&')
                            || (first == Operator::BitwiseOr && ch == '|')
                        {
                            return Some(Token::Operator(Operator::Logical(
                                LogicalOperator::try_from(first).unwrap(),
                            )));
                        }

                        self.scratch = Some((pos, ch));
                        Some(Token::Operator(first))
                    }
                    _ => {
                        self.state = State::Idle;
                        self.scratch = Some((pos, ch));
                        return Some(Token::Operator(first));
                    }
                };
                self.state = State::Idle;
                return tok;
            }
            State::Started(Started::String(start)) if ch == '"' => {
                self.state = State::Idle;
                //'"' is 1 byte long
                return Some(Token::String(&self.input[start + 1..pos]));
            }
            State::Started(Started::Ident(start)) => {
                match ch {
                    'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                        continue;
                    }
                    _ => {
                        self.state = State::Idle;
                        self.scratch = Some((pos, ch));
                        let ident = &self.input[start..pos];
                        if let Ok(keyword) = Keyword::try_from(ident) {
                            return Some(Token::Keyword(keyword));
                        } else {
                            //idents are always valid ascii
                            return Some(Token::Ident(&self.input[start..pos]));
                        }
                    }
                }
            }
            State::Started(Started::Number(start)) if !ch.is_ascii_digit() => {
                self.state = State::Idle;
                self.scratch = Some((pos, ch));
                let num = &self.input[start..pos];
                return Some(Token::Number(num));
            }
        }
    }
}
