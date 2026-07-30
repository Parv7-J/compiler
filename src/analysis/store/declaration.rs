use std::collections::HashMap;
use std::collections::hash_map::Entry;

use super::SymbolType;
use super::scope::ScopeStore;
use super::{IdentId, ScopeId};
use crate::analysis::SymbolId;
use crate::analysis::store::Key;
use crate::lexer::token::Span;

///uniquely identifies a declaration, in any scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DeclarationId(pub usize);

#[derive(Debug, Clone)]
pub struct DeclarationEntry {
    pub current_entry: usize,
    pub entries: Vec<DeclarationInfo>,
}

impl DeclarationEntry {
    pub fn new(entry: DeclarationInfo) -> Self {
        Self {
            current_entry: 0,
            entries: vec![entry],
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeclarationInfo {
    ///'scope_id': scope owned by the declaration
    pub scope_id: ScopeId,
    ///'span': span of the declaration's identifier
    pub span: Span,
    ///'ty': type of declaration
    pub ty: DeclarationType,
}

#[derive(Debug, Clone)]
pub struct DeclarationStore {
    pub scope_store: ScopeStore,
    pub ids: HashMap<Key, DeclarationId>,
    pub db: Vec<DeclarationEntry>,
    pub unknown: HashMap<Key, UnknownEntry>,
}

impl DeclarationStore {
    pub fn new() -> Self {
        Self {
            scope_store: ScopeStore::new(),
            ids: HashMap::new(),
            db: Vec::new(),
            unknown: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: Key, span: Span, ty: DeclarationType) -> DeclarationId {
        let new_scope = self.new_scope(key.scid);
        let d_info = DeclarationInfo {
            scope_id: new_scope,
            span,
            ty,
        };
        match self.ids.entry(key) {
            Entry::Occupied(occupied_entry) => {
                let did = *occupied_entry.get();
                self.db[did.0].entries.push(d_info);
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

    ///DeclarationKey -> DeclarationId
    pub fn get(&self, key: Key) -> Option<DeclarationId> {
        self.ids.get(&key).copied()
    }

    ///DeclarationId -> Current Declaration's Scope
    pub fn scope_from_id(&self, did: DeclarationId) -> ScopeId {
        let dentry = &self.db[did.0];
        dentry.entries[dentry.current_entry].scope_id
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///should only be called once for every redeclaration (i.e. that have the same declaration id
    ///but are diff declarations)
    ///updates the internal state so that the next call to this would report the next declaration ->
    ///in order to initialize one by one
    ///panics if there is no entry related to the declaration id
    pub fn initialize<T>(&mut self, did: DeclarationId, initializer: T)
    where
        T: FnOnce(&mut DeclarationInfo),
    {
        let dentry = &mut self.db[did.0];
        let dinfo = &mut dentry.entries[dentry.current_entry];
        initializer(dinfo);
        dentry.current_entry += 1;
    }

    ///finds declarations, searching from the 'scope' defined in 'key', and goes up the parent
    ///chain, stopping when either the declaration is found, or we reach the outermost scope
    pub fn find(&self, starting_scope: ScopeId, iid: IdentId) -> Option<DeclarationId> {
        let mut key = Key::new(starting_scope, iid);
        loop {
            let mut scope = key.scid;
            match self.get(key) {
                Some(did) => return Some(did),
                None => {
                    let parent_scope = self.parent_scope(scope);
                    if scope == parent_scope {
                        break;
                    }
                    scope = parent_scope;
                }
            };
            key = Key::new(scope, key.iid);
        }
        None
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///panics if declaration_id is not in the store
    pub fn refer(&self, id: DeclarationId) -> &DeclarationInfo {
        &self.db[id.0].entries[0]
    }

    ///should always be called when it is ensured that the id corresponds to an entry
    ///panics if declaration_id is not in the store
    pub fn refer_mut(&mut self, id: DeclarationId) -> &mut DeclarationInfo {
        &mut self.db[id.0].entries[0]
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

impl DeclarationStore {
    pub fn insert_unknown(&mut self, key: Key, span: Span, ty: UnknownType, owned_scope: ScopeId) {
        let u_info = UnknownInfo {
            scope_id: owned_scope,
            span,
            ty,
        };
        match self.unknown.entry(key) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().entries.push(u_info);
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(UnknownEntry::new(u_info));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnknownEntry {
    pub current_entry: usize,
    pub entries: Vec<UnknownInfo>,
}

impl UnknownEntry {
    pub fn new(entry: UnknownInfo) -> Self {
        Self {
            current_entry: 0,
            entries: vec![entry],
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnknownInfo {
    ///'scope_id': scope owned by the declaration
    pub scope_id: ScopeId,
    ///'span': span of the declaration's identifier
    pub span: Span,
    ///'ty': type of declaration
    pub ty: UnknownType,
}

#[derive(Debug, Clone)]
pub enum UnknownType {
    UnknownMethods(ScopedMethods),
    UnknowRequires(HashMap<DeclarationId, Vec<ScopedMethods>>),
}

#[derive(Debug, Clone)]
pub enum DeclarationType {
    Packing(Packing),
    Aor(Aor),
    Procedure(Procedure),
    Api(Api),
}

impl DeclarationType {
    pub fn packing() -> Self {
        Self::Packing(Packing {
            fields: HashMap::new(),
            methods: Vec::new(),
            requires: HashMap::new(),
        })
    }

    pub fn aor() -> Self {
        Self::Aor(Aor {
            variants: HashMap::new(),
            methods: Vec::new(),
            requires: HashMap::new(),
        })
    }
    pub fn api() -> Self {
        Self::Api(Api {
            supers: Vec::new(),
            methods: HashMap::new(),
        })
    }

    pub fn procedure() -> Self {
        Self::Procedure(Procedure {
            arguments: HashMap::new(),
            return_ty: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ScopedMethods {
    pub scope: ScopeId,
    pub methods: HashMap<IdentId, DeclarationId>,
}

#[derive(Debug, Clone)]
pub struct Packing {
    pub fields: HashMap<IdentId, SymbolId>,
    pub methods: Vec<ScopedMethods>,
    pub requires: HashMap<DeclarationId, Vec<ScopedMethods>>,
}

impl Packing {
    pub fn set_fields(&mut self, fields: HashMap<IdentId, SymbolId>) {
        self.fields = fields;
    }

    pub fn add_methods(&mut self, methods: HashMap<IdentId, DeclarationId>, scope: ScopeId) {
        self.methods.push(ScopedMethods { scope, methods });
    }

    pub fn add_requires(
        &mut self,
        api: DeclarationId,
        requires: HashMap<IdentId, DeclarationId>,
        scope: ScopeId,
    ) {
        match self.requires.entry(api) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().push(ScopedMethods {
                    scope,
                    methods: requires,
                });
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![ScopedMethods {
                    scope,
                    methods: requires,
                }]);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Aor {
    pub variants: HashMap<IdentId, SymbolId>,
    pub methods: Vec<ScopedMethods>,
    pub requires: HashMap<DeclarationId, Vec<ScopedMethods>>,
}

impl Aor {
    pub fn set_variants(&mut self, variants: HashMap<IdentId, SymbolId>) {
        self.variants = variants;
    }

    pub fn add_methods(&mut self, methods: HashMap<IdentId, DeclarationId>, scope: ScopeId) {
        self.methods.push(ScopedMethods { scope, methods });
    }

    pub fn add_requires(
        &mut self,
        api: DeclarationId,
        requires: HashMap<IdentId, DeclarationId>,
        scope: ScopeId,
    ) {
        match self.requires.entry(api) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().push(ScopedMethods {
                    scope,
                    methods: requires,
                });
            }
            Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(vec![ScopedMethods {
                    scope,
                    methods: requires,
                }]);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Api {
    pub supers: Vec<DeclarationId>,
    pub methods: HashMap<IdentId, DeclarationId>,
}

impl Api {
    pub fn set_supers(&mut self, supers: Vec<DeclarationId>) {
        self.supers = supers;
    }

    pub fn set_methods(&mut self, methods: HashMap<IdentId, DeclarationId>) {
        self.methods = methods;
    }
}
impl Procedure {
    pub fn set_arguments(&mut self, arguments: HashMap<IdentId, SymbolId>) {
        self.arguments = arguments;
    }

    pub fn set_return_ty(&mut self, return_ty: Option<SymbolType>) {
        self.return_ty = return_ty;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    pub arguments: HashMap<IdentId, SymbolId>,
    pub return_ty: Option<SymbolType>,
}
