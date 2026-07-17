use miette::Context;

use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::error::ParseError;

impl Parser<'_> {
    pub fn parse_get(&mut self) -> miette::Result<Get> {
        let get = self.expect(TokenKind::Keyword(Keyword::Get))?;
        let imports = self.parse_identlist()?;
        if imports.is_empty() {
            return Err(ParseError::EmptyImportsList {
                span: get.span.into(),
            }
            .into());
        }
        self.expect(TokenKind::Keyword(Keyword::From))?;
        let module = self.parse_string()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Get { imports, module })
    }

    pub fn parse_require(&mut self) -> miette::Result<Require> {
        self.expect(TokenKind::Keyword(Keyword::Require))?;
        let api = self.parse_ident()?;
        self.expect(TokenKind::Keyword(Keyword::For))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let procs = self.parse_procs()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Require {
            ident,
            api,
            procedures: procs,
        })
    }

    pub fn parse_api(&mut self) -> miette::Result<Api> {
        self.expect(TokenKind::Keyword(Keyword::Api))?;
        let ident = self.parse_ident()?;
        let super_api = if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Also))) {
            let token = self.next().unwrap();
            let list = self.parse_identlist()?;
            if list.is_empty() {
                return Err(ParseError::EmptySubApis {
                    span: token.span.into(),
                }
                .into());
            }
            list
        } else {
            vec![]
        };
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let procedures = self.parse_procs()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Api {
            ident,
            super_api,
            procedures,
        })
    }

    pub fn parse_methods(&mut self) -> miette::Result<Methods> {
        let context = "Parsing Methods";
        self.expect(TokenKind::Keyword(Keyword::Methods))
            .context(context)?;
        let ident = self.parse_ident().context(context)?;
        let curly_open = self
            .expect(TokenKind::Delimiter(Delimiter::CurlyOpen))
            .context(context)?;
        let procs = self.parse_procs()?;
        let curly_close = self
            .expect(TokenKind::Delimiter(Delimiter::CurlyClose))
            .context(context)?;
        if procs.is_empty() {
            let block_span = from_spans(curly_open.span, curly_close.span);
            return Err(ParseError::EmptyMethodsBlock { span: block_span }.into());
        }
        Ok(Methods {
            ident,
            procedures: procs,
        })
    }

    pub fn parse_procs(&mut self) -> miette::Result<Vec<Procedure>> {
        let mut procs = Vec::new();
        loop {
            if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Proc))) {
                procs.push(self.parse_proc()?);
                continue;
            }
            break;
        }
        Ok(procs)
    }

    pub fn parse_proc(&mut self) -> miette::Result<Procedure> {
        self.expect(TokenKind::Keyword(Keyword::Proc))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::SquareOpen))?;
        let args = self.parse_fields()?;
        self.expect(TokenKind::Delimiter(Delimiter::SquareClose))?;

        let mut return_value = None;
        if self.peek() == Some(TokenKind::Punctuation(Punctuation::Colon)) {
            self.next();
            return_value = Some(self.parse_identty()?);
        }

        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let body = self.parse_block()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;

        Ok(Procedure {
            ident,
            args,
            return_value,
            body,
        })
    }

    pub fn parse_packing(&mut self) -> miette::Result<Packing> {
        self.expect(TokenKind::Keyword(Keyword::Packing))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let fields = self.parse_fields()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        // struct Foo
        // let a = self.parse_packing();
        // println!("{a:?}");
        let b = 1;
        Ok(Packing { ident, fields })
    }

    pub fn parse_aor(&mut self) -> miette::Result<Aor> {
        self.expect(TokenKind::Keyword(Keyword::Aor))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let variants = self.parse_variants()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Aor { ident, variants })
    }

    pub fn parse_identlist(&mut self) -> miette::Result<Vec<SpannedIdent>> {
        let mut idents = Vec::new();
        loop {
            if self.peek() != Some(TokenKind::Ident) {
                break;
            }
            let ident = self.parse_ident()?;
            idents.push(ident);
            if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                break;
            }
            self.next();
        }
        Ok(idents)
    }

    pub fn parse_fields(&mut self) -> miette::Result<Vec<Field>> {
        if !matches!(self.peek(), Some(TokenKind::Ident)) {
            return Ok(vec![]);
        }
        let mut fields = Vec::new();
        loop {
            let field = self.parse_field()?;
            fields.push(field);
            if !matches!(
                self.peek(),
                Some(TokenKind::Punctuation(Punctuation::Comma))
            ) {
                break;
            }
            self.next();
        }
        Ok(fields)
    }

    pub fn parse_variants(&mut self) -> miette::Result<Vec<Variant>> {
        let mut variants = Vec::new();

        loop {
            let variant = self.parse_variant()?;
            variants.push(variant);
            if !matches!(
                self.peek(),
                Some(TokenKind::Punctuation(Punctuation::Comma))
            ) {
                break;
            }
            self.next();
        }

        Ok(variants)
    }
}
