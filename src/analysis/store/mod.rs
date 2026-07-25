pub mod declaration;
pub mod ident;
pub mod scope;
pub mod symbol;

pub use declaration::{DeclarationId, DeclarationStore};
pub use ident::{IdentId, IdentStore};
pub use scope::ScopeId;
pub use symbol::{SymbolId, SymbolKey, SymbolStore, SymbolType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    scid: ScopeId,
    iid: IdentId,
}

impl Key {
    pub fn new(scope_id: ScopeId, ident_id: IdentId) -> Self {
        Self {
            scid: scope_id,
            iid: ident_id,
        }
    }
}
