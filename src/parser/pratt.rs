//TODO: tuple types, hand initializing structs

use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::error::Expected;
use crate::parser::error::Found;
use crate::parser::error::ParseError;

impl Parser<'_> {
    pub fn parse_exprstmt(&mut self) -> miette::Result<Stmt> {
        //allow assignments inside expression statements
        let expr = self.expr_bp(0)?;
        // println!("{expr:#?}");
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Expr(expr))
    }

    pub fn parse_expr(&mut self) -> miette::Result<Expr> {
        let expr = self.expr_bp(3)?;
        // println!("{expr:#?}");
        Ok(expr)
    }

    fn expr_bp(&mut self, mut min_bp: u8) -> miette::Result<Expr> {
        let token = match self.next() {
            Some(token) => token,
            None => {
                return Err(ParseError::Unexpected {
                    expected: Expected::Expr,
                    found: Found::Eof,
                    span: (self.lexer.input.len().saturating_sub(1), 1).into(),
                }
                .into());
            }
        };
        let mut lhs = match token.kind {
            TokenKind::Delimiter(Delimiter::ParenOpen) => {
                let inner_expr = self.parse_expr()?;
                self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
                inner_expr
            }
            TokenKind::Delimiter(Delimiter::SquareOpen) => {
                let mut arr = Vec::new();
                loop {
                    if self.peek().is_none()
                        || self.peek() == Some(TokenKind::Delimiter(Delimiter::SquareClose))
                    {
                        break;
                    }
                    arr.push(self.parse_expr()?);
                    if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                        break;
                    }
                    self.expect(TokenKind::Punctuation(Punctuation::Comma))
                        .expect("already checked for comma");
                }
                self.expect(TokenKind::Delimiter(Delimiter::SquareClose))?;
                Expr::List(arr)
            }
            TokenKind::Ident => Expr::Atom(Atom::Ident(SpannedIdent(token.span))),
            TokenKind::Keyword(Keyword::Boolean(boolean)) => {
                Expr::Atom(Atom::Boolean(SpannedBoolean {
                    boolean,
                    span: token.span,
                }))
            }
            TokenKind::String | TokenKind::Number => Expr::Atom(token.into()),
            TokenKind::Operator(op) => {
                let ((), r_bp) = prefix_binding_power(op).unwrap();
                let rhs = self.expr_bp(r_bp)?;
                Expr::Cons(
                    SpannedOperator {
                        op,
                        span: token.span,
                    },
                    vec![rhs],
                )
            }
            kind => {
                return Err(ParseError::Unexpected {
                    expected: Expected::Expr,
                    found: Found::Kind(kind),
                    span: token.span.into(),
                }
                .into());
            }
        };

        while let Some(kind) = self.peek() {
            match kind {
                TokenKind::Operator(op) => {
                    let (l_bp, mut r_bp) = infix_binding_power(op).unwrap();
                    if l_bp < min_bp {
                        break;
                    }
                    match op {
                        Operator::Assign | Operator::CompoundAssign(_) => {
                            min_bp = 3;
                            r_bp = r_bp.max(min_bp);
                        }
                        _ => {}
                    }
                    let t = self.next().unwrap();
                    let rhs = self.expr_bp(r_bp)?;
                    lhs = Expr::Cons(SpannedOperator { op, span: t.span }, vec![lhs, rhs]);
                }
                TokenKind::Delimiter(delim) => {
                    if delim == Delimiter::ParenOpen {
                        println!("here fam");
                        let l_bp = 19;
                        if l_bp < min_bp {
                            break;
                        }
                        self.next().unwrap();
                        let mut arr = Vec::new();
                        loop {
                            if self.peek().is_none()
                                || self.peek() == Some(TokenKind::Delimiter(Delimiter::ParenClose))
                            {
                                break;
                            }
                            arr.push(self.parse_expr()?);
                            if self.peek() != Some(TokenKind::Punctuation(Punctuation::Comma)) {
                                break;
                            }
                            self.expect(TokenKind::Punctuation(Punctuation::Comma))?;
                        }
                        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
                        lhs = Expr::Call(vec![lhs, Expr::List(arr)]);
                    }

                    let (l_bp, expected_kind) = match delim {
                        Delimiter::SquareOpen => (19, TokenKind::Delimiter(Delimiter::SquareClose)),
                        _ => break,
                    };
                    if l_bp < min_bp {
                        break;
                    }
                    //access should have exactly one expr
                    self.next().unwrap();
                    let expr = self.parse_expr()?;
                    self.expect(expected_kind)?;
                    lhs = Expr::Access(vec![lhs, expr]);
                }
                _ => break,
            }
        }

        Ok(lhs)
    }
}

impl From<Token> for Atom {
    fn from(value: Token) -> Self {
        match value.kind {
            TokenKind::String => Atom::String(value.span),
            TokenKind::Number => Atom::Number(value.span),
            _ => unreachable!(),
        }
    }
}

fn infix_binding_power(op: Operator) -> Option<(u8, u8)> {
    match op {
        Operator::Dot => Some((19, 20)),
        Operator::Star | Operator::ForwardSlash => Some((15, 16)),
        Operator::Plus | Operator::Minus => Some((13, 14)),
        Operator::Comparision(_) => Some((11, 12)),
        Operator::BitwiseAnd => Some((9, 10)),
        Operator::BitwiseOr => Some((7, 8)),
        Operator::Logical(LogicalOperator::And) => Some((5, 6)),
        Operator::Logical(LogicalOperator::Or) => Some((3, 4)),
        Operator::Assign | Operator::CompoundAssign(_) => Some((2, 1)),
        _ => None,
    }
}

fn prefix_binding_power(op: Operator) -> Option<((), u8)> {
    match op {
        Operator::Not | Operator::Minus | Operator::Star | Operator::BitwiseAnd => Some(((), 17)),
        _ => None,
    }
}
