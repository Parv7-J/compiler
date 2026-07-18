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
    pub lexer: Lexer<'a, std::str::Chars<'a>>,
    pub token: Option<Token>,
    pub errors: Vec<miette::Report>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a, std::str::Chars<'a>>) -> Self {
        Self {
            lexer,
            token: None,
            errors: Vec::new(),
        }
    }

    //TODO: filename should be a path
    pub fn parse(mut self, fname: &str) -> Ast<'a> {
        let mut items = Vec::new();
        let source = Arc::new(miette::NamedSource::new(
            fname,
            self.lexer.input.to_string(),
        ));
        loop {
            match self.peek() {
                Some(TokenKind::Keyword(_)) => {
                    let item_result = self.parse_item();
                    match item_result {
                        Ok(item) => items.push(item),
                        Err(report) => self.errors.push(report),
                        //here we got a report, so we need to actually move to a good token and
                        //start parsing again by reaching a synchronization point
                    }
                }
                Some(_) => {
                    let Token { kind, span } = self
                        .next()
                        .expect("already peeked and confirmed presence of a token");
                    self.errors
                        .push(miette::Report::from(ParseError::TopLevelNonItem {
                            kind,
                            span: span.into(),
                        }));
                }
                None => break,
            }
        }

        if !self.errors.is_empty() {
            eprintln!("Found {} errors ->\n", self.errors.len());
        }
        for (no, report) in self.errors.into_iter().enumerate() {
            eprintln!(
                "Error {}:\n {:?}\n",
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

    fn consume(&mut self) -> Token {
        self.token
            .take()
            .or_else(|| self.lexer.next())
            .expect("only called after peek")
    }

    fn fake_consume(&mut self) -> Token {
        self.token
            .or_else(|| self.lexer.clone().next())
            .expect("only called after peek")
    }

    fn parse_item(&mut self) -> miette::Result<Item> {
        let item = match self
            .peek()
            .expect("only called by parse and parse_block, which already checks if there is a token present")
        {
            TokenKind::Keyword(Keyword::Get) => Item::Get(self.parse_get()?),
            TokenKind::Keyword(Keyword::Proc) => Item::Procedure(self.parse_proc()?),
            TokenKind::Keyword(Keyword::Methods) => Item::Methods(self.parse_methods()?),
            TokenKind::Keyword(Keyword::Require) => Item::Require(self.parse_require()?),
            TokenKind::Keyword(Keyword::Aor) => Item::Aor(self.parse_aor()?),
            TokenKind::Keyword(Keyword::Packing) => Item::Packing(self.parse_packing()?),
            TokenKind::Keyword(Keyword::Api) => Item::Api(self.parse_api()?),
            kind => return Err(ParseError::TopLevelNonItem {kind, span: self.next().unwrap().span.into() }.into())
        };
        Ok(item)
    }

    fn expect(&mut self, kind: TokenKind) -> miette::Result<Token> {
        match self.peek() {
            Some(token_kind) if token_kind == kind => Ok(self.consume()),
            Some(_) => {
                let token = self.fake_consume();
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

//     let Span { start, end } = span;
//     let ident = &self.lexer.input[start as usize..end as usize];
//     match self.idents.ids.entry(ident) {
//         Entry::Occupied(occupied_entry) => *occupied_entry.get(),
//         Entry::Vacant(vacant_entry) => {
//             let pos = self.idents.db.len();
//             self.idents.db.push(ident.to_string());
//             vacant_entry.insert(pos);
//             pos
//         }
//     }
// }
//
// fn _id_to_ident(&self, id: usize) -> Option<&String> {
//     self.idents.db.get(id)
// }
