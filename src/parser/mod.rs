use std::collections::hash_map::Entry;

use crate::{Lexer, lexer::token::*};
mod ast;
use ast::*;

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

    pub fn parse(mut self) -> Ast<'a> {
        let mut declarations = Vec::new();
        while let Some(token) = self.next() {
            if let TokenKind::Keyword(keyword) = token.kind {
                declarations.push(self.parse_keyword(keyword));
            }
        }

        // println!("{:#?}", self.idents);

        Ast {
            declarations,
            intern: self.idents,
        }
    }

    fn parse_keyword(&mut self, keyword: Keyword) -> Declaration {
        match keyword {
            Keyword::Type(_ty) => todo!(),
            Keyword::If => todo!(),
            Keyword::Else => todo!(),
            Keyword::While => todo!(),
            Keyword::For => todo!(),
            Keyword::Seed => todo!(),

            Keyword::Get => todo!(),
            Keyword::Proc => {
                let proc = self.parse_proc().unwrap();
                Declaration::Procedure(proc)
            }
            Keyword::Methods => {
                let methods = self.parse_methods().unwrap();
                Declaration::Methods(methods)
            }
            Keyword::Require => {
                let require = self.parse_require().unwrap();
                Declaration::Require(require)
            }
            Keyword::Aor => {
                let aor = self.parse_aor().unwrap();
                Declaration::Aor(aor)
            }
            Keyword::Packing => {
                let packing = self.parse_packing().unwrap();
                Declaration::Packing(packing)
            }
            Keyword::Api => {
                let api = self.parse_api().unwrap();
                Declaration::Api(api)
            }
            Keyword::Also | Keyword::Range | Keyword::In | Keyword::From => {
                panic!("Cannot have 'also' as the top token");
            }
        }
    }

    fn expect(&mut self, token: TokenKind) -> Result<Token, String> {
        let next_token = self.next();
        match next_token {
            Some(t) if t.kind == token => Ok(next_token.unwrap()),
            _ => Err(format!("Expected: {token:?} Found: {next_token:?}")),
        }
    }

    // fn parse_get(&mut self) -> Result<Get, String> {
    //     let mut imports = Vec::new();
    //
    //     loop {
    //         let import = self.parse_ident()?;
    //         imports.push(import);
    //         if self.peek() != Some(Token::Punctuation(Punctuation::Comma)) {
    //             break;
    //         }
    //         self.next();
    //     }
    //
    //     self.expect(Token::Keyword(Keyword::From))?;
    //     let s = self.parse_string()?;
    // }

    fn parse_require(&mut self) -> Result<Require, String> {
        let api = self.parse_ident()?;
        self.expect(TokenKind::Keyword(Keyword::For))?;
        let ident = self.parse_ident()?;

        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;

        let mut procs = Vec::new();
        loop {
            if self.peek() == Some(TokenKind::Keyword(Keyword::Proc)) {
                self.next();
                procs.push(self.parse_proc()?);
            } else {
                break;
            }
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Require {
            ident,
            api,
            procedures: procs,
        })
    }

    fn parse_api(&mut self) -> Result<Api, String> {
        let ident = self.parse_ident()?;

        let mut super_api = Vec::new();

        if self.peek() == Some(TokenKind::Keyword(Keyword::Also)) {
            self.next();

            loop {
                let api = self.parse_ident()?;
                super_api.push(api);
                if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                    break;
                }
                self.next();
            }
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;

        let mut procs = Vec::new();

        loop {
            if self.peek() == Some(TokenKind::Keyword(Keyword::Proc)) {
                self.next();
                procs.push(self.parse_proc()?);
            } else {
                break;
            }
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Api {
            ident,
            super_api,
            procedures: procs,
        })
    }

    fn parse_methods(&mut self) -> Result<Methods, String> {
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let mut procs = Vec::new();

        loop {
            if self.peek() == Some(TokenKind::Keyword(Keyword::Proc)) {
                self.next();
                procs.push(self.parse_proc()?);
            } else {
                break;
            }
        }

        if procs.is_empty() {
            return Err(String::from("Atleast one method needed for methods"));
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Methods {
            ident,
            procedures: procs,
        })
    }

    fn parse_proc(&mut self) -> Result<Procedure, String> {
        let ident = self.parse_ident()?;

        self.expect(TokenKind::Delimiter(Delimiter::SquareOpen))?;

        let mut args = Vec::new();
        if self.peek() != Some(TokenKind::Delimiter(Delimiter::SquareClose)) {
            loop {
                let arg = self.parse_field()?;
                args.push(arg);
                if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                    break;
                }
                self.next();
            }
        }

        self.expect(TokenKind::Delimiter(Delimiter::SquareClose))?;

        let mut return_value = None;
        if self.peek() == Some(TokenKind::Punctuation(Punctuation::Colon)) {
            self.next();
            return_value = Some(self.parse_identty()?);
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;

        Ok(Procedure {
            ident,
            args,
            return_value,
        })
    }

    fn parse_packing(&mut self) -> Result<Packing, String> {
        let ident = self.parse_ident()?;

        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;

        let mut fields = Vec::new();

        if self.peek() == Some(TokenKind::Delimiter(Delimiter::CurlyClose)) {
            self.next();
            return Ok(Packing { ident, fields });
        }

        loop {
            let field = self.parse_field()?;
            fields.push(field);
            if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                break;
            }
            self.next();
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;

        Ok(Packing { ident, fields })
    }

    fn parse_aor(&mut self) -> Result<Aor, String> {
        let ident = self.parse_ident()?;

        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;

        let mut variants = Vec::new();

        loop {
            let variant = self.parse_variant()?;
            variants.push(variant);
            if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                break;
            }
            self.next();
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;

        Ok(Aor { ident, variants })
    }

    fn parse_variant(&mut self) -> Result<Variant, String> {
        let ident = self.parse_ident()?;

        if self.peek() != Some(TokenKind::Delimiter(Delimiter::ParenOpen)) {
            return Ok(Variant::Ident(ident));
        }
        self.next();

        let ty = self.parse_identty()?;

        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;

        Ok(Variant::Field(Field { ident, ty }))
    }

    fn parse_field(&mut self) -> Result<Field, String> {
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenOpen))?;
        let ty = self.parse_identty()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
        Ok(Field { ident, ty })
    }

    fn parse_identty(&mut self) -> Result<IdentTy, String> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Keyword(Keyword::Type(t)),
                ..
            }) => Ok(IdentTy::Type(t)),
            Some(Token {
                kind: TokenKind::Ident,
                span,
            }) => Ok(IdentTy::Ident(Ident(self.span_to_id(span)))),
            token => Err(format!("Expected Type/Ident, Found: {token:?}")),
        }
    }

    fn parse_ident(&mut self) -> Result<Ident, String> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Ident,
                span,
            }) => Ok(Ident(self.span_to_id(span))),
            token => Err(format!("Expected Ident, Found: {token:?}")),
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
