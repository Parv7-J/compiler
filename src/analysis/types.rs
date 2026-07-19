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
    pub ids: HashMap<DeclarationKey, DeclarationId>,
    pub db: Vec<Vec<DeclarationInfo>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationKey {
    scope_id: ScopeId,
    ident_id: IdentId,
}

impl DeclarationKey {
    pub fn new(scope: ScopeId, ident_id: IdentId) -> Self {
        Self {
            scope_id: scope,
            ident_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeclarationInfo {
    pub span: Span,
    pub ty: DeclarationType,
}

#[derive(Debug, Clone)]
pub enum DeclarationType {
    PendingType,
    PendingNonType,
    Packing(Vec<SymbolId>),
    Aor,
    Procedure,
    Methods,
    Api,
    Require,
    Get,
}

//only one declaration for each span -> thus we can store the span directly inside DeclarationInfo

pub enum IsTy {
    Yes,
    No,
}

impl DeclarationStore {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            db: Vec::new(),
        }
    }

    ///inserts the IdentId inside the store, returning it or
    ///panics because we have two duplicate interns, and thus two duplicate named declarations, even though
    ///they may be of different type
    pub fn insert(&mut self, key: DeclarationKey, span: Span, is_ty: IsTy) -> DeclarationId {
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                let declaration_info = match is_ty {
                    IsTy::Yes => DeclarationInfo {
                        span,
                        ty: DeclarationType::PendingType,
                    },
                    IsTy::No => DeclarationInfo {
                        span,
                        ty: DeclarationType::PendingNonType,
                    },
                };
                self.db[occupied_entry.get().0].push(declaration_info);
                *occupied_entry.get()
            }
            Entry::Vacant(vacant_entry) => {
                let idx = DeclarationId(self.db.len());
                vacant_entry.insert(idx);
                let declaration_info = match is_ty {
                    IsTy::Yes => DeclarationInfo {
                        span,
                        ty: DeclarationType::PendingType,
                    },
                    IsTy::No => DeclarationInfo {
                        span,
                        ty: DeclarationType::PendingNonType,
                    },
                };
                self.db.push(vec![declaration_info]);
                idx
            }
        }
    }

    ///gets the DeclarationId from IdentId, or else None
    pub fn getid(&self, key: DeclarationKey) -> Option<DeclarationId> {
        self.ids.get(&key).copied()
    }

    pub fn value(&self, id: DeclarationId) -> Option<&Vec<DeclarationInfo>> {
        self.db.get(id.0)
    }

    pub fn value_mut(&mut self, id: DeclarationId) -> Option<&mut Vec<DeclarationInfo>> {
        self.db.get_mut(id.0)
    }

    ///similar to get on hashmap
    pub fn getinfo(&self, id: DeclarationId, idx: usize) -> Option<&DeclarationInfo> {
        self.db.get(id.0).and_then(|v| v.get(idx))
    }

    ///similar to get_mut on hashmap
    pub fn getmutinfo(&mut self, id: DeclarationId, idx: usize) -> Option<&mut DeclarationInfo> {
        self.db.get_mut(id.0).and_then(|v| v.get_mut(idx))
    }
}

#[derive(Debug, Clone)]
pub struct SymbolStore {
    pub ids: HashMap<SymbolKey, SymbolId>,
    pub db: Vec<Vec<SymbolInfo>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolKey {
    scope_id: ScopeId,
    declaration_id: DeclarationId,
    ident_id: IdentId,
}
impl SymbolKey {
    pub fn new(scope: ScopeId, dec_id: DeclarationId, ident_id: IdentId) -> Self {
        Self {
            scope_id: scope,
            declaration_id: dec_id,
            ident_id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymbolInfo {
    pub span: Span,
    pub ty: SymbolType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    Pending,
    Variant,
    BuiltInType(SpannedTy),
    UserDefinedType(DeclarationId),
}

impl SymbolStore {
    pub fn new() -> Self {
        Self {
            ids: HashMap::new(),
            db: Vec::new(),
        }
    }

    //this is wrong, as we can allow dup ids

    ///inserts the IdentId inside the store, returning it or
    ///panics because we have two duplicate interns, and thus two duplicate named ids, even though
    ///they may be of different type
    pub fn insert(&mut self, key: SymbolKey, span: Span) -> SymbolId {
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                self.db[occupied_entry.get().0].push(SymbolInfo {
                    span,
                    ty: SymbolType::Pending,
                });
                *occupied_entry.get()
            }
            Entry::Vacant(vacant_entry) => {
                let idx = SymbolId(self.db.len());
                vacant_entry.insert(idx);
                self.db.push(vec![SymbolInfo {
                    span,
                    ty: SymbolType::Pending,
                }]);
                idx
            }
        }
    }

    pub fn value(&self, id: SymbolId) -> Option<&Vec<SymbolInfo>> {
        self.db.get(id.0)
    }

    pub fn value_mut(&mut self, id: SymbolId) -> Option<&mut Vec<SymbolInfo>> {
        self.db.get_mut(id.0)
    }

    pub fn getid(&self, key: SymbolKey) -> Option<SymbolId> {
        self.ids.get(&key).copied()
    }

    ///similar to get on hashmap
    pub fn getinfo(&self, id: SymbolId, idx: usize) -> Option<&SymbolInfo> {
        self.db.get(id.0).and_then(|v| v.get(idx))
    }

    ///similar to get_mut on hashmap
    pub fn getmutinfo(&mut self, id: SymbolId, idx: usize) -> Option<&mut SymbolInfo> {
        self.db.get_mut(id.0).and_then(|v| v.get_mut(idx))
    }
}
