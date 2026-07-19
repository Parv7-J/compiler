use std::fmt::Display;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use crate::lexer::token::TokenKind;

#[derive(Error, Diagnostic, Debug, Clone)]
pub enum AnalysisError {
    #[error("Duplicate Item Name")]
    #[diagnostic(help("rename an item"))]
    DuplicateItem {
        #[label("duplicate item")]
        duplicate_span: SourceSpan,
        #[label("item already defined with the same name")]
        already_declared_span: SourceSpan,
    },
    #[error("Duplicate Field Name")]
    #[diagnostic(help("rename a field"))]
    DuplicateField {
        #[label("duplicate field")]
        duplicate_span: SourceSpan,
        #[label("field already defined with the same name")]
        already_declared_span: SourceSpan,
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
