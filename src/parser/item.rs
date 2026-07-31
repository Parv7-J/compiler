use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::error::ParseError;

impl Parser<'_> {
    pub fn parse_get(&mut self) -> miette::Result<Get> {
        let get = self.consume();
        let imports = self.parse_identlist(TokenKind::Keyword(Keyword::From));
        if imports.is_empty() {
            return Err(ParseError::EmptyImportsList {
                span: get.span.into(),
            }
            .into());
        }
        self.expect_and_push(TokenKind::Keyword(Keyword::From));
        let module = self.parse_string()?;
        self.expect_and_push(TokenKind::Punctuation(Punctuation::Semicolon));
        Ok(Get { imports, module })
    }

    pub fn parse_require(&mut self) -> miette::Result<Require> {
        self.consume();
        let api = self.option_ident();
        self.expect_and_push(TokenKind::Keyword(Keyword::For));
        Ok(Require {
            ident: self.option_ident(),
            api,
            procedures: self.parse_procs()?,
        })
    }

    pub fn parse_api(&mut self) -> miette::Result<Api> {
        self.consume();
        let ident = self.option_ident();
        let super_api = if let Ok(token) = self.expect(TokenKind::Keyword(Keyword::Also)) {
            let list = self.parse_identlist(TokenKind::Delimiter(Delimiter::CurlyOpen));
            if list.is_empty() {
                self.errors.push(
                    ParseError::EmptySubApis {
                        span: token.span.into(),
                    }
                    .into(),
                );
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
        self.consume();
        Ok(Methods {
            ident: self.option_ident(),
            procedures: self.parse_procs()?,
        })
    }

    pub fn parse_proc(&mut self) -> miette::Result<Procedure> {
        self.consume();
        let ident = self.option_ident();
        self.expect(TokenKind::Delimiter(Delimiter::SquareOpen))?;
        let args = self.parse_fields(TokenKind::Delimiter(Delimiter::SquareClose));
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
        self.consume();
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let fields = self.parse_fields(TokenKind::Delimiter(Delimiter::CurlyClose));
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Packing { ident, fields })
    }

    pub fn parse_aor(&mut self) -> miette::Result<Aor> {
        self.consume();
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let variants = self.parse_variants();
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

    fn parse_identlist(&mut self, closing: TokenKind) -> Vec<SpannedIdent> {
        let mut idents = Vec::new();
        loop {
            match self.parse_ident() {
                Ok(ident) => idents.push(ident),
                Err(report) => {
                    self.errors.push(report);
                    self.synchronize(|kind| {
                        kind == closing
                            || matches!(kind, TokenKind::Punctuation(Punctuation::Comma))
                    });
                }
            }
            if self
                .expect(TokenKind::Punctuation(Punctuation::Comma))
                .is_err()
            {
                break;
            }
        }
        idents
    }

    fn parse_fields(&mut self, closing: TokenKind) -> Vec<Field> {
        let mut fields = Vec::new();
        loop {
            match self.parse_field() {
                Ok(field) => fields.push(field),
                Err(report) => {
                    self.errors.push(report);
                    self.synchronize(|kind| {
                        kind == closing
                            || matches!(kind, TokenKind::Punctuation(Punctuation::Comma))
                    });
                }
            };

            if self
                .expect(TokenKind::Punctuation(Punctuation::Comma))
                .is_err()
            {
                break;
            }
        }

        fields
    }

    fn parse_variants(&mut self) -> Vec<Variant> {
        let mut variants = Vec::new();
        loop {
            match self.parse_variant() {
                Ok(variant) => variants.push(variant),
                Err(report) => {
                    self.errors.push(report);
                    self.synchronize(|kind| {
                        matches!(
                            kind,
                            TokenKind::Punctuation(Punctuation::Comma)
                                | TokenKind::Delimiter(Delimiter::CurlyClose)
                        )
                    });
                }
            };

            if self
                .expect(TokenKind::Punctuation(Punctuation::Comma))
                .is_err()
            {
                break;
            }
        }

        variants
    }

    fn synchronize(&mut self, f: impl Fn(TokenKind) -> bool) {
        while let Some(kind) = self.peek() {
            if f(kind) {
                return;
            } else {
                self.consume();
            }
        }
    }

    fn option_ident(&mut self) -> Option<SpannedIdent> {
        match self.parse_ident() {
            Ok(ident) => Some(ident),
            Err(report) => {
                self.errors.push(report);
                None
            }
        }
    }
}
