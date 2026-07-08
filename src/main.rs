use std::io::BufRead;

fn main() {
    let mut stdin = std::io::stdin().lock();
    let mut input = String::new();
    stdin.read_line(&mut input).unwrap();
    let lexer = Lexer::new(input.chars());
    let ts = TokenStream::new(lexer);
    println!("{ts:?}");
}

struct Lexer<I> {
    input: I,
    state: State,
    scratch: Option<char>,
}

impl<'a> Lexer<std::str::Chars<'a>> {
    pub fn new(iter: impl IntoIterator<Item = char, IntoIter = std::str::Chars<'a>>) -> Self {
        Self {
            input: iter.into_iter(),
            state: State::Idle,
            scratch: None,
        }
    }
}

impl<'a> Iterator for Lexer<std::str::Chars<'a>> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ch = match self.scratch.take().or_else(|| self.input.next()) {
                Some(ch) => ch,
                None => match &self.state {
                    State::Started(Started::Operator(op)) => {
                        let op = *op;
                        self.state = State::Idle;
                        return Some(Token::Operator(op));
                    }
                    State::Started(Started::String(_)) => {
                        panic!("Unclosed quotation for string");
                    }
                    _ => return None,
                },
            };

            if let State::Started(Started::String(_)) = self.state {
            } else {
                if ch == '\n' || ch == ' ' || ch == '\r' || ch == '\t' {
                    continue;
                }
            }

            match self.state {
                State::Idle => match ch {
                    '&' | '|' | '=' | '+' | '-' | '*' | '/' => {
                        self.state = State::Started(Started::Operator(ch.into()))
                    }
                    '"' => {
                        self.state = State::Started(Started::String(String::new()));
                    }
                    _ => {
                        panic!("Invalid token: {ch}");
                    }
                },
                State::Started(Started::Operator(ref mut first)) => {
                    let first = *first;
                    let tok = match ch {
                        '=' => {
                            let op = if let Operator::Equal = first {
                                Operator::Assign
                            } else {
                                let op = match CompoundAssignOperator::try_from(first) {
                                    Ok(op) => op,
                                    Err(op) => {
                                        self.scratch = Some(ch);
                                        self.state = State::Idle;
                                        return Some(Token::Operator(op));
                                    }
                                };
                                Operator::CompoundAssign(op)
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

                            self.scratch = Some(ch);
                            Some(Token::Operator(first))
                        }
                        '+' | '-' | '*' | '/' => {
                            self.state = State::Started(Started::Operator(ch.into()));
                            return Some(Token::Operator(first));
                        }
                        '"' => {
                            self.state = State::Started(Started::String(String::new()));
                            return Some(Token::Operator(first));
                        }
                        _ => {
                            panic!("unknown token");
                        }
                    };
                    self.state = State::Idle;
                    return tok;
                }
                State::Started(Started::String(ref mut s)) => match ch {
                    '"' => {
                        let s = s.clone();
                        self.state = State::Idle;
                        return Some(Token::String(s));
                    }
                    _ => {
                        s.push(ch);
                    }
                },
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum State {
    Idle,
    Started(Started),
}

#[derive(Debug, Clone)]
struct TokenStream(Vec<Token>);

impl TokenStream {
    fn new(lexer: Lexer<std::str::Chars<'_>>) -> Self {
        Self(lexer.collect())
    }
}

#[derive(Debug, Clone)]
enum Token {
    Operator(Operator),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
enum Started {
    Operator(Operator),
    String(String),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Operator {
    BitwiseAnd,
    BitwiseOr,
    Not,
    Equal,
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Logical(LogicalOperator),
    CompoundAssign(CompoundAssignOperator),
}

impl From<char> for Operator {
    fn from(value: char) -> Self {
        match value {
            '&' => Operator::BitwiseAnd,
            '|' => Operator::BitwiseOr,
            '!' => Operator::Not,
            '=' => Operator::Equal,
            '+' => Operator::Add,
            '-' => Operator::Sub,
            '*' => Operator::Mul,
            '/' => Operator::Div,
            _ => panic!("cant convert man"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum LogicalOperator {
    And,
    Or,
}

impl TryFrom<Operator> for LogicalOperator {
    type Error = Operator;

    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        let r = match value {
            Operator::BitwiseOr => Self::Or,
            Operator::BitwiseAnd => Self::And,
            _ => return Err(value),
        };
        Ok(r)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CompoundAssignOperator {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
}

impl TryFrom<Operator> for CompoundAssignOperator {
    type Error = Operator;
    fn try_from(value: Operator) -> Result<Self, Self::Error> {
        let r = match value {
            Operator::BitwiseAnd => Self::And,
            Operator::BitwiseOr => Self::Or,
            Operator::Add => Self::Add,
            Operator::Sub => Self::Sub,
            Operator::Mul => Self::Mul,
            Operator::Div => Self::Div,
            _ => return Err(value),
        };
        Ok(r)
    }
}
