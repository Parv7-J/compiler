use std::fmt::Display;

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
    #[error("Empty import list")]
    #[diagnostic(help("You must atleast import one thing, or remove the whole get statement"))]
    EmptyImportsList {
        #[label("Expected Import list after this")]
        span: SourceSpan,
    },
    #[error("Empty Sub Api List")]
    #[diagnostic(help("You must add atleast one subapi, or remove the whole 'also' keyword"))]
    EmptySubApis {
        #[label("Expected SubApi list after this")]
        span: SourceSpan,
    },
    #[error("Invalid Token at Top level")]
    #[diagnostic(help("Only items are allowed at top level"))]
    TopLevelNonItem {
        kind: TokenKind,
        #[label("expected item found {kind}")]
        span: SourceSpan,
    },
    #[error("Unexpected Token")]
    Unexpected {
        expected: Expected,
        found: Found,
        #[label("expected {expected}, got {found}")]
        span: SourceSpan,
    },
}

#[derive(Debug)]
pub enum Expected {
    Kind(TokenKind),
    Type,
    Expr,
}

#[derive(Debug)]
pub enum Found {
    Kind(TokenKind),
    Eof,
}

impl Display for Expected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expected::Kind(token_kind) => write!(f, "{token_kind}"),
            Expected::Type => write!(f, "type"),
            Expected::Expr => write!(f, "expression"),
        }
    }
}

impl Display for Found {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Found::Kind(token_kind) => write!(f, "{token_kind}"),
            Found::Eof => write!(f, "end of file"),
        }
    }
}
