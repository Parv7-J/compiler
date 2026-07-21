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
        Ok(Require {
            ident: self.parse_ident()?,
            api,
            procedures: self.parse_procs()?,
        })
    }

    pub fn parse_api(&mut self) -> miette::Result<Api> {
        self.expect(TokenKind::Keyword(Keyword::Api))?;
        let ident = self.parse_ident()?;
        let super_api = if let Ok(token) = self.expect(TokenKind::Keyword(Keyword::Also)) {
            let list = self.parse_identlist()?;
            if list.is_empty() {
                return Err(ParseError::EmptySubApis {
                    span: token.span.into(),
                }
                .into());
            }
            list
        } else {
            Vec::new()
        };
        Ok(Api {
            ident,
            super_api,
            procedures: self.parse_procs()?,
        })
    }

    pub fn parse_methods(&mut self) -> miette::Result<Methods> {
        self.expect(TokenKind::Keyword(Keyword::Methods))?;
        Ok(Methods {
            ident: self.parse_ident()?,
            procedures: self.parse_procs()?,
        })
    }

    pub fn parse_proc(&mut self) -> miette::Result<Procedure> {
        self.expect(TokenKind::Keyword(Keyword::Proc))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::SquareOpen))?;
        let args = self.parse_fields()?;
        self.expect(TokenKind::Delimiter(Delimiter::SquareClose))?;

        let return_value = if self
            .expect(TokenKind::Punctuation(Punctuation::Colon))
            .is_ok()
        {
            Some(self.parse_identty()?)
        } else {
            None
        };

        Ok(Procedure {
            ident,
            args,
            return_value,
            body: self.parse_block()?,
        })
    }

    pub fn parse_packing(&mut self) -> miette::Result<Packing> {
        self.expect(TokenKind::Keyword(Keyword::Packing))?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let fields = self.parse_fields()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
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

    fn parse_procs(&mut self) -> miette::Result<Vec<Procedure>> {
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let mut procs = Vec::new();
        loop {
            if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Proc))) {
                procs.push(self.parse_proc()?);
                continue;
            }
            break;
        }
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(procs)
    }

    fn parse_identlist(&mut self) -> miette::Result<Vec<SpannedIdent>> {
        if !matches!(self.peek(), Some(TokenKind::Ident)) {
            return Ok(vec![]);
        }
        let mut idents = Vec::new();
        loop {
            idents.push(self.parse_ident()?);
            if self
                .expect(TokenKind::Punctuation(Punctuation::Comma))
                .is_err()
            {
                break;
            }
        }
        Ok(idents)
    }

    fn parse_fields(&mut self) -> miette::Result<Vec<Field>> {
        if !matches!(self.peek(), Some(TokenKind::Ident)) {
            return Ok(vec![]);
        }
        let mut fields = Vec::new();
        loop {
            fields.push(self.parse_field()?);
            if self
                .expect(TokenKind::Punctuation(Punctuation::Comma))
                .is_err()
            {
                break;
            }
        }
        Ok(fields)
    }

    fn parse_variants(&mut self) -> miette::Result<Vec<Variant>> {
        let mut variants = Vec::new();
        loop {
            let variant = self.parse_variant()?;
            variants.push(variant);
            if self
                .expect(TokenKind::Punctuation(Punctuation::Comma))
                .is_err()
            {
                break;
            }
        }

        Ok(variants)
    }
}
