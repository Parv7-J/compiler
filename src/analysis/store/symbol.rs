use super::{DeclarationId, IdentId, ScopeId};
use std::collections::{HashMap, hash_map::Entry};

use crate::lexer::token::Span;
use crate::parser::ast::ArrType;
use crate::parser::ast::SpannedTy;

///uniquely identifies a symbol, in any declaration and any scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone)]
pub struct SymbolStore {
    pub ids: HashMap<SymbolKey, SymbolId>,
    pub db: Vec<Vec<SymbolInfo>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolKey {
    scope_id: ScopeId,
    ident_id: IdentId,
}

impl SymbolKey {
    pub fn new(scope_id: ScopeId, ident_id: IdentId) -> Self {
        Self { scope_id, ident_id }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SymbolInfo {
    pub span: Span,
    pub ty: SymbolType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolType {
    Pending,
    Variant,
    BuiltInType(SpannedTy),
    UserDefinedType(DeclarationId),
    ArrType {
        arr_ty: ArrType,
        span: Span,
        inner_ty: Box<SymbolType>,
    },
    PtrType {
        ty: Box<SymbolType>,
        span: Span,
    },
    Error {
        span: Span,
    },
}

impl SymbolStore {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            db: Vec::new(),
        }
    }

    ///inserts the IdentId inside the store, returning it or
    ///panics because we have two duplicate interns, and thus two duplicate named ids, even though
    ///they may be of different type
    pub fn insert(&mut self, key: SymbolKey, span: Span, symbol_type: SymbolType) -> SymbolId {
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                self.db[occupied_entry.get().0].push(SymbolInfo {
                    span,
                    ty: symbol_type,
                });
                *occupied_entry.get()
            }
            Entry::Vacant(vacant_entry) => {
                let idx = SymbolId(self.db.len());
                vacant_entry.insert(idx);
                self.db.push(vec![SymbolInfo {
                    span,
                    ty: symbol_type,
                }]);
                idx
            }
        }
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///panics if symbol_id is not in the store
    pub fn refer(&self, id: SymbolId) -> &SymbolInfo {
        &self.db[id.0][0]
    }

    ///gets the SymbolId from SymbolKey, or else None
    pub fn get(&self, key: SymbolKey) -> Option<SymbolId> {
        self.ids.get(&key).copied()
    }

    pub fn mutable_info(&mut self, sid: SymbolId) -> &mut SymbolInfo {
        let v = &mut self.db[sid.0];
        debug_assert!(!v.is_empty());
        let idx = v.len() - 1;
        &mut v[idx]
    }
}
