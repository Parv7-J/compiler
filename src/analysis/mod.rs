#![allow(unused)]
use crate::{lexer::token::Span, parser::ast::*};

mod types;
use types::*;

pub struct AstAnalyzer<'a> {
    ast: Ast<'a>,
    analyzer: Analyzer<'a>,
}

impl<'a> AstAnalyzer<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        let input = ast.input;
        Self {
            ast,
            analyzer: Analyzer {
                intern: Intern::new(input),
                item_store: ItemStore::new(),
                symbol_store: SymbolStore::new(),
            },
        }
    }

    pub fn analyze(mut self) {
        self.collect_top_level_definitions();
        self.register_items();
        println!("Intern: {:?}", self.analyzer.intern);
        println!("ItemStore: {:?}", self.analyzer.item_store);
        println!("SymbolStore: {:?}", self.analyzer.symbol_store);
    }

    pub fn collect_top_level_definitions(&mut self) {
        for item in &self.ast.items {
            let intern_id: InternId = self.analyzer.intern.insert(item.get_ident_span());
            let isty = if matches!(item, Item::Packing(_) | Item::Aor(_)) {
                IsTy::Yes
            } else {
                IsTy::No
            };
            self.analyzer.item_store.insert(intern_id, isty);
        }
    }

    pub fn register_items(&mut self) {
        for item in &self.ast.items {
            match item {
                Item::Packing(packing) => self.analyzer.register_packing(packing),
                Item::Aor(aor) => todo!(),
                Item::Procedure(procedure) => todo!(),
                Item::Methods(methods) => todo!(),
                Item::Api(api) => todo!(),
                Item::Require(require) => todo!(),
                Item::Get(get) => todo!(),
            };
        }
    }
}

#[derive(Debug, Clone)]
pub struct Analyzer<'a> {
    intern: Intern<'a>,
    item_store: ItemStore,
    symbol_store: SymbolStore,
}

impl Analyzer<'_> {
    fn register_packing(&mut self, packing: &Packing) -> (ItemId, Span) {
        let packing_span = packing.ident.0;
        let intern_id = self
            .intern
            .contains(packing_span)
            .expect("item must be inserted");
        let packing_id = self
            .item_store
            .getid(intern_id)
            .expect("item must be inserted");

        let mut symbols = Vec::new();
        for field in &packing.fields {
            symbols.push(self.register_field(field));
        }

        let packing_info = self
            .item_store
            .getmutinfo(packing_id)
            .expect("item must be inserted");

        *packing_info = ItemInfo::Packing(symbols);

        (packing_id, packing_span)
    }

    ///interpretation matters, as an intern can be both symbol as well as item
    fn register_field(&mut self, field: &Field) -> (SymbolId, Span) {
        let field_span = field.ident.0;
        let intern_id = self.intern.insert(field_span);
        let symbol_id = self.symbol_store.insert(intern_id);

        let (new_symbol_info, symbol_span) = match &field.ty {
            IdentTy::Type(spanned_ty) => (SymbolInfo::BuiltIn(spanned_ty.ty), spanned_ty.span),
            IdentTy::Ident(spanned_ident) => {
                let type_span = spanned_ident.0;
                let intern_id = self.intern.contains(type_span).expect("undefined type");
                let item_id = self.item_store.getid(intern_id).expect("not a type");
                let item_info = self.item_store.getmutinfo(item_id).unwrap();

                if !matches!(*item_info, ItemInfo::PendingType) {
                    panic!("an item but not a type");
                }

                (SymbolInfo::Type(item_id), type_span)
            }
            IdentTy::Arr(spanned_arr) => todo!(),
            IdentTy::Ptr(spanned_ptr) => todo!(),
        };

        let symbol_info = self
            .symbol_store
            .getmutinfo(symbol_id)
            .expect("just manufactured the id");

        *symbol_info = new_symbol_info;
        (symbol_id, symbol_span)
    }

    // fn intern_to_item(&mut self, span: Span) -> Option<&mut ItemInfo> {
    //     let intern_id = self.intern.insert(span);
    //     self.item_store.get(intern_id)
    // }
}
