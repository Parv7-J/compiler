mod declaration;
mod ident;
mod symbol;

pub use declaration::*;
pub use ident::*;
pub use symbol::*;

///identifies a sequence of letters, and thus 'foo' appearing in any place would have the same
///IdentId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentId(pub usize);

///uniquely identifies a declaration, in any scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(pub usize);

///uniquely identifies a symbol, in any declaration and any scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

///each level of depth has the same scope id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);
