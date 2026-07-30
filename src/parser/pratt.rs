//TODO: tuple types, hand initializing structs

use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::error::Expected;
use crate::parser::error::Found;
use crate::parser::error::ParseError;

const EXPR_STMT_BP: u8 = 0;
const EXPR_BP: u8 = 3;
const POSTFIX_BP: u8 = 19;

impl Parser<'_> {
    ///Expression statements are expressions that end with a semicolon, thus they allow assignments
    pub fn parse_exprstmt(&mut self) -> miette::Result<Stmt> {
        let expr = self.expr_bp(EXPR_STMT_BP)?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Expr(expr))
    }

    ///Expressions evaluate to a value, thus no assignments are allowed in expressions
    pub fn parse_expr(&mut self) -> miette::Result<Expr> {
        self.expr_bp(EXPR_BP)
    }

    fn expr_bp(&mut self, mut min_bp: u8) -> miette::Result<Expr> {
        let token = match self.next() {
            Some(token) => token,
            None => {
                return Err(ParseError::Unexpected {
                    expected: Expected::Expr,
                    found: Found::Eof,
                    span: (self.lexer.input().len().saturating_sub(1), 1).into(),
                }
                .into());
            }
        };

        let mut lhs = match token.kind {
            TokenKind::Delimiter(Delimiter::ParenOpen) => {
                let expr_inner = self.parse_expr()?;
                self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
                expr_inner
            }
            TokenKind::Delimiter(Delimiter::SquareOpen) => {
                let mut expr_list = Vec::new();
                loop {
                    if self.peek().is_none()
                        || self.peek() == Some(TokenKind::Delimiter(Delimiter::SquareClose))
                    {
                        break;
                    }
                    expr_list.push(self.parse_expr()?);
                    if self
                        .expect(TokenKind::Punctuation(Punctuation::Comma))
                        .is_err()
                    {
                        break;
                    }
                }
                self.expect(TokenKind::Delimiter(Delimiter::SquareClose))?;
                Expr::List(ExprList(expr_list))
            }
            TokenKind::Ident => Expr::Atom(Atom::Ident(SpannedIdent(token.span))),
            TokenKind::Keyword(Keyword::Boolean(boolean)) => {
                Expr::Atom(Atom::Boolean(SpannedBoolean {
                    boolean,
                    span: token.span,
                }))
            }
            TokenKind::Keyword(Keyword::This) => Expr::Atom(Atom::This(token.span)),
            TokenKind::String => Expr::Atom(Atom::String(token.span)),
            TokenKind::Number => Expr::Atom(Atom::Number(token.span)),
            TokenKind::Operator(op) => {
                let ((), r_bp) = prefix_binding_power(op).ok_or(ParseError::NotPrefix {
                    op,
                    span: token.span.into(),
                })?;

                let operand = Box::new(self.expr_bp(r_bp)?);
                Expr::Prefix {
                    op: SpannedOperator {
                        op,
                        span: token.span,
                    },
                    operand,
                }
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
                    let (l_bp, mut r_bp) = infix_binding_power(op).ok_or(ParseError::NotInfix {
                        op,
                        span: token.span.into(),
                    })?;
                    if l_bp < min_bp {
                        break;
                    }
                    match op {
                        Operator::Assign | Operator::CompoundAssign(_) => {
                            min_bp = EXPR_BP;
                            r_bp = r_bp.max(min_bp);
                        }
                        _ => {}
                    }
                    let t = self.next().unwrap();
                    let rhs = Box::new(self.expr_bp(r_bp)?);
                    lhs = Expr::Infix {
                        op: SpannedOperator { op, span: t.span },
                        lhs: Box::new(lhs),
                        rhs,
                    };
                }
                TokenKind::Delimiter(delim) => {
                    if delim == Delimiter::ParenOpen {
                        let l_bp = POSTFIX_BP;
                        if l_bp < min_bp {
                            break;
                        }
                        self.next().unwrap();
                        let mut argument_list = Vec::new();
                        loop {
                            if self.peek().is_none()
                                || self.peek() == Some(TokenKind::Delimiter(Delimiter::ParenClose))
                            {
                                break;
                            }
                            argument_list.push(self.parse_expr()?);
                            if self
                                .expect(TokenKind::Punctuation(Punctuation::Comma))
                                .is_err()
                            {
                                break;
                            }
                        }
                        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
                        lhs = Expr::Call {
                            function: Box::new(lhs),
                            arguments: ExprList(argument_list),
                        };
                    }

                    let (l_bp, expected_kind) = match delim {
                        Delimiter::SquareOpen => (19, TokenKind::Delimiter(Delimiter::SquareClose)),
                        _ => break,
                    };
                    if l_bp < min_bp {
                        break;
                    }
                    self.next().unwrap();
                    let rhs = Box::new(self.parse_expr()?);
                    self.expect(expected_kind)?;
                    lhs = Expr::Access {
                        lhs: Box::new(lhs),
                        rhs,
                    }
                }
                _ => break,
            }
        }

        Ok(lhs)
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

// pub enum InfixOperator {
//         Dot,
//         Star,
//         ForwardSlash,
//         Plus,
//         Minus,
//         Comparision(_) => Some((11, 12)),
//         BitwiseAnd => Some((9, 10)),
//         BitwiseOr => Some((7, 8)),
//         Logical(LogicalOperator::And) => Some((5, 6)),
//         Logical(LogicalOperator::Or) => Some((3, 4)),
//         Assign | Operator::CompoundAssign(_) => Some((2, 1)),
// }
