use anyhow::Context;

use super::Parser;
use super::ast::*;
use crate::lexer::token::*;

impl Parser<'_> {
    pub fn parse_get(&mut self) -> anyhow::Result<Get> {
        self.expect(TokenKind::Keyword(Keyword::Get))?;
        let imports = self.parse_identlist()?;
        anyhow::ensure!(imports.len() > 0);
        self.expect(TokenKind::Keyword(Keyword::From))?;
        let module = self.parse_string()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Get { imports, module })
    }

    pub fn parse_require(&mut self) -> anyhow::Result<Require> {
        self.expect(TokenKind::Keyword(Keyword::Require))?;
        let api = self.parse_ident()?;
        self.expect(TokenKind::Keyword(Keyword::For))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let procs = self.parse_procs(true)?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Require {
            ident,
            api,
            procedures: procs,
        })
    }

    pub fn parse_api(&mut self) -> anyhow::Result<Api> {
        self.expect(TokenKind::Keyword(Keyword::Api))?;
        let ident = self.parse_ident()?;
        let super_api = if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Also))) {
            self.next();
            let list = self.parse_identlist()?;
            anyhow::ensure!(list.len() > 0);
            list
        } else {
            vec![]
        };
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let procedures = self.parse_procs(true)?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Api {
            ident,
            super_api,
            procedures,
        })
    }

    pub fn parse_methods(&mut self) -> anyhow::Result<Methods> {
        let context = "Parsing Methods";
        self.expect(TokenKind::Keyword(Keyword::Methods))
            .context(context)?;
        let ident = self.parse_ident().context(context)?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))
            .context(context)?;
        let procs = self.parse_procs(false).context(context)?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))
            .context(context)?;
        Ok(Methods {
            ident,
            procedures: procs,
        })
    }

    pub fn parse_procs(&mut self, allow_empty: bool) -> anyhow::Result<Vec<Procedure>> {
        let mut procs = Vec::new();
        loop {
            if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Proc))) {
                procs.push(self.parse_proc()?);
                continue;
            }
            break;
        }
        if procs.is_empty() && !allow_empty {
            anyhow::bail!("No procedures defined in scope");
        }
        Ok(procs)
    }

    pub fn parse_proc(&mut self) -> anyhow::Result<Procedure> {
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

    pub fn parse_packing(&mut self) -> anyhow::Result<Packing> {
        self.expect(TokenKind::Keyword(Keyword::Packing))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let fields = self.parse_fields()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Packing { ident, fields })
    }

    pub fn parse_aor(&mut self) -> anyhow::Result<Aor> {
        self.expect(TokenKind::Keyword(Keyword::Aor))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let variants = self.parse_variants()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Aor { ident, variants })
    }

    pub fn parse_identlist(&mut self) -> anyhow::Result<Vec<Ident>> {
        let mut idents = Vec::new();
        loop {
            let ident = self.parse_ident()?;
            idents.push(ident);
            if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                break;
            }
            self.next();
        }
        Ok(idents)
    }

    pub fn parse_fields(&mut self) -> anyhow::Result<Vec<Field>> {
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

    pub fn parse_variants(&mut self) -> anyhow::Result<Vec<Variant>> {
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
