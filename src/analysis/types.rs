#![allow(unused)]
use std::collections::{HashMap, hash_map::Entry};

use crate::lexer::token::*;
use crate::parser::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InternId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

#[derive(Debug, Clone)]
pub struct Intern<'a> {
    pub db: Vec<String>,
    pub ids: HashMap<String, InternId>,
    pub input: &'a str,
}

impl<'a> Intern<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            db: Vec::new(),
            ids: HashMap::new(),
            input,
        }
    }

    ///inserts the string referenced by the '@arg span' inside self.input, inside the Intern,
    ///returning it, or
    ///returns the corresponding id relating to the string spanned, if entry already exists
    pub fn insert(&mut self, span: Span) -> InternId {
        let Span { start, end } = span;
        let ident = &self.input[start as usize..end as usize];
        if self.ids.contains_key(ident) {
            return self.ids.get(ident).copied().unwrap();
        }
        let pos = InternId(self.db.len());
        self.db.push(ident.to_string());
        self.ids.insert(ident.to_string(), pos);
        pos
    }

    ///checks if the string pointed by the span inside self.input is interned, returning the
    ///InternId if yes, or None. Diff from insert in that it doesnt insert the string
    pub fn contains(&self, span: Span) -> Option<InternId> {
        let Span { start, end } = span;
        let ident = &self.input[start as usize..end as usize];
        self.ids.get(ident).copied()
    }

    ///gives back the string corresponding to the InternId, or None if id not present
    pub fn get(&self, id: InternId) -> Option<&String> {
        self.db.get(id.0)
    }
}

#[derive(Debug, Clone)]
pub struct ItemStore {
    pub items: HashMap<InternId, ItemId>,
    pub item_info: Vec<ItemInfo>,
}

#[derive(Debug, Clone)]
pub enum ItemInfo {
    PendingType,
    PendingNonType,
    Packing(Vec<(SymbolId, Span)>),
    Aor,
    Procedure,
    Methods,
    Api,
    Require,
    Get,
}

pub enum IsTy {
    Yes,
    No,
}

impl ItemStore {
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
            item_info: Vec::new(),
        }
    }

    ///inserts the InternId inside the store, returning it or
    ///panics because we have two duplicate interns, and thus two duplicate named items, even though
    ///they may be of different type
    pub fn insert(&mut self, id: InternId, ty: IsTy) -> ItemId {
        match self.items.entry(id) {
            Entry::Occupied(occupied_entry) => {
                panic!("repeated declaration")
            }
            Entry::Vacant(vacant_entry) => {
                let idx = ItemId(self.item_info.len());
                vacant_entry.insert(idx);
                let item_info = match ty {
                    IsTy::Yes => ItemInfo::PendingType,
                    IsTy::No => ItemInfo::PendingNonType,
                };
                self.item_info.push(item_info);
                idx
            }
        }
    }

    ///gets the ItemId from InternId, or else None
    pub fn getid(&self, id: InternId) -> Option<ItemId> {
        self.items.get(&id).copied()
    }

    ///similar to get on hashmap
    pub fn getinfo(&self, id: ItemId) -> Option<&ItemInfo> {
        self.item_info.get(id.0)
    }

    ///similar to get_mut on hashmap
    pub fn getmutinfo(&mut self, id: ItemId) -> Option<&mut ItemInfo> {
        self.item_info.get_mut(id.0)
    }
}

#[derive(Debug, Clone)]
pub struct SymbolStore {
    pub symbols: HashMap<InternId, SymbolId>,
    pub symbol_info: Vec<SymbolInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolInfo {
    Pending,
    BuiltIn(Ty),
    Type(ItemId),
}

impl SymbolStore {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            symbol_info: Vec::new(),
        }
    }

    //this is wrong, as we can allow dup symbols

    ///inserts the InternId inside the store, returning it or
    ///panics because we have two duplicate interns, and thus two duplicate named symbols, even though
    ///they may be of different type
    pub fn insert(&mut self, id: InternId) -> SymbolId {
        match self.symbols.entry(id) {
            Entry::Occupied(occupied_entry) => {
                panic!("repeated symbol")
            }
            Entry::Vacant(vacant_entry) => {
                let idx = SymbolId(self.symbol_info.len());
                vacant_entry.insert(idx);
                self.symbol_info.push(SymbolInfo::Pending);
                idx
            }
        }
    }

    ///gets the ItemId from InternId, or else None
    pub fn getid(&self, id: InternId) -> Option<SymbolId> {
        self.symbols.get(&id).copied()
    }

    ///similar to get on hashmap
    pub fn getinfo(&self, id: SymbolId) -> Option<&SymbolInfo> {
        self.symbol_info.get(id.0)
    }

    ///similar to get_mut on hashmap
    pub fn getmutinfo(&mut self, id: SymbolId) -> Option<&mut SymbolInfo> {
        self.symbol_info.get_mut(id.0)
    }
}
