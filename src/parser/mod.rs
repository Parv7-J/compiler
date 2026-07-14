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
        let mut items = Vec::new();
        while let Some(token) = self.next() {
            if let TokenKind::Keyword(keyword) = token.kind {
                items.push(self.parse_item(keyword));
            }
        }

        // println!("{:#?}", self.idents);

        Ast {
            items,
            intern: self.idents,
        }
    }

    fn parse_item(&mut self, keyword: Keyword) -> Item {
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
                Item::Procedure(proc)
            }
            Keyword::Methods => {
                let methods = self.parse_methods().unwrap();
                Item::Methods(methods)
            }
            Keyword::Require => {
                let require = self.parse_require().unwrap();
                Item::Require(require)
            }
            Keyword::Aor => {
                let aor = self.parse_aor().unwrap();
                Item::Aor(aor)
            }
            Keyword::Packing => {
                let packing = self.parse_packing().unwrap();
                Item::Packing(packing)
            }
            Keyword::Api => {
                let api = self.parse_api().unwrap();
                Item::Api(api)
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

    //a block is defined between { and } -> defines a scope
    fn parse_block(&mut self) -> Result<Block, String> {
        //assignments, if else, while, returns, for, function calling, can be a declaration too!
        todo!()
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let kind = self.peek();
        match kind {
            //TODO: clean this up, and i guess there is some bug here
            Some(TokenKind::Ident) => {
                let ident = self.parse_identty()?;
                if self.peek() == Some(TokenKind::Ident) {
                    self.parse_exprstmt(ident)
                } else {
                    self.parse_declaration(ident)
                }
            }
            Some(TokenKind::Keyword(Keyword::Type(_))) => {
                let identty = self.parse_identty()?;
                self.parse_declaration(identty)
            }
            Some(TokenKind::Keyword(Keyword::If)) => self.parse_conditional(),
            Some(TokenKind::Keyword(Keyword::For)) => self.parse_for(),
            Some(TokenKind::Keyword(Keyword::While)) => self.parse_while(),
            Some(TokenKind::Keyword(Keyword::Seed)) => self.parse_seed(),
            _ => Err(format!("Unexpected kind: {kind:?}")),
        }
    }

    fn parse_exprstmt(&self, ident: IdentTy) -> Result<Stmt, String> {
        Err(String::from("hello"))
    }

    fn parse_expr(&mut self) -> Result<Expr, String> {
        Err(String::from("hello"))
    }

    fn parse_declaration(&mut self, ty: IdentTy) -> Result<Stmt, String> {
        let var = self.parse_ident()?;
        self.expect(TokenKind::Operator(Operator::Assign))?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Declaration { ty, var, value })
    }

    fn parse_seed(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Keyword(Keyword::Seed))?;
        let return_value = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Seed { return_value })
    }

    fn parse_conditional(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Keyword(Keyword::If))?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let block = self.parse_block()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        let ifs = Conditional { condition, block };
        let (elseifs, elses) = self.parse_else()?;
        Ok(Stmt::Conditional {
            ifs,
            elseifs,
            elses,
        })
    }

    fn parse_else(&mut self) -> Result<(Vec<Conditional>, Option<Block>), String> {
        let mut elseifs = Vec::new();
        let mut elses = None;
        #[allow(clippy::while_let_loop)]
        loop {
            match self.peek() {
                Some(TokenKind::Keyword(Keyword::Else)) => {
                    self.next();
                    match self.peek() {
                        Some(TokenKind::Keyword(Keyword::If)) => {
                            self.next();
                            let condition = self.parse_expr()?;
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
                            let block = self.parse_block()?;
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                            elseifs.push(Conditional { condition, block });
                        }
                        _ => {
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
                            let block = self.parse_block()?;
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                            elses = Some(block);
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
        Ok((elseifs, elses))
    }

    fn parse_for(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Keyword(Keyword::For))?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenOpen))?;
        let ty = self.parse_identty()?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
        self.expect(TokenKind::Keyword(Keyword::In))?;
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::Range)) => {
                self.next();
                let range = self.parse_range()?;
                self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                let block = self.parse_block()?;
                self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                Ok(Stmt::For {
                    ident,
                    ty,
                    range: Some(range),
                    block,
                })
            }
            _ => {
                self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                let block = self.parse_block()?;
                self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                Ok(Stmt::For {
                    ident,
                    ty,
                    range: None,
                    block,
                })
            }
        }
    }

    fn parse_while(&mut self) -> Result<Stmt, String> {
        self.expect(TokenKind::Keyword(Keyword::While))?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        let block = self.parse_block()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Stmt::While { condition, block })
    }

    fn parse_range(&mut self) -> Result<(Expr, Expr, Option<Expr>), String> {
        let start = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
        let end = self.parse_expr()?;
        match self.peek() {
            Some(TokenKind::Punctuation(Punctuation::Comma)) => {
                self.next();
                let jump = self.parse_expr()?;
                Ok((start, end, Some(jump)))
            }
            _ => Ok((start, end, None)),
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
