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
        //NOTE: can we get away without cloning? as we are printing the errors right away
        let source = Arc::new(miette::NamedSource::new(
            fname,
            self.lexer.input.to_string(),
        ));

        while let Some(kind) = self.peek() {
            if matches!(kind, TokenKind::Keyword(_)) {
                let item_result = self.parse_item();
                match item_result {
                    Ok(item) => items.push(item),
                    Err(report) => self.errors.push(report),
                    //TODO: add synchronization
                }
                continue;
            }

            //TODO: fix this, by adding synchronization/batching
            let Token { kind, span } = self.next().unwrap();
            self.errors
                .push(miette::Report::from(ParseError::TopLevelNonItem {
                    kind,
                    span: span.into(),
                }));
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
            input: self.lexer.input,
        }
    }

    fn next(&mut self) -> Option<Token> {
        self.token.take().or_else(|| self.lexer.next())
    }

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

    ///panics if there is no next token to consume
    fn consume(&mut self) -> Token {
        self.token.take().or_else(|| self.lexer.next()).unwrap()
    }

    ///panics if there is no next token to peek
    fn parse_item(&mut self) -> miette::Result<Item> {
        let item = match self.peek().unwrap() {
            TokenKind::Keyword(Keyword::Get) => Item::Get(self.parse_get()?),
            TokenKind::Keyword(Keyword::Proc) => Item::Procedure(self.parse_proc()?),
            TokenKind::Keyword(Keyword::Methods) => Item::Methods(self.parse_methods()?),
            TokenKind::Keyword(Keyword::Require) => Item::Require(self.parse_require()?),
            TokenKind::Keyword(Keyword::Aor) => Item::Aor(self.parse_aor()?),
            TokenKind::Keyword(Keyword::Packing) => Item::Packing(self.parse_packing()?),
            TokenKind::Keyword(Keyword::Api) => Item::Api(self.parse_api()?),
            TokenKind::Delimiter(Delimiter::CurlyOpen) => Item::Block(self.parse_block()?),
            kind => {
                return Err(ParseError::TopLevelNonItem {
                    kind,
                    span: self.next().unwrap().span.into(),
                }
                .into());
            }
        };
        Ok(item)
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
                span: (self.lexer.input.len().saturating_sub(1), 1).into(),
            }
            .into()),
        }
    }
}
