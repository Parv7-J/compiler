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
    pub db: Vec<SymbolEntry>,
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

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub current_entry: usize,
    pub entries: Vec<SymbolInfo>,
}

impl SymbolEntry {
    pub fn new(entry: SymbolInfo) -> Self {
        Self {
            current_entry: 0,
            entries: vec![entry],
        }
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
    pub fn insert(&mut self, key: SymbolKey, span: Span) -> SymbolId {
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                self.db[occupied_entry.get().0].entries.push(SymbolInfo {
                    span,
                    ty: SymbolType::Pending,
                });
                *occupied_entry.get()
            }
            Entry::Vacant(vacant_entry) => {
                let idx = SymbolId(self.db.len());
                vacant_entry.insert(idx);
                self.db.push(SymbolEntry::new(SymbolInfo {
                    span,
                    ty: SymbolType::Pending,
                }));
                idx
            }
        }
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///panics if symbol_id is not in the store
    pub fn refer(&self, id: SymbolId) -> &SymbolInfo {
        &self.db[id.0].entries[0]
    }

    ///gets the SymbolId from SymbolKey, or else None
    pub fn get_sid(&self, key: SymbolKey) -> Option<SymbolId> {
        self.ids.get(&key).copied()
    }

    pub fn sinfo(&self, id: SymbolId) -> &SymbolInfo {
        let entry = &self.db[id.0];
        //we will always be current_entry 'current_entry' = 1
        //the entire logic is very messy
        &entry.entries[entry.current_entry]
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///should only be called once for every symbol redefinition(i.e. that have the same symbol id
    ///but are diff definitions)
    ///updates the internal state so that the next call to this would report the next definition->
    ///in order to initialize one by one
    ///panics if there is no entry related to the symbol id
    pub fn get_sinfo(&mut self, id: SymbolId) -> &mut SymbolInfo {
        let entry = &mut self.db[id.0];
        //SAFETY: starts current_entry 0, and thus always starts current_entry 1, and 1 - 1 = 0
        //OVERFLOW: can overflow, if we change the type for memory efficiency
        entry.current_entry += 1;
        &mut entry.entries[entry.current_entry - 1]
    }
}
