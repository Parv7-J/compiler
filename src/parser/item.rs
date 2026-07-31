use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::ParseResult;
use crate::parser::error::Found;
use crate::parser::error::ParseError;

impl Parser<'_> {
    pub fn parse_get(&mut self) -> ParseResult<Get> {
        let get = self.consume();
        let imports =
            self.parse_delimited(COMMA, TokenKind::Keyword(Keyword::From), Self::parse_ident);
        if imports.is_empty() {
            return Err(ParseError::EmptyImportsList {
                span: get.span.into(),
            });
        }
        self.expect(TokenKind::Keyword(Keyword::From))?;
        let module = self.parse_string()?;
        self.expect_and_push(SEMICOLON);
        Ok(Get { imports, module })
    }

    pub fn parse_api(&mut self) -> ParseResult<Api> {
        self.consume();
        let ident = self.option_ident();
        let super_api = if let Ok(token) = self.expect(TokenKind::Keyword(Keyword::Also)) {
            let list = self.parse_delimited(COMMA, C_CLOSE, Self::parse_ident);
            if list.is_empty() {
                self.errors.push(ParseError::EmptySubApis {
                    span: token.span.into(),
                });
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

    pub fn parse_methods(&mut self) -> ParseResult<Methods> {
        self.consume();
        Ok(Methods {
            ident: self.option_ident(),
            procedures: self.parse_procs()?,
        })
    }

    pub fn parse_require(&mut self) -> ParseResult<Require> {
        self.consume();
        let api = self.option_ident();
        self.expect_and_push(TokenKind::Keyword(Keyword::For));
        Ok(Require {
            ident: self.option_ident(),
            api,
            procedures: self.parse_procs()?,
        })
    }

    pub fn parse_proc(&mut self) -> ParseResult<Procedure> {
        self.consume();
        let ident = self.option_ident();
        self.expect(S_OPEN)?;
        let args = self.parse_delimited(COMMA, S_CLOSE, Self::parse_field);
        self.expect(S_CLOSE)?;

        let return_value = if self.expect(COLON).is_ok() {
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

    pub fn parse_packing(&mut self) -> ParseResult<Packing> {
        self.consume();
        let ident = self.parse_ident()?;
        self.expect(C_OPEN)?;
        let fields = self.parse_delimited(COMMA, C_CLOSE, Self::parse_field);
        self.expect(C_CLOSE)?;
        Ok(Packing { ident, fields })
    }

    pub fn parse_aor(&mut self) -> ParseResult<Aor> {
        self.consume();
        let ident = self.parse_ident()?;
        self.expect(C_OPEN)?;
        let variants = self.parse_delimited(COMMA, C_CLOSE, Self::parse_variant);
        self.expect(C_CLOSE)?;
        Ok(Aor { ident, variants })
    }

    fn parse_procs(&mut self) -> ParseResult<Vec<Procedure>> {
        self.expect_and_push(C_OPEN);
        let mut procs = Vec::new();
        loop {
            if matches!(self.peek(), Some(TokenKind::Keyword(Keyword::Proc))) {
                procs.push(self.parse_proc()?);
                continue;
            }
            break;
        }
        self.expect_and_push(C_CLOSE);
        Ok(procs)
    }

    pub fn parse_delimited<T>(
        &mut self,
        delimiter: TokenKind,
        end: TokenKind,
        mut method: impl FnMut(&mut Self) -> ParseResult<T>,
    ) -> Vec<T> {
        match self.peek() {
            Some(tk) if tk == end => return vec![],
            None => return vec![],
            _ => {}
        };

        let mut list = Vec::new();

        loop {
            match method(self) {
                Ok(list_item) => list.push(list_item),
                Err(report) => {
                    self.errors.push(report);
                    self.synchronize(|kind| kind == delimiter || kind == end);
                }
            }

            if let Err(ParseError::Unexpected {
                expected: _,
                found,
                span,
            }) = self.expect(delimiter)
            {
                if !matches!(found, Found::Kind(TokenKind::Ident)) {
                    break;
                }

                self.errors
                    .push(ParseError::MustDelimit { delimiter, span });
            }
        }
        list
    }

    ///syncs till the next token according to f, or EOF.
    ///doesnt consume the sync token
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
