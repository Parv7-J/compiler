//TOOD: a main function needs to be always defined -> entry point
use miette::Report;
use std::sync::Arc;

use crate::parser::ast::Ast;
use crate::parser::ast::Item;
use crate::parser::error::Expected;
use crate::parser::error::Found;
use crate::parser::error::ParseError;
use crate::{Lexer, lexer::token::*};

pub mod ast;
mod common;
mod error;
mod item;
mod pratt;
mod stmt;

pub struct Parser<'a> {
    pub lexer: Lexer<'a>,
    pub token: Option<Token>,
    pub errors: Vec<Report>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a>) -> Self {
        Self {
            lexer,
            token: None,
            errors: Vec::new(),
        }
    }

    pub fn parse(mut self, fname: &str) -> Ast<'a> {
        let mut items = Vec::new();
        //can we get away without to_string()? as we are printing the errors right away
        let source = Arc::new(miette::NamedSource::new(
            fname,
            self.lexer.input().to_string(),
        ));

        while let Some(item) = self.parse_item() {
            items.push(item)
        }

        if !self.errors.is_empty() {
            eprintln!("Found {} syntax errors ->\n", self.errors.len());
        }
        for (no, report) in self.errors.into_iter().enumerate() {
            eprintln!(
                "Syntax Error {}:\n {:?}\n",
                no + 1,
                report.with_source_code(source.clone())
            );
        }

        Ast {
            items,
            input: self.lexer.input(),
        }
    }

    ///removes the token from the lexer
    fn next(&mut self) -> Option<Token> {
        self.token.take().or_else(|| self.lexer.next())
    }

    ///returns the token at the top of the lexer, but doesnt remove it
    fn peek(&mut self) -> Option<TokenKind> {
        let token = self.token.as_ref();
        if let Some(t) = token {
            return Some(t.kind);
        }

        let token = self.lexer.next();
        if let Some(t) = token {
            self.token = Some(t);
            return Some(t.kind);
        }

        None
    }

    ///panics if there is no next token to consume, i.e should be called only if certain that EOF is
    ///not the next token
    fn consume(&mut self) -> Token {
        self.token.take().or_else(|| self.lexer.next()).unwrap()
    }

    ///Returns the next item, or None if EOF is encountered
    fn parse_item(&mut self) -> Option<Item> {
        while self.peek().is_some() {
            let kind = self.peek().unwrap();
            if let TokenKind::Keyword(keyword) = kind {
                match keyword {
                    Keyword::Get => match self.parse_get() {
                        Ok(get) => return Some(Item::Get(get)),
                        Err(err) => self.errors.push(err),
                    },
                    Keyword::Proc => match self.parse_proc() {
                        Ok(proc) => return Some(Item::Procedure(proc)),
                        Err(err) => self.errors.push(err),
                    },
                    Keyword::Methods => match self.parse_methods() {
                        Ok(methods) => return Some(Item::Methods(methods)),
                        Err(err) => self.errors.push(err),
                    },
                    Keyword::Require => match self.parse_require() {
                        Ok(require) => return Some(Item::Require(require)),
                        Err(err) => self.errors.push(err),
                    },
                    Keyword::Aor => match self.parse_aor() {
                        Ok(aor) => return Some(Item::Aor(aor)),
                        Err(err) => self.errors.push(err),
                    },
                    Keyword::Packing => match self.parse_packing() {
                        Ok(packing) => return Some(Item::Packing(packing)),
                        Err(err) => self.errors.push(err),
                    },
                    Keyword::Api => match self.parse_api() {
                        Ok(api) => return Some(Item::Api(api)),
                        Err(err) => self.errors.push(err),
                    },
                    _ => {
                        self.handle_toplevel();
                    }
                }
                continue;
            }
            self.handle_toplevel();
        }
        None
    }

    fn handle_toplevel(&mut self) {
        let bad_token = self.consume();
        self.errors.push(
            ParseError::TopLevelNonItem {
                kind: bad_token.kind,
                span: bad_token.span.into(),
            }
            .into(),
        );

        while let Some(potential) = self.peek() {
            if !matches!(potential, TokenKind::Keyword(_)) {
                self.consume();
            } else {
                return;
            }
        }
    }

    fn expect_and_push(&mut self, kind: TokenKind) {
        if let Err(report) = self.expect(kind) {
            self.errors.push(report)
        }
    }

    fn expect(&mut self, kind: TokenKind) -> miette::Result<Token> {
        match self.peek() {
            Some(token_kind) if token_kind == kind => Ok(self.consume()),
            Some(_) => {
                let token = self.token.or_else(|| self.lexer.clone().next()).unwrap();
                Err(ParseError::Unexpected {
                    expected: Expected::Kind(kind),
                    found: Found::Kind(token.kind),
                    span: token.span.into(),
                }
                .into())
            }
            None => Err(ParseError::Unexpected {
                expected: Expected::Kind(kind),
                found: Found::Eof,
                span: (self.lexer.input().len().saturating_sub(1), 1).into(),
            }
            .into()),
        }
    }
}
