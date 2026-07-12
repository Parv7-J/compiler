mod lexer;
mod parser;

pub use lexer::Lexer;
pub use lexer::token::Token;
// pub use parser::Parser;

use crate::lexer::token::TokenKind;

#[derive(Debug, Clone)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
    pub newlines: Vec<u32>,
}

impl TokenStream {
    pub fn new<'a>(lexer: Lexer<'a, std::str::Chars<'a>>) -> Self {
        let mut lexer = lexer;
        let mut tokens = Vec::new();
        loop {
            let token = lexer.advance_token();
            if let TokenKind::Eof = token.kind {
                break;
            }
            tokens.push(token);
        }
        Self {
            tokens,
            newlines: lexer.newlines,
        }
    }
}
