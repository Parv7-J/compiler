pub mod declaration;
pub mod ident;
pub mod scope;
pub mod symbol;

pub use declaration::{DeclarationId, DeclarationStore};
pub use ident::{IdentId, IdentStore};
pub use scope::ScopeId;
pub use symbol::{SymbolId, SymbolKey, SymbolStore, SymbolType};
