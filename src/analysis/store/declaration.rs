use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::ops::Deref;

use super::SymbolType;
use super::scope::ScopeStore;
use super::{IdentId, ScopeId};
use crate::analysis::SymbolId;
use crate::lexer::token::Span;
use crate::parser::ast::Item;

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
    pub scope_store: ScopeStore,
    pub ids: HashMap<DeclarationKey, DeclarationId>,
    pub db: Vec<DeclarationEntry>,
}

impl DeclarationStore {
    pub fn new() -> Self {
        Self {
            scope_store: ScopeStore::new(),
            ids: HashMap::new(),
            db: Vec::new(),
        }
    }

    ///inserts the IdentId inside the store, returning it or
    ///panics because we have two duplicate interns, and thus two duplicate named declarations, even though
    ///they may be of different type
    pub fn insert(&mut self, key: DeclarationKey, span: Span, item: &Item) -> DeclarationId {
        let ty = match item {
            Item::Packing(_) => DeclarationType::Packing(None),
            Item::Aor(_) => DeclarationType::Aor(None),
            Item::Procedure(_) => DeclarationType::Procedure(None),
            Item::Api(_) => DeclarationType::Api(None),
            _ => unreachable!("insert should only be called on types, functions, and interfaces"),
        };
        let scope_id = self.scope_store.new_scope(key.scope_id);
        let d_info = DeclarationInfo { scope_id, span, ty };
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                let did = *occupied_entry.get();
                self.db[did.0].info.push(d_info);
                did
            }
            Entry::Vacant(vacant_entry) => {
                let did = DeclarationId(self.db.len());
                vacant_entry.insert(did);
                self.db.push(DeclarationEntry::new(d_info));
                did
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

    ///gets the declaration info of the declaration that is going to be instantiated
    ///panics if 'did' doesnt correspond to a valid entry
    pub fn dinfo(&self, id: DeclarationId) -> &DeclarationInfo {
        let entry = &self.db[id.0];
        &entry.info[entry.at]
    }

    ///calls scope_store.new_scope
    pub fn new_scope(&mut self, parent: ScopeId) -> ScopeId {
        self.scope_store.new_scope(parent)
    }

    ///calls scope_store.parent_scope
    pub fn parent_scope(&self, scope: ScopeId) -> ScopeId {
        self.scope_store.parent_scope(scope)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationKey {
    scope_id: ScopeId,
    ident_id: IdentId,
}

impl DeclarationKey {
    pub fn new(scope_id: ScopeId, ident_id: IdentId) -> Self {
        Self { scope_id, ident_id }
    }
}

#[derive(Debug, Clone)]
pub struct DeclarationEntry {
    pub at: usize,
    pub info: Vec<DeclarationInfo>,
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
    pub scope_id: ScopeId,
    pub span: Span,
    pub ty: DeclarationType,
}

#[derive(Debug, Clone)]
pub enum DeclarationType {
    Packing(Option<Packing>),
    Aor(Option<Aor>),
    Procedure(Option<Procedure>),
    Api(Option<Api>),
}

#[derive(Debug, Clone)]
pub struct Packing {
    pub fields: HashMap<IdentId, SymbolId>,
    pub methods: Option<Vec<HashMap<IdentId, DeclarationId>>>,
    pub requires: Option<HashMap<DeclarationId, Vec<HashMap<IdentId, DeclarationId>>>>,
}

#[derive(Debug, Clone)]
pub struct Aor {
    pub variants: HashMap<IdentId, SymbolId>,
    pub methods: Option<Vec<HashMap<IdentId, DeclarationId>>>,
    pub requires: Option<HashMap<DeclarationId, Vec<HashMap<IdentId, DeclarationId>>>>,
}

#[derive(Debug, Clone)]
pub struct Api {
    pub supers: Vec<DeclarationId>,
    pub methods: HashMap<IdentId, DeclarationId>,
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub arguments: HashMap<IdentId, SymbolId>,
    pub return_ty: Option<SymbolType>,
}
