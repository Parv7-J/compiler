use super::Parser;
use super::ast::*;
use crate::lexer::token::*;
use crate::parser::ParseResult;

impl Parser<'_> {
    pub fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        match self.peek() {
            Some(TokenKind::Keyword(Keyword::If)) => self.parse_if(),
            Some(TokenKind::Keyword(Keyword::For)) => self.parse_for(),
            Some(TokenKind::Keyword(Keyword::While)) => self.parse_while(),
            Some(TokenKind::Keyword(Keyword::Seed)) => self.parse_seed(),
            Some(TokenKind::Keyword(Keyword::Break)) => {
                let token = self.consume();
                self.expect(SEMICOLON)?;
                Ok(Stmt::Break(token.span))
            }
            Some(TokenKind::Keyword(Keyword::Continue)) => {
                let token = self.consume();
                self.expect(SEMICOLON)?;
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

    fn is_declaration(&mut self) -> bool {
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

    fn parse_declaration(&mut self) -> ParseResult<Stmt> {
        let ty = self.parse_identty()?;
        let var = self.parse_ident()?;
        self.expect(TokenKind::Operator(Operator::Assign))?;
        let value = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Declaration { ty, var, value })
    }

    fn parse_if(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::If))?;
        let ifs = Conditional {
            condition: self.parse_expr()?,
            block: self.parse_block()?,
        };
        let (elseifs, elses) = self.parse_else()?;
        Ok(Stmt::If {
            ifs,
            elseifs,
            elses,
        })
    }

    fn parse_else(&mut self) -> ParseResult<(Vec<Conditional>, Option<Block>)> {
        let mut elseifs = Vec::new();
        let mut elses = None;
        while self.expect(TokenKind::Keyword(Keyword::Else)).is_ok() {
            if self.expect(TokenKind::Keyword(Keyword::If)).is_err() {
                elses = Some(self.parse_block()?);
                break;
            }
            elseifs.push(Conditional {
                condition: self.parse_expr()?,
                block: self.parse_block()?,
            });
        }
        Ok((elseifs, elses))
    }

    fn parse_for(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::For))?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenOpen))?;
        let ty = self.parse_ty()?;
        let ident = self.parse_ident()?;
        self.expect(TokenKind::Delimiter(Delimiter::ParenClose))?;
        self.expect(TokenKind::Keyword(Keyword::In))?;
        let (range, collection) = if self.expect(TokenKind::Keyword(Keyword::Range)).is_ok() {
            (Some(self.parse_range()?), None)
        } else {
            (None, Some(self.parse_ident()?))
        };
        Ok(Stmt::For {
            ident,
            ty,
            range,
            collection,
            block: self.parse_block()?,
        })
    }

    fn parse_while(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::While))?;
        Ok(Stmt::While {
            condition: self.parse_expr()?,
            block: self.parse_block()?,
        })
    }

    fn parse_seed(&mut self) -> ParseResult<Stmt> {
        self.expect(TokenKind::Keyword(Keyword::Seed))?;
        let return_value = self.parse_expr()?;
        self.expect(TokenKind::Punctuation(Punctuation::Semicolon))?;
        Ok(Stmt::Seed { return_value })
    }
}
