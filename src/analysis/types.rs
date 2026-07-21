use std::collections::{HashMap, hash_map::Entry};

use crate::lexer::token::*;
use crate::parser::ast::*;

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

// ///we can have multiple declarations for the same scope level ->
// ///fn foo() {
// ///     {
// ///         let a = 1;
// ///     }
// ///     {
// ///         let a = String::new();
// ///     }
// /// }
// #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
// pub struct RedeclarationId(pub usize);

//when we enter a new scope again, we add one to the redec id
//or if we try to enter again
#[derive(Debug, Clone)]
pub struct IdentStore<'a> {
    pub input: &'a str,
    pub db: Vec<String>,
    pub ids: HashMap<String, IdentId>,
}

impl<'a> IdentStore<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            db: Vec::new(),
            ids: HashMap::new(),
        }
    }

    ///inserts the string referenced by the '@arg span' inside self.input, inside the IdentStore,
    ///returning it, or
    ///returns the corresponding id relating to the string spanned, if entry already exists
    pub fn insert(&mut self, span: Span) -> IdentId {
        let Span { start, end } = span;
        let ident = &self.input[start as usize..end as usize];
        if self.ids.contains_key(ident) {
            return self.ids.get(ident).copied().unwrap();
        }
        let pos = IdentId(self.db.len());
        self.db.push(ident.to_string());
        self.ids.insert(ident.to_string(), pos);
        pos
    }

    ///checks if the string pointed by the span inside self.input is interned, returning the
    ///IdentId if yes, or None. Diff from insert in that it doesnt insert the string
    pub fn contains(&self, span: Span) -> Option<IdentId> {
        let Span { start, end } = span;
        let ident = &self.input[start as usize..end as usize];
        self.ids.get(ident).copied()
    }

    ///gives back the string corresponding to the IdentId, or None if id not present
    pub fn get(&self, id: IdentId) -> Option<&String> {
        self.db.get(id.0)
    }
}

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

#[derive(Debug, Clone)]
pub struct SymbolStore {
    pub ids: HashMap<SymbolKey, SymbolId>,
    pub db: Vec<SymbolEntry>,
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
        dbg!(entry.at);
        &mut entry.info[entry.at - 1]
    }
}
