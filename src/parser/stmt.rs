use super::Parser;
use super::ast::*;
use crate::lexer::token::*;

impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> miette::Result<Stmt> {
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::If)) => self.parse_conditional(),
            Some(TokenKind::Keyword(Keyword::For)) => self.parse_for(),
            Some(TokenKind::Keyword(Keyword::While)) => self.parse_while(),
            Some(TokenKind::Keyword(Keyword::Seed)) => self.parse_seed(),
            Some(TokenKind::Keyword(Keyword::Break)) => {
                let token = self.next().unwrap();
                self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
                Ok(Stmt::Break(token.span))
            }
            Some(TokenKind::Keyword(Keyword::Continue)) => {
                let token = self.next().unwrap();
                self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
                Ok(Stmt::Continue(token.span))
            }
            _ => {
                if self.is_declaration() {
                    self.parse_declaration()
                } else {
                    self.parse_exprstmt()
                }
            }
        }
    }

    pub fn is_declaration(&mut self) -> bool {
        let mut parser = Parser {
            lexer: self.lexer.clone(),
            token: self.token,
            errors: vec![],
        };

        if parser.parse_identty().is_ok() && parser.parse_ident().is_ok() {
            return true;
        }

        false
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
        let block = self.parse_block()?;
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
                            let block = self.parse_block()?;
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
        let ty = self.parse_ty()?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
        self.expect(TokenKind::Keyword(Keyword::In))?;
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::Range)) => {
                self.next();
                let range = self.parse_range()?;
                let block = self.parse_block()?;
                Ok(Stmt::For {
                    ident,
                    ty,
                    range: Some(range),
                    inn: None,
                    block,
                })
            }
            _ => {
                let inn = self.parse_ident()?;
                let block = self.parse_block()?;
                Ok(Stmt::For {
                    ident,
                    ty,
                    range: None,
                    inn: Some(inn),
                    block,
                })
            }
        }
    }

    pub fn parse_while(&mut self) -> miette::Result<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::While))?;
        let condition = self.parse_expr()?;
        let block = self.parse_block()?;
        Ok(Stmt::While { condition, block })
    }
}
