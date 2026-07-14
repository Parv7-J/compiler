#![allow(dead_code)]

use anyhow::Context;
use std::collections::hash_map::Entry;

use crate::parser::ast::Ast;
use crate::parser::ast::Intern;
use crate::parser::ast::Item;
use crate::{Lexer, lexer::token::*};

mod ast;
mod common;
mod item;
mod pratt;
mod stmt;

pub struct Parser<'a> {
    pub lexer: Lexer<'a, std::str::Chars<'a>>,
    pub token: Option<Token>,
    pub idents: Intern<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: Lexer<'a, std::str::Chars<'a>>) -> Self {
        Self {
            lexer,
            token: None,
            idents: Intern::new(),
        }
    }

    pub fn parse(mut self) -> anyhow::Result<Ast<'a>> {
        let mut items = Vec::new();
        while let Some(kind) = self.peek() {
            if let TokenKind::Keyword(_) = kind {
                items.push(self.parse_item().context("Parsing top level items")?);
            }
        }

        Ok(Ast {
            items,
            intern: self.idents,
        })
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

    fn parse_item(&mut self) -> anyhow::Result<Item> {
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
            kind => anyhow::bail!("Expected top level item keyword, Found: {kind:?}"),
        };
        Ok(item)
    }

    fn expect(&mut self, kind: TokenKind) -> anyhow::Result<Token> {
        match self.next() {
            Some(token) if token.kind == kind => Ok(token),
            Some(token) => Err(anyhow::anyhow!(
                "Expected Kind: {kind:?}, Found: {:?}",
                token.kind
            )),
            None => Err(anyhow::anyhow!("Expected Kind: {kind:?}, Reached EOF")),
        }
    }
    fn span_to_id(&mut self, span: Span) -> usize {
        let Span { start, end } = span;
        let ident = &self.lexer.input[start as usize..end as usize];
        match self.idents.ids.entry(ident) {
            Entry::Occupied(occupied_entry) => *occupied_entry.get(),
            Entry::Vacant(vacant_entry) => {
                let pos = self.idents.db.len();
                self.idents.db.push(ident.to_string());
                vacant_entry.insert(pos);
                pos
            }
        }
    }

    fn _id_to_ident(&self, id: usize) -> Option<&String> {
        self.idents.db.get(id)
    }
}
