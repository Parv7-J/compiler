use super::Parser;
use super::ast::*;
use crate::lexer::token::*;

//TODO: add tokens: char, '', break, continue,
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum Operator {
//     BitwiseAnd,
//     BitwiseOr,
//     Not,
//     Assign,
//     Plus,
//     Minus,
//     Star,
//     ForwardSlash,
//     Dot,
//     Comparision(ComparisionOperator),
//     Logical(LogicalOperator),
//     CompoundAssign(CompoundAssignOperator),
// }
//
//
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum Delimiter {
//only these ->
//     ParenOpen,
//     ParenClose,
// }
//
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum ComparisionOperator {
//     LessThan,
//     GreaterThan,
//     LessEqual,
//     GreaterEqual,
//     Equal,
//     NotEqual,
// }
//
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum LogicalOperator {
//     And,
//     Or,
// }
//
// #[derive(Clone, Copy, Debug, PartialEq)]
// pub enum CompoundAssignOperator {
//     Add,
//     Sub,
//     Mul,
//     Div,
//     And,
//     Or,
// }

impl Parser<'_> {
    pub fn parse_exprstmt(&mut self) -> miette::Result<Stmt> {
        let expr = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Expr(expr))
    }

    pub fn parse_expr(&mut self) -> miette::Result<S> {
        Ok(self.expr_bp(0))
    }

    fn expr_bp(&mut self, min_bp: u8) -> S {
        let mut lhs = match self.peek() {
            Some(TokenKind::String | TokenKind::Ident | TokenKind::Number) => {
                S::Atom(self.next().unwrap().into())
            }
            Some(TokenKind::Operator(op)) => {
                let ((), r_bp) = prefix_binding_power(op).unwrap();
                self.next();
                let rhs = self.expr_bp(r_bp);
                S::Cons(op, vec![rhs])
            }
            _ => panic!("bad token"),
        };

        #[allow(clippy::all)]
        loop {
            match self.peek() {
                Some(TokenKind::Operator(op)) => {
                    let (l_bp, r_bp) = infix_binding_power(op).unwrap();
                    if l_bp < min_bp {
                        break;
                    } else {
                        self.next();
                        let rhs = self.expr_bp(r_bp);
                        lhs = S::Cons(op, vec![lhs, rhs]);
                    }
                }
                _ => todo!(),
            };
        }

        lhs
    }
}

impl From<Token> for Atom {
    fn from(value: Token) -> Self {
        match value.kind {
            TokenKind::String => Atom::String(value.span),
            TokenKind::Ident => Atom::Ident(value.span),
            TokenKind::Number => Atom::Number(value.span),
            _ => unreachable!(),
        }
    }
}

//idents, and number and string

fn infix_binding_power(op: Operator) -> Option<(u8, u8)> {
    let p = match op {
        Operator::BitwiseAnd => todo!(),
        Operator::BitwiseOr => todo!(),
        Operator::Assign => todo!(),
        Operator::Plus => (1, 2),
        Operator::Minus => todo!(),
        Operator::Star => (3, 4),
        Operator::ForwardSlash => todo!(),
        Operator::Dot => todo!(),
        Operator::Comparision(comparision_operator) => todo!(),
        Operator::Logical(logical_operator) => todo!(),
        Operator::CompoundAssign(compound_assign_operator) => todo!(),
        _ => return None,
    };
    Some(p)
}

fn prefix_binding_power(op: Operator) -> Option<((), u8)> {
    match op {
        Operator::Not => todo!(),
        Operator::Minus => todo!(),
        Operator::Star => todo!(),
        _ => None,
    }
}

fn posfix_binding_power(delim: Delimiter) -> Option<(u8, ())> {
    //a(parse_args) -> function call, a[parse_expr] -> array access
    //[parse_expr in a loop] -> array literal, "parse_string" -> string literal
    //'parse_char' -> char literal, (parse_expr) -> expression or (parse_expr in a loop,) -> tuple
    //a { parse_field } -> postfix op
    match delim {
        Delimiter::ParenOpen => todo!(),
        Delimiter::ParenClose => todo!(),
        Delimiter::SquareOpen => todo!(),
        Delimiter::SquareClose => todo!(),
        Delimiter::CurlyOpen => todo!(),
        Delimiter::CurlyClose => todo!(),
    }
}
