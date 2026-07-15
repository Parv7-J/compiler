use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::error::ParseError;

impl Parser<'_> {
    pub fn parse_block(&mut self) -> miette::Result<Block> {
        let mut block = Vec::new();
        loop {
            let token = self.peek();
            if token == Some(TokenKind::Delimiter(Delimiter::CurlyClose)) {
                break;
            }
            let Some(TokenKind::Keyword(keyword)) = token else {
                block.push(BlockItem::Stmt(self.parse_stmt()?));
                continue;
            };
            let item = match keyword {
                Keyword::Proc => {
                    let proc = self.parse_proc()?;
                    Item::Procedure(proc)
                }
                Keyword::Methods => {
                    let methods = self.parse_methods()?;
                    Item::Methods(methods)
                }
                Keyword::Require => {
                    let require = self.parse_require()?;
                    Item::Require(require)
                }
                Keyword::Aor => {
                    let aor = self.parse_aor()?;
                    Item::Aor(aor)
                }
                Keyword::Packing => {
                    let packing = self.parse_packing()?;
                    Item::Packing(packing)
                }
                Keyword::Api => {
                    let api = self.parse_api()?;
                    Item::Api(api)
                }
                _ => {
                    block.push(BlockItem::Stmt(self.parse_stmt()?));
                    continue;
                }
            };
            block.push(BlockItem::Item(item));
        }
        Ok(Block(block))
    }

    pub fn parse_range(&mut self) -> miette::Result<(S, S, Option<S>)> {
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

    pub fn parse_variant(&mut self) -> miette::Result<Variant> {
        let ident = self.parse_ident()?;
        if self.peek() != Some(TokenKind::Delimiter(Delimiter::ParenOpen)) {
            return Ok(Variant::Ident(ident));
        }
        self.next();
        let ty = self.parse_identty()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
        Ok(Variant::Field(Field { ident, ty }))
    }

    pub fn parse_field(&mut self) -> miette::Result<Field> {
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenOpen))?;
        let ty = self.parse_identty()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
        Ok(Field { ident, ty })
    }

    pub fn parse_identty(&mut self) -> miette::Result<IdentTy> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Keyword(Keyword::Type(t)),
                ..
            }) => Ok(IdentTy::Type(t)),
            Some(Token {
                kind: TokenKind::Ident,
                span,
            }) => Ok(IdentTy::Ident(Ident(self.span_to_id(span)))),
            Some(token) => Err(ParseError::UnexpectedType {
                kind: token.kind,
                span: token.span.into(),
            }
            .into()),
            None => Err(ParseError::UnexpectedEofType {
                end: (self.lexer.input.len().saturating_sub(1), 1).into(),
            }
            .into()),
        }
    }

    pub fn parse_ident(&mut self) -> miette::Result<Ident> {
        let token = self.expect(TokenKind::Ident)?;
        let id = self.span_to_id(token.span);
        Ok(Ident(id))
    }

    pub fn parse_string(&mut self) -> miette::Result<LiteralString> {
        let Token { kind: _kind, span } = self.expect(TokenKind::String)?;
        Ok(LiteralString(span))
    }
}
