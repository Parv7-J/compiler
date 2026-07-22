use crate::analysis::SymbolId;
use crate::analysis::SymbolType;
use crate::lexer::token::Span;

use super::{DeclarationId, IdentId, ScopeId};
use std::collections::HashMap;
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
