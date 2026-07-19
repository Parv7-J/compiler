#![allow(unused)]
use std::sync::Arc;

use crate::{lexer::token::Span, parser::ast::*};

mod error;
mod types;
use error::*;
use miette::Report;
use types::*;

pub struct AstAnalyzer<'a> {
    ast: Ast<'a>,
    analyzer: Analyzer<'a>,
    errors: Vec<Report>,
}

//packing Foo {a(i32), b(Bar)} -> no need for the span of 'packing', we need the span of 'Foo', we
//need the span of 'a' and 'b', and we also need the span of 'i32', 'Bar' -> empty () will be caught
//at parsetime, and any errors like undelimited ), or ident missing etc. etc. will be also caught at
//parse time. so we now dont need spans of anything else as they are not necessary. But do we need
//spans of keywords like i32 and Foo??? at parse time we remove the spans of things like delims,
//puncts, etc. but we do track the spans of ty using spannedty and of idents using spannedident, so
//lets keep the spans intact
//packing Bar {brr(u32)}

impl<'a> AstAnalyzer<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        let input = ast.input;
        Self {
            ast,
            analyzer: Analyzer {
                scope: ScopeId(0),
                idents: IdentStore::new(input),
                declarations: DeclarationStore::new(),
                symbols: SymbolStore::new(),
            },
            errors: Vec::new(),
        }
    }

    pub fn analyze(mut self) -> Analyzer<'a> {
        self.collect_top_level_definitions();

        let source = Arc::new(miette::NamedSource::new(
            "language",
            self.ast.input.to_string(),
        ));
        if !self.errors.is_empty() {
            eprintln!("Found {} semantic errors ->\n", self.errors.len());
        }
        for (no, report) in self.errors.into_iter().enumerate() {
            eprintln!(
                "Error {}:\n {:?}\n",
                no + 1,
                report.with_source_code(source.clone())
            );
        }

        self.analyzer
    }

    pub fn collect_top_level_definitions(&mut self) {
        for item in &self.ast.items {
            let ident_span = item.get_ident_span();
            let ident_id = self.analyzer.idents.insert(item.get_ident_span());
            let is_ty = if matches!(item, Item::Packing(_) | Item::Aor(_)) {
                IsTy::Yes
            } else {
                IsTy::No
            };
            let key = DeclarationKey::new(self.analyzer.scope, ident_id);
            if let Some(id) = self.analyzer.declarations.getid(key) {
                let already_declared_span = self.analyzer.declarations.getinfo(id, 0).unwrap().span;
                self.errors.push(
                    AnalysisError::DuplicateItem {
                        already_declared_span: already_declared_span.into(),
                        duplicate_span: ident_span.into(),
                    }
                    .into(),
                );
                continue;
            }
            self.analyzer.declarations.insert(key, ident_span, is_ty);
        }

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
    scope: ScopeId,
    idents: IdentStore<'a>,
    declarations: DeclarationStore,
    symbols: SymbolStore,
}

impl Analyzer<'_> {
    ///only called by collect top level declarations, and thus the declaration must already be
    ///inserted
    fn register_packing(&mut self, packing: &Packing) -> miette::Result<DeclarationId> {
        let packing_iid = self.idents.contains(packing.ident.0).unwrap();
        let packing_key = DeclarationKey::new(self.scope, packing_iid);
        let packing_did = self.declarations.getid(packing_key).unwrap();

        let mut symbols = Vec::new();
        for field in &packing.fields {
            symbols.push(self.register_field(field, packing_did)?);
        }

        let packing_info = self.declarations.getmutinfo(packing_did, 0).unwrap();
        packing_info.ty = DeclarationType::Packing(symbols);
        Ok(packing_did)
    }

    // fn register_aor(&mut self, aor: &Aor) -> DeclarationId {
    //     let packing_span = aor.ident.0;
    //     let packing_ident = self
    //         .idents
    //         .contains(packing_span)
    //         .expect("item must be inserted");
    //     let packing_id = self
    //         .declarations
    //         .getid(packing_ident)
    //         .expect("item must be inserted");
    //
    //     let mut symbols = Vec::new();
    //     for variant in &aor.variants {
    //         symbols.push(match variant {
    //             Variant::Field(field) => self.register_field(field),
    //             Variant::SpannedIdent(spanned_ident) => self.register_variant(spanned_ident),
    //         })
    //     }
    //
    //     let packing_info = self
    //         .declarations
    //         .getmutinfo(packing_id)
    //         .expect("item must be inserted");
    //
    //     packing_info.ty = DeclarationType::Packing(symbols);
    //     packing_id
    // }
    //
    // fn register_variant(&mut self, ident: SpannedIdent) -> SymbolId {
    //     let ident_span = ident.0;
    //     let ident_id = self.idents.insert(ident_span); //we can have diff symbols, so we need diff
    //     //id's, and not identid -> symbolid, because
    //     //a packing Foo {foo} and a packing Bar {foo}
    //     //-> will map to same symbol even though they
    //     //are completely different
    //     //but decs are goiung to be unique, so no problem there, but symbols are defined by a scope
    //     //-> so we may use a pair of ident ids??? where the first defines the scope, and the second
    //     //the symbol insdie that scope -> i think this would break, but dont know why -> so we do
    //     //need scoped symboling -> and in the topmost thing, we define scopes, where each scope
    //     //stores a symbol list  -> so instead what we can do is have a (dec id, ident id) pair-> resulting in a
    //     //symbol id
    //     todo!()
    // }

    ///interpretation matters, as an ident can be both symbol as well as item
    fn register_field(&mut self, field: &Field, did: DeclarationId) -> miette::Result<SymbolId> {
        let field_span = field.ident.0;
        let field_iid = self.idents.insert(field_span);
        let field_key = SymbolKey::new(self.scope, did, field_iid);
        if self.symbols.getid(field_key).is_some() {
            panic!("multiple same named symbol");
        }
        let field_sid = self.symbols.insert(field_key, field_span);

        let new_field_type = match &field.ty {
            IdentTy::Type(spanned_ty) => SymbolType::BuiltInType(*spanned_ty),
            IdentTy::Ident(spanned_ident) => {
                let type_span = spanned_ident.0;
                let type_ident = self.idents.contains(type_span).expect("undefined type");
                let key = DeclarationKey::new(self.scope, type_ident);
                let type_id = self.declarations.getid(key).expect("not a type");
                let type_info = self.declarations.getmutinfo(type_id, 0).unwrap();

                if !matches!(type_info.ty, DeclarationType::PendingType) {
                    panic!("an item but not a type");
                }

                SymbolType::UserDefinedType(type_id)
            }
            IdentTy::Arr(spanned_arr) => todo!(),
            IdentTy::Ptr(spanned_ptr) => todo!(),
        };

        let field_info = self
            .symbols
            .getmutinfo(field_sid, 0)
            .expect("just manufactured the id");

        field_info.ty = new_field_type;
        Ok(field_sid)
    }
}
