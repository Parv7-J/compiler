use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[allow(unused)]
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
    #[error("Duplicate Variant Name")]
    #[diagnostic(help("rename a variant"))]
    DuplicateVariant {
        #[label("duplicate variant")]
        duplicate_span: SourceSpan,
        #[label("variant already defined with the same name")]
        already_declared_span: SourceSpan,
    },
    #[error("Undefined Type")]
    #[diagnostic(help("define the type"))]
    UndefinedType {
        #[label("type definition not found ")]
        span: SourceSpan,
    },
    #[error("Undefined Api")]
    #[diagnostic(help("define the api"))]
    UndefinedApi {
        #[label("api definition not found ")]
        span: SourceSpan,
    },
    #[error("Not a Type")]
    #[diagnostic(help("use a different name for the type"))]
    NotAType {
        #[label("referenced here")]
        span: SourceSpan,
        #[label("item, not a type")]
        item_span: SourceSpan,
    },
    #[error("Not An Api")]
    #[diagnostic(help("use a different name for the api"))]
    NotAnApi {
        #[label("referenced here")]
        span: SourceSpan,
        #[label("item, not an api")]
        item_span: SourceSpan,
    },
    #[error("Defining methods for undefined type")]
    #[diagnostic(help("define the type before attaching methods on it"))]
    MethodsForUndefinedType {
        #[label("type not defined")]
        span: SourceSpan,
    },
    #[error("Init method not defined")]
    #[diagnostic(help("define an init method to initialize the type"))]
    InitMethodUndefined {
        #[label("in this methods block")]
        span: SourceSpan,
    },
    #[error("Defining methods for item which is not a type")]
    #[diagnostic(help("define the type before attaching methods to it"))]
    MethodsForNotAType {
        #[label("not a type")]
        span: SourceSpan,
        #[label("item, not a type")]
        item_span: SourceSpan,
    },
}
