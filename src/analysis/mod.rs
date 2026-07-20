//TODO: right now duplicate declarations are cooking us, as the analyzer is thinking there is only
//one declaration while doing symbol resolution
//i.e it doesnt understand the diff between a same named field in two diff structs of the same name,
//and a same named field in the same struct -> as symbols are being ided by sid, iid, did
//maybe the fix is to have the struct store a mapping of iid -> vec<sid>
//not a big bug but needs to be fixed

use std::{collections::HashMap, sync::Arc};

use crate::{lexer::token::Span, parser::ast::*};

mod error;
mod types;
use error::*;
use miette::Report;
use types::*;

pub struct AstAnalyzer<'a> {
    ast: Ast<'a>,
    analyzer: Analyzer<'a>,
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
                errors: Vec::new(),
            },
        }
    }

    pub fn analyze(mut self) -> Analyzer<'a> {
        self.collect_top_level_definitions();

        let source = Arc::new(miette::NamedSource::new(
            "language",
            self.ast.input.to_string(),
        ));
        if !self.analyzer.errors.is_empty() {
            eprintln!("Found {} semantic errors ->\n", self.analyzer.errors.len());
        }
        let errors = std::mem::take(&mut self.analyzer.errors);
        for (no, report) in errors.into_iter().enumerate() {
            eprintln!(
                "Error {}:\n {:?}\n",
                no + 1,
                report.with_source_code(source.clone())
            );
        }

        self.analyzer
    }

    pub fn collect_top_level_definitions(&mut self) {
        let analyzer = &mut self.analyzer;
        let items = &self.ast.items;

        for item in items {
            let item_span = item.span();
            let item_iid = analyzer.idents.insert(item_span);

            let is_ty = if matches!(item, Item::Packing(_) | Item::Aor(_)) {
                IsTy::Yes
            } else {
                IsTy::No
            };

            let item_key = DeclarationKey::new(analyzer.scope, item_iid);

            if let Some(item_did) = analyzer.declarations.get_did(item_key) {
                let already_declared_span = analyzer.declarations.first_declaration(item_did).span;
                analyzer.errors.push(
                    AnalysisError::DuplicateItem {
                        already_declared_span: already_declared_span.into(),
                        duplicate_span: item_span.into(),
                    }
                    .into(),
                );
            }

            analyzer.declarations.insert(item_key, item_span, is_ty);
        }

        for item in items {
            match item {
                Item::Packing(packing) => analyzer.register_packing(packing),
                Item::Aor(aor) => analyzer.register_aor(aor),
                Item::Procedure(procedure) => analyzer.register_procedure(procedure),
                Item::Methods(_methods) => todo!(),
                Item::Api(_api) => todo!(),
                Item::Require(_require) => todo!(),
                Item::Get(_get) => todo!(),
                Item::Block(_) => unreachable!(),
            };
        }
    }
}

#[derive(Debug)]
pub struct Analyzer<'a> {
    scope: ScopeId,
    idents: IdentStore<'a>,
    declarations: DeclarationStore,
    symbols: SymbolStore,
    errors: Vec<Report>,
}

