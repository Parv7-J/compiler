use anyhow::Context;

use super::Parser;
use super::ast::*;
use crate::lexer::token::*;

impl Parser<'_> {
    pub fn parse_exprstmt(&mut self) -> anyhow::Result<Stmt> {
        let expr = self
            .parse_expr()
            .context("Parsing an expression statement")?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))
            .context("Expression statement must end with semicolon")?;
        Ok(Stmt::Expr(expr))
    }

    pub fn parse_expr(&mut self) -> anyhow::Result<Expr> {
        Ok(Expr)
    }
}
