use std::collections::HashMap;

use crate::Span;

///identifies a sequence of letters, and thus 'foo' appearing in any place would have the same
///IdentId
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentId(pub usize);

#[derive(Debug, Clone)]
pub struct IdentStore<'a> {
    input: &'a str,
    db: Vec<String>,
    ids: HashMap<String, IdentId>,
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
