use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::error::Expected;
use crate::parser::error::Found;
use crate::parser::error::ParseError;

impl Parser<'_> {
    pub fn parse_block(&mut self) -> miette::Result<Block> {
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let mut block = Vec::new();
        while !matches!(
            self.peek(),
            Some(TokenKind::Delimiter(Delimiter::CurlyClose)),
        ) {
            if matches!(
                self.peek(),
                Some(TokenKind::Delimiter(Delimiter::CurlyOpen))
            ) {
                block.push(BlockItem::Item(Item::Block(self.parse_block()?)));
                continue;
            }
            let Some(TokenKind::Keyword(keyword)) = self.peek() else {
                block.push(BlockItem::Stmt(self.parse_stmt()?));
                continue;
            };
            let item = match keyword {
                Keyword::Proc => Item::Procedure(self.parse_proc()?),
                Keyword::Methods => Item::Methods(self.parse_methods()?),
                Keyword::Require => Item::Require(self.parse_require()?),
                Keyword::Aor => Item::Aor(self.parse_aor()?),
                Keyword::Packing => Item::Packing(self.parse_packing()?),
                Keyword::Api => Item::Api(self.parse_api()?),
                _ => {
                    block.push(BlockItem::Stmt(self.parse_stmt()?));
                    continue;
                }
            };
            block.push(BlockItem::Item(item));
        }
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))
            .unwrap();
        Ok(Block(block))
    }

    pub fn parse_range(&mut self) -> miette::Result<Range> {
        let start = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
        let end = self.parse_expr()?;
        let step = match self.peek() {
            Some(TokenKind::Punctuation(Punctuation::Comma)) => {
                self.next();
                Some(self.parse_expr()?)
            }
            _ => None,
        };
        Ok(Range { start, end, step })
    }

    pub fn parse_variant(&mut self) -> miette::Result<Variant> {
        let ident = self.parse_ident()?;
        if self
            .expect(TokenKind::Delimiter(Delimiter::ParenOpen))
            .is_err()
        {
            return Ok(Variant::SpannedIdent(ident));
        }
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
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::Type(_))) => self.parse_builtintype(),
            Some(TokenKind::Ident) => Ok(IdentTy::Ident(SpannedIdent(
                self.expect(TokenKind::Ident)?.span,
            ))),
            Some(TokenKind::Operator(Operator::Star)) => Ok(IdentTy::Ptr(self.parse_ptr()?)),
            Some(_) => {
                let token = self.token.or_else(|| self.lexer.clone().next()).unwrap();
                Err(ParseError::Unexpected {
                    expected: Expected::Type,
                    found: Found::Kind(token.kind),
                    span: token.span.into(),
                }
                .into())
            }
            None => Err(ParseError::Unexpected {
                expected: Expected::Type,
                found: Found::Eof,
                span: (self.lexer.input().len().saturating_sub(1), 1).into(),
            }
            .into()),
        }
    }

    fn parse_ptr(&mut self) -> miette::Result<SpannedPtr> {
        let star = self.expect(TokenKind::Operator(Operator::Star))?;
        let ty = self.parse_identty()?;
        Ok(SpannedPtr {
            ptr: star.span,
            ty: Box::new(ty),
        })
    }

    fn parse_arr(&mut self) -> miette::Result<SpannedArr> {
        let (arr_ty, span) = match self.next().unwrap() {
            Token {
                kind: TokenKind::Keyword(Keyword::Type(ty)),
                span,
            } => {
                let arr_ty = match ty {
                    Ty::Arr => ArrType::Arr,
                    Ty::HeapArr => ArrType::HeapArr,
                    _ => unreachable!(),
                };
                (arr_ty, span)
            }
            _ => unreachable!(),
        };
        self.expect(TokenKind::Delimiter(Delimiter::SquareOpen))?;
        let inner_ty = self.parse_identty()?;
        self.expect(TokenKind::Delimiter(Delimiter::SquareClose))?;
        Ok(SpannedArr {
            arr_ty,
            inner_ty: Box::new(inner_ty),
            span,
        })
    }

    pub fn parse_ident(&mut self) -> miette::Result<SpannedIdent> {
        let token = self.expect(TokenKind::Ident)?;
        Ok(SpannedIdent(token.span))
    }

    fn parse_builtintype(&mut self) -> miette::Result<IdentTy> {
        let TokenKind::Keyword(Keyword::Type(ty)) = self.peek().unwrap() else {
            unreachable!()
        };

        Ok(match ty {
            Ty::Arr | Ty::HeapArr => IdentTy::Arr(self.parse_arr()?),
            _ => IdentTy::Type(self.parse_ty()?),
        })
    }

    pub fn parse_ty(&mut self) -> miette::Result<SpannedTy> {
        match self.next() {
            Some(Token {
                kind: TokenKind::Keyword(Keyword::Type(ty)),
                span,
            }) => Ok(SpannedTy { ty, span }),
            Some(t) => Err(ParseError::Unexpected {
                expected: Expected::Type,
                found: Found::Kind(t.kind),
                span: t.span.into(),
            }
            .into()),
            None => Err(ParseError::Unexpected {
                expected: Expected::Type,
                found: Found::Eof,
                span: (self.lexer.input().len().saturating_sub(1), 1).into(),
            }
            .into()),
        }
    }

    pub fn parse_string(&mut self) -> miette::Result<LiteralString> {
        let Token { kind: _kind, span } = self.expect(TokenKind::String)?;
        Ok(LiteralString(span))
    }
}
