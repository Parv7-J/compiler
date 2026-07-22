use std::ops::Deref;

///each level of depth has the same scope id
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(pub usize);

impl Deref for ScopeId {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct ScopeStore {
    parents: Vec<ScopeId>,
}

impl ScopeStore {
    pub fn new() -> Self {
        Self {
            parents: vec![ScopeId(0)],
        }
    }

    pub fn new_scope(&mut self, parent: ScopeId) -> ScopeId {
        let len = self.parents.len();
        self.parents.push(parent);
        ScopeId(len)
    }

    pub fn parent_scope(&self, scope: ScopeId) -> ScopeId {
        self.parents[scope.0]
    }
}
