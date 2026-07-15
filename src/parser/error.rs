use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::lexer::token::TokenKind;

#[derive(Error, Diagnostic, Debug)]
pub enum ParseError {
    #[error("Empty methods block")]
    #[diagnostic(help("You must define at least one 'proc' inside a methods block."))]
    EmptyMethodsBlock {
        #[label("expected procedures between these braces")]
        span: SourceSpan,
    },
    #[error("Invalid Token at Top level")]
    #[diagnostic(help("Only items are allowed at top level"))]
    TopLevelNonItem {
        kind: TokenKind,
        #[label("expected item found {kind}")]
        span: SourceSpan,
    },
    #[error("Invalid Token")]
    UnexpectedToken {
        kind: TokenKind,
        #[label("expected {expected} found {kind}")]
        span: SourceSpan,
        expected: TokenKind,
    },
    #[error("Unexpected EOF")]
    UnexpectedEof {
        kind: TokenKind,
        #[label("expected {kind}")]
        end: SourceSpan,
    },
    #[error("Unexpected Type")]
    UnexpectedType {
        kind: TokenKind,
        #[label("expected type found {kind}")]
        span: SourceSpan,
    },
    #[error("Unexpected EOF")]
    UnexpectedEofType {
        #[label("expected type")]
        end: SourceSpan,
    },
}
