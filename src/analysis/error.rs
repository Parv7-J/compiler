use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug, Clone)]
pub enum AnalysisError {
    #[error("invalid type found")]
    InvalidType {
        #[label("type not defined")]
        span: SourceSpan,
        #[label("not a type")]
        item_span: Option<SourceSpan>,
    },
    #[error("invalid api found")]
    InvalidApi {
        #[label("api not defined")]
        span: SourceSpan,
        #[label("not an api")]
        item_span: Option<SourceSpan>,
    },
    #[error("Duplicate Item Name")]
    #[diagnostic(help("rename an item"))]
    DuplicateItem {
        #[label("duplicate item")]
        duplicate_span: SourceSpan,
        #[label("item already defined with the same name")]
        declared_span: SourceSpan,
    },
    #[error("Duplicate Method")]
    #[diagnostic(help("rename a method"))]
    DuplicateMethod {
        #[label("duplicate method")]
        duplicate_span: SourceSpan,
        #[label("method already defined with the same name")]
        declared_span: SourceSpan,
    },
    #[error("Duplicate Field Name")]
    #[diagnostic(help("rename a field"))]
    DuplicateField {
        #[label("duplicate field")]
        duplicate_span: SourceSpan,
        #[label("field already defined with the same name")]
        declared_span: SourceSpan,
    },
    #[error("Duplicate Variant Name")]
    #[diagnostic(help("rename a variant"))]
    DuplicateVariant {
        #[label("duplicate variant")]
        duplicate_span: SourceSpan,
        #[label("variant already defined with the same name")]
        declared_span: SourceSpan,
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
    #[error("Unimplemented Method required by Api require")]
    #[diagnostic(help("define the required method"))]
    UnimplementedMethod {
        #[label("requires this method to be implemented")]
        method_span: SourceSpan,
        #[label("this api")]
        api_span: SourceSpan,
        #[label("we require the api for this type")]
        require_span: SourceSpan,
    },
    #[error("Method not required by the Api")]
    #[diagnostic(help("remove the method"))]
    ExtraMethod {
        #[label("this method is not required by")]
        method_span: SourceSpan,
        #[label("this api")]
        api_span: SourceSpan,
        #[label("on this type")]
        require_span: SourceSpan,
    },
    #[error("Superapi not implemented")]
    #[diagnostic(help("implemented the required api"))]
    UnimplementedSuperApi {
        #[label("this api requires a superapi")]
        require_span: SourceSpan,
        #[label("this api requires a superapi")]
        api_span: SourceSpan,
        #[label("this api requires a superapi")]
        super_api: SourceSpan,
    },
}
