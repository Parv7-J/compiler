use super::Parser;
use super::ast::*;
use crate::lexer::token::*;

impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> miette::Result<Stmt> {
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::Type(_))) => self.parse_declaration(),
            Some(TokenKind::Keyword(Keyword::If)) => self.parse_conditional(),
            Some(TokenKind::Keyword(Keyword::For)) => self.parse_for(),
            Some(TokenKind::Keyword(Keyword::While)) => self.parse_while(),
            Some(TokenKind::Keyword(Keyword::Seed)) => self.parse_seed(),
            _ => self.parse_exprstmt(),
        }
    }

    pub fn parse_declaration(&mut self) -> miette::Result<Stmt> {
        let ty = self.parse_identty()?;
        let var = self.parse_ident()?;
        self.expect(TokenKind::Operator(Operator::Assign))?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Declaration { ty, var, value })
    }

    pub fn parse_seed(&mut self) -> miette::Result<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::Seed))?;
        let return_value = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Seed { return_value })
    }

    pub fn parse_conditional(&mut self) -> miette::Result<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::If))?;
        let condition = self.parse_expr()?;
        println!("condition: {condition:?}");
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let block = self.parse_block()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        let ifs = Conditional { condition, block };
        let (elseifs, elses) = self.parse_else()?;
        Ok(Stmt::Conditional {
            ifs,
            elseifs,
            elses,
        })
    }

    pub fn parse_else(&mut self) -> miette::Result<(Vec<Conditional>, Option<Block>)> {
        let mut elseifs = Vec::new();
        let mut elses = None;
        #[allow(clippy::while_let_loop)]
        loop {
            match self.peek() {
                Some(TokenKind::Keyword(Keyword::Else)) => {
                    self.next();
                    match self.peek() {
                        Some(TokenKind::Keyword(Keyword::If)) => {
                            self.next();
                            let condition = self.parse_expr()?;
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
                            let block = self.parse_block()?;
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                            elseifs.push(Conditional { condition, block });
                        }
                        _ => {
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
                            let block = self.parse_block()?;
                            self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                            elses = Some(block);
                            break;
                        }
                    }
                }
                _ => break,
            }
        }
        Ok((elseifs, elses))
    }

    pub fn parse_for(&mut self) -> miette::Result<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::For))?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenOpen))?;
        let ty = self.parse_identty()?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
        self.expect(TokenKind::Keyword(Keyword::In))?;
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::Range)) => {
                self.next();
                let range = self.parse_range()?;
                self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
                let block = self.parse_block()?;
                self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                Ok(Stmt::For {
                    ident,
                    ty,
                    range: Some(range),
                    block,
                })
            }
            _ => {
                self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
                let block = self.parse_block()?;
                self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
                Ok(Stmt::For {
                    ident,
                    ty,
                    range: None,
                    block,
                })
            }
        }
    }

    pub fn parse_while(&mut self) -> miette::Result<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::While))?;
        let condition = self.parse_expr()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyOpen))?;
        let block = self.parse_block()?;
        self.expect(TokenKind::Delimiter(Delimiter::CurlyClose))?;
        Ok(Stmt::While { condition, block })
    }
}