impl Analyzer<'_> {
    fn register_procedure(&mut self, procedure: &Procedure) {
        let procedure_did = self.did(procedure.ident.0);
        let mut arguments = HashMap::new();

        for arg in &procedure.args {
            if let Some((iid, sid)) = self.register_field(arg, procedure_did) {
                arguments.insert(iid, sid);
            }
        }

        let return_ty = procedure
            .return_value
            .as_ref()
            .map(|ty| self.symbol_type(ty));

        let procedure_info = self.declarations.get_dinfo(procedure_did);
        procedure_info.ty = DeclarationType::Resolved(Resolved::Procedure {
            arguments,
            return_ty,
        });
    }

    // fn register_methods(&mut self, methods: &Methods) {
    //     let procedure_did = self.did(methods.ident.0);
    //     let mut arguments = HashMap::new();
    //
    //     for arg in &procedure.args {
    //         if let Some((iid, sid)) = self.register_field(arg, procedure_did) {
    //             arguments.insert(iid, sid);
    //         }
    //     }
    //
    //     let return_ty = procedure
    //         .return_value
    //         .as_ref()
    //         .map(|ty| self.symbol_type(ty));
    //
    //     let procedure_info = self.declarations.get_dinfo(procedure_did);
    //     procedure_info.ty = DeclarationType::Resolved(Resolved::Procedure {
    //         arguments,
    //         return_ty,
    //     });
    // }

    fn register_api(&mut self, procedure: &Procedure) {
        let procedure_did = self.did(procedure.ident.0);
        let mut arguments = HashMap::new();

        for arg in &procedure.args {
            if let Some((iid, sid)) = self.register_field(arg, procedure_did) {
                arguments.insert(iid, sid);
            }
        }

        let return_ty = procedure
            .return_value
            .as_ref()
            .map(|ty| self.symbol_type(ty));

        let procedure_info = self.declarations.get_dinfo(procedure_did);
        procedure_info.ty = DeclarationType::Resolved(Resolved::Procedure {
            arguments,
            return_ty,
        });
    }

    fn register_require(&mut self, procedure: &Procedure) {
        let procedure_did = self.did(procedure.ident.0);
        let mut arguments = HashMap::new();

        for arg in &procedure.args {
            if let Some((iid, sid)) = self.register_field(arg, procedure_did) {
                arguments.insert(iid, sid);
            }
        }

        let return_ty = procedure
            .return_value
            .as_ref()
            .map(|ty| self.symbol_type(ty));

        let procedure_info = self.declarations.get_dinfo(procedure_did);
        procedure_info.ty = DeclarationType::Resolved(Resolved::Procedure {
            arguments,
            return_ty,
        });
    }

    fn register_packing(&mut self, packing: &Packing) {
        //TODO: dont register the packing as it wont even be used?, and type checking is done normally
        //like that only
        let packing_did = self.did(packing.ident.0);
        let mut symbols = HashMap::new();

        for field in &packing.fields {
            if let Some((iid, sid)) = self.register_field(field, packing_did) {
                symbols.insert(iid, sid);
            }
        }

        self.resolve(packing_did, ResolvedType::Packing(symbols));
    }

    fn register_aor(&mut self, aor: &Aor) {
        let aor_did = self.did(aor.ident.0);
        let mut symbols = HashMap::new();

        for variant in &aor.variants {
            match variant {
                Variant::Field(field) => {
                    //TODO: make register_variant which pushes a duplicate variant not duplicate
                    //field error
                    if let Some((iid, sid)) = self.register_field(field, aor_did) {
                        symbols.insert(iid, sid);
                    }
                }
                Variant::SpannedIdent(spanned_ident) => {
                    let variant_span = spanned_ident.0;
                    let variant_iid = self.idents.insert(variant_span);
                    let variant_key = SymbolKey::new(self.scope, aor_did, variant_iid);
                    if let Some(variant_sid) = self.symbols.get_sid(variant_key) {
                        let already_declared_span =
                            self.symbols.first_declaration(variant_sid).span.into();
                        self.errors.push(
                            AnalysisError::DuplicateVariant {
                                already_declared_span,
                                duplicate_span: variant_span.into(),
                            }
                            .into(),
                        );
                        continue;
                    }
                    let variant_sid = self.symbols.insert(variant_key, variant_span);
                    let variant_info = self.symbols.get_sinfo(variant_sid);
                    variant_info.ty = SymbolType::Variant;
                    symbols.insert(variant_iid, variant_sid);
                }
            }
        }

        self.resolve(aor_did, ResolvedType::Aor(symbols));
    }

    ///doesnt check if the field is already initialized, so the caller must make sure thats
    ///not the case
    ///NOTE: calls get_sinfo once, thus moving the 'at' pointer for field declaration by 1
    fn register_field(&mut self, field: &Field, did: DeclarationId) -> Option<(IdentId, SymbolId)> {
        let field_span = field.ident.0;
        let field_iid = self.idents.insert(field_span);
        let field_key = SymbolKey::new(self.scope, did, field_iid);

        if let Some(field_sid) = self.symbols.get_sid(field_key) {
            //here we need to actually store dup fields, as they may belong to dup items
            let already_declared_span = self.symbols.first_declaration(field_sid).span.into();
            self.errors.push(
                AnalysisError::DuplicateField {
                    already_declared_span,
                    duplicate_span: field_span.into(),
                }
                .into(),
            );
            return None;
        }

        let field_sid = self.symbols.insert(field_key, field_span);
        let field_type = self.symbol_type(&field.ty);
        let field_info = self.symbols.get_sinfo(field_sid);
        field_info.ty = field_type;

        Some((field_iid, field_sid))
    }

    fn symbol_type(&mut self, ty: &IdentTy) -> SymbolType {
        match ty {
            IdentTy::Type(spanned_ty) => SymbolType::BuiltInType(*spanned_ty),
            IdentTy::Ident(spanned_ident) => {
                let adt_span = spanned_ident.0;
                let adt_iid = match self.idents.contains(adt_span) {
                    Some(iid) => iid,
                    None => {
                        self.errors.push(
                            AnalysisError::UndefinedType {
                                span: adt_span.into(),
                            }
                            .into(),
                        );
                        return SymbolType::Error { span: adt_span };
                    }
                };

                let adt_key = DeclarationKey::new(self.scope, adt_iid);
                let adt_did = match self.declarations.get_did(adt_key) {
                    Some(did) => did,
                    None => {
                        self.errors.push(
                            AnalysisError::UndefinedType {
                                span: adt_span.into(),
                            }
                            .into(),
                        );
                        return SymbolType::Error { span: adt_span };
                    }
                };

                let adt_info = self.declarations.dinfo(adt_did);
                if !matches!(
                    adt_info.ty,
                    DeclarationType::PendingType | DeclarationType::ResolvedType(_)
                ) {
                    self.errors.push(
                        AnalysisError::NotAType {
                            span: adt_span.into(),
                            item_span: adt_info.span.into(),
                        }
                        .into(),
                    );
                    return SymbolType::Error { span: adt_span };
                }

                SymbolType::UserDefinedType(adt_did)
            }
            IdentTy::Arr(spanned_arr) => {
                let inner_ty = &spanned_arr.inner_ty;
                let symbol_type = self.symbol_type(inner_ty);

                SymbolType::ArrType {
                    arr_ty: spanned_arr.arr_ty,
                    inner_ty: Box::new(symbol_type),
                    span: spanned_arr.span,
                }
            }
            IdentTy::Ptr(spanned_ptr) => {
                let inner_ty = &spanned_ptr.ty;
                let symbol_type = self.symbol_type(inner_ty);

                SymbolType::PtrType {
                    ty: Box::new(symbol_type),
                    span: spanned_ptr.ptr,
                }
            }
        }
    }

    ///panics if span was not already inserted, in both identstore(a call to 'contains' just for checking logic)
    ///and declarationstore
    fn did(&mut self, span: Span) -> DeclarationId {
        let iid = self.idents.contains(span).unwrap();
        let key = DeclarationKey::new(self.scope, iid);
        self.declarations.get_did(key).unwrap()
    }

    ///doesnt check if the packing is already initialized, so the caller must make sure thats
    ///not the case
    ///NOTE: calls get_dinfo once, thus moving the 'at' pointer for packing declaration by 1
    fn resolve(&mut self, did: DeclarationId, resolved_type: ResolvedType) {
        let info = self.declarations.get_dinfo(did);
        info.ty = DeclarationType::ResolvedType(resolved_type);
    }
}
