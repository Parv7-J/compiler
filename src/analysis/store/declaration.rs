use super::scope::ScopeStore;
use crate::analysis::SymbolId;
use crate::lexer::token::Span;
use crate::parser::ast::Item;

use super::{IdentId, ScopeId};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::Deref;

///uniquely identifies a declaration, in any scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(pub usize);

impl Deref for DeclarationId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct DeclarationStore {
    pub scopes: ScopeStore,
    pub ids: HashMap<DeclarationKey, DeclarationId>,
    pub db: Vec<DeclarationEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationKey {
    parent_scope_id: ScopeId,
    ident_id: IdentId,
}

impl DeclarationKey {
    pub fn new(parent_scope_id: ScopeId, ident_id: IdentId) -> Self {
        Self {
            parent_scope_id,
            ident_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeclarationEntry {
    pub at: usize,
    pub info: Vec<DeclarationInfo>,
}

#[derive(Debug, Clone)]
pub struct TypeEntry {
    pub ty: Vec<DeclarationInfo>,
    pub methods: Vec<DeclarationInfo>,
    pub require: Vec<DeclarationInfo>,
}

#[derive(Debug, Clone)]
pub struct Ent {
    pub at: usize,
    pub info: Vec<DeclarationInfo>,
}

pub struct PackingDeclaration {
    pub at: usize,
    pub types: Vec<HashMap<IdentId, SymbolId>>,
}

pub struct AorDeclarations {
    pub at: usize,
    pub declarations: Vec<HashMap<IdentId, SymbolId>>,
}

impl DeclarationEntry {
    pub fn new(info: DeclarationInfo) -> Self {
        Self {
            at: 0,
            info: vec![info],
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeclarationInfo {
    pub scope: ScopeId,
    pub span: Span,
    pub ty: DeclarationType,
}

struct Packing {
    fields: Option<HashMap<IdentId, SymbolId>>,
    methods: Option<Vec<HashMap<IdentId, DeclarationId>>>,
    requires: Option<HashMap<DeclarationId, Vec<HashMap<IdentId, DeclarationId>>>>,
}

struct Aor {
    variants: Option<HashMap<IdentId, SymbolId>>,
    methods: Option<Vec<HashMap<IdentId, DeclarationId>>>,
    requires: Option<HashMap<DeclarationId, Vec<HashMap<IdentId, DeclarationId>>>>,
}

struct Api {
    supers: Option<Vec<DeclarationId>>,
    methods: Option<HashMap<IdentId, DeclarationId>>,
}

impl DeclarationStore {
    pub fn new() -> Self {
        Self {
            scopes: Scopes::new(),
            ids: HashMap::new(),
            db: Vec::new(),
        }
    }

    ///inserts the IdentId inside the store, returning it or
    ///panics because we have two duplicate interns, and thus two duplicate named declarations, even though
    ///they may be of different type
    pub fn insert(&mut self, key: DeclarationKey, span: Span, item: &Item) -> DeclarationId {
        //we should leave the collection of require and methods, and treat them in the next stage
        let ty = match item {
            Item::Packing(_) => DeclarationType::Pending(Pending::Packing),
            Item::Aor(_) => DeclarationType::Pending(Pending::Aor),
            Item::Procedure(_) => DeclarationType::Pending(Pending::Procedure),
            Item::Api(_) => DeclarationType::Pending(Pending::Api),
            _ => unimplemented!(),
        };
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                let new_scope = self.scopes.add_scope(key.parent_scope_id);
                let declaration_info = DeclarationInfo {
                    scope: new_scope,
                    span,
                    ty,
                };
                self.db[occupied_entry.get().0].info.push(declaration_info);
                *occupied_entry.get()
            }
            Entry::Vacant(vacant_entry) => {
                let idx = DeclarationId(self.db.len());
                vacant_entry.insert(idx);
                let new_scope = self.scopes.add_scope(key.parent_scope_id);
                let declaration_info = DeclarationInfo {
                    scope: new_scope,
                    span,
                    ty,
                };
                self.db.push(DeclarationEntry::new(declaration_info));
                idx
            }
        }
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///panics if declaration_id is not in the store
    pub fn first_declaration(&self, id: DeclarationId) -> &DeclarationInfo {
        &self.db[id.0].info[0]
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///panics if declaration_id is not in the store
    pub fn first_declaration_mut(&mut self, id: DeclarationId) -> &mut DeclarationInfo {
        &mut self.db[id.0].info[0]
    }

    ///gets the DeclarationId from DeclarationKey, or else None
    pub fn get_did(&self, key: DeclarationKey) -> Option<DeclarationId> {
        self.ids.get(&key).copied()
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///should only be called once for every redeclaration (i.e. that have the same declaration id
    ///but are diff declarations)
    ///updates the internal state so that the next call to this would report the next declaration ->
    ///in order to initialize one by one
    ///panics if there is no entry related to the declaration id
    pub fn get_dinfo(&mut self, id: DeclarationId) -> &mut DeclarationInfo {
        let entry = &mut self.db[id.0];
        //SAFETY: starts at 0, and thus always starts at 1, and 1 - 1 = 0
        //OVERFLOW: can overflow, if we change the type for memory efficiency
        entry.at += 1;
        &mut entry.info[entry.at - 1]
    }

    pub fn dinfo(&self, id: DeclarationId) -> &DeclarationInfo {
        let entry = &self.db[id.0];
        &entry.info[entry.at]
    }
}
