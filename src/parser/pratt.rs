use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::error::ParseError;

impl Parser<'_> {
    pub fn parse_exprstmt(&mut self) -> miette::Result<Stmt> {
        //allow assignments inside expression statements
        let expr = self.expr_bp(0)?;
        println!("{expr:#?}");
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Expr(expr))
    }

    pub fn parse_expr(&mut self) -> miette::Result<S> {
        let expr = self.expr_bp(3)?;
        println!("{expr:#?}");
        Ok(expr)
    }

    fn expr_bp(&mut self, mut min_bp: u8) -> miette::Result<S> {
        let token = match self.next() {
            Some(token) => token,
            None => {
                return Err(ParseError::UnexpectedEofExpr {
                    end: (self.lexer.input.len().saturating_sub(1), 1).into(),
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
            TokenKind::Ident => {
                let id = self.span_to_id(token.span);
                S::Atom(Atom::Ident(SpannedIdent {
                    id,
                    span: token.span,
                }))
            }
            TokenKind::String | TokenKind::Number => S::Atom(token.into()),
            TokenKind::Operator(op) => {
                let ((), r_bp) = prefix_binding_power(op).unwrap();
                let rhs = self.expr_bp(r_bp)?;
                S::Cons(
                    SpannedOperator {
                        op,
                        span: token.span,
                    },
                    vec![rhs],
                )
            }
            kind => {
                return Err(ParseError::UnexpectedExpr {
                    kind,
                    span: token.span.into(),
                }
                .into());
            }
        };

        while let Some(TokenKind::Operator(op)) = self.peek() {
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
            lhs = S::Cons(SpannedOperator { op, span: t.span }, vec![lhs, rhs]);
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

// fn posfix_binding_power(delim: Delimiter) -> Option<(u8, ())> {
//     //a(parse_args) -> function call, a[parse_expr] -> array access
//     //[parse_expr in a loop] -> array literal, "parse_string" -> string literal
//     //'parse_char' -> char literal, (parse_expr) -> expression or (parse_expr in a loop,) -> tuple
//     //a { parse_field } -> postfix op
//     match delim {
//         Delimiter::ParenOpen |
//         Delimiter::SquareOpen => Some((19))
//         Delimiter::CurlyOpen => todo!(),
//     }
// }
