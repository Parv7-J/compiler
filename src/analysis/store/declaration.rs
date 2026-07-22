use crate::analysis::SymbolId;
use crate::analysis::SymbolType;
use crate::lexer::token::Span;

use super::{DeclarationId, IdentId, ScopeId};
use std::collections::HashMap;
use std::collections::hash_map::Entry;

#[derive(Debug, Clone)]
pub struct DeclarationStore {
    pub scopes: Scopes,
    pub ids: HashMap<DeclarationKey, DeclarationId>,
    pub db: Vec<DeclarationEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationKey {
    parent_scope_id: ScopeId,
    ident_id: IdentId,
}

#[derive(Debug, Clone)]
pub struct Scopes {
    parents: Vec<ScopeId>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            parents: vec![ScopeId(0)],
        }
    }

    pub fn add_scope(&mut self, parent: ScopeId) -> ScopeId {
        let len = self.parents.len();
        self.parents.push(parent);
        ScopeId(len)
    }

    pub fn parent_scope(&self, scope: ScopeId) -> ScopeId {
        self.parents[scope.0]
    }
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

#[derive(Debug, Clone)]
pub enum DeclarationType {
    Pending,
    Resolved(Resolved),
    PendingType,
    ResolvedType(ResolvedType),
}

#[derive(Debug, Clone)]
pub enum Resolved {
    Procedure {
        arguments: HashMap<IdentId, SymbolId>,
        return_ty: Option<SymbolType>,
    },
    Methods,
    Api,
    Require,
    Get,
}

//foo.a
//get the type for foo -> Vec<SymbolId>,
//get the type for a -> i think we need an ident id to symbolid map here

#[derive(Debug, Clone)]
pub enum ResolvedType {
    Packing(HashMap<IdentId, SymbolId>),
    Aor(HashMap<IdentId, SymbolId>),
}

pub enum IsTy {
    Yes,
    No,
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
    pub fn insert(
        &mut self,
        key: DeclarationKey,
        parent_scope: ScopeId,
        span: Span,
        is_ty: IsTy,
    ) -> DeclarationId {
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                let new_scope = self.scopes.add_scope(parent_scope);
                let ty = match is_ty {
                    IsTy::Yes => DeclarationType::PendingType,
                    IsTy::No => DeclarationType::Pending,
                };
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
                let new_scope = self.scopes.add_scope(parent_scope);
                let ty = match is_ty {
                    IsTy::Yes => DeclarationType::PendingType,
                    IsTy::No => DeclarationType::Pending,
                };
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
