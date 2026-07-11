mod lexer;
mod parser;

pub use lexer::Lexer;
pub use lexer::token::Token;
pub use parser::Parser;

#[derive(Debug, Clone)]
pub struct TokenStream(Vec<Token>);

impl<'a> TokenStream<'a> {
    pub fn new(lexer: Lexer<'a, std::str::CharIndices<'a>>) -> Self {
        Self(lexer.collect())
    }
}
