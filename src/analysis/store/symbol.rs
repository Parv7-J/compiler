use super::{DeclarationId, IdentId, ScopeId};
use std::collections::{HashMap, hash_map::Entry};
use std::ops::Deref;

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

impl Deref for SymbolId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolKey {
    parent_scope_id: ScopeId,
    ident_id: IdentId,
}

impl SymbolKey {
    pub fn new(parent_scope_id: ScopeId, ident_id: IdentId) -> Self {
        Self {
            parent_scope_id,
            ident_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    pub at: usize,
    pub info: Vec<SymbolInfo>,
}

impl SymbolEntry {
    pub fn new(info: SymbolInfo) -> Self {
        Self {
            at: 0,
            info: vec![info],
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
                self.db[occupied_entry.get().0].info.push(SymbolInfo {
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
    pub fn first_declaration(&self, id: SymbolId) -> &SymbolInfo {
        &self.db[id.0].info[0]
    }

    ///gets the SymbolId from SymbolKey, or else None
    pub fn get_sid(&self, key: SymbolKey) -> Option<SymbolId> {
        self.ids.get(&key).copied()
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///should only be called once for every symbol redefinition(i.e. that have the same symbol id
    ///but are diff definitions)
    ///updates the internal state so that the next call to this would report the next definition->
    ///in order to initialize one by one
    ///panics if there is no entry related to the symbol id
    pub fn get_sinfo(&mut self, id: SymbolId) -> &mut SymbolInfo {
        let entry = &mut self.db[id.0];
        //SAFETY: starts at 0, and thus always starts at 1, and 1 - 1 = 0
        //OVERFLOW: can overflow, if we change the type for memory efficiency
        entry.at += 1;
        &mut entry.info[entry.at - 1]
    }
}
