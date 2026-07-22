use super::*;
use crate::{
    analysis::store::declaration::{self, DeclarationKey, DeclarationType},
    lexer::token::Span,
};

use miette::Report;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Analyzer<'a> {
    pub scope: ScopeId,
    pub idents: IdentStore<'a>,
    pub declarations: DeclarationStore,
    pub symbols: SymbolStore,
    pub errors: Vec<Report>,
}

impl Analyzer<'_> {
    // pub fn register_methods(&mut self, methods: &Methods) {
    //     let type_span = methods.ident.0;
    //     let type_iid = self.idents.insert(type_span);
    //     let type_did = match self.find_declaration(type_iid) {
    //         Some(did) => did,
    //         None => {
    //             self.errors.push(
    //                 AnalysisError::MethodsForUndefinedType {
    //                     span: type_span.into(),
    //                 }
    //                 .into(),
    //             );
    //             return;
    //         }
    //     };
    //     let type_info = self.declarations.first_declaration_mut(type_did);
    //     if !matches!(
    //         type_info.ty,
    //         DeclarationType::Pending(Pending::Packing | Pending::Aor)
    //             | DeclarationType::Resolved(Resolved::Packing(_) | Resolved::Aor(_))
    //     ) {
    //         self.errors.push(
    //             AnalysisError::MethodsForNotAType {
    //                 span: type_span.into(),
    //                 item_span: type_info.span.into(),
    //             }
    //             .into(),
    //         );
    //         return;
    //     }
    //
    //     //we have made sure that the type is actually a type now
    //
    //     let methods_scope = self.declarations.scopes.add_scope(self.scope);
    //     self.enter_scope(methods_scope);
    //
    //     let mut procedures = HashMap::new();
    //     let mut has_init = false;
    //
    //     for method in &methods.procedures {
    //         let (iid, did) = self.register_method(method);
    //         if self.idents.get(iid).unwrap() == "init" {
    //             has_init = true;
    //         }
    //         procedures.insert(iid, did);
    //     }
    //
    //     if !has_init {
    //         self.errors.push(
    //             AnalysisError::InitMethodUndefined {
    //                 span: methods_span.into(),
    //             }
    //             .into(),
    //         );
    //     }
    //
    //     let methods_info = self.declarations.get_dinfo(methods_did);
    //     methods_info.ty = DeclarationType::Resolved(Resolved::Methods {
    //         methods: procedures,
    //     });
    //     self.exit_scope();
    // }
    // //
    // // //TODO: give the 'this' symbol to it
    // fn register_method(&mut self, method: &Procedure) -> (IdentId, DeclarationId) {
    //     let method_span = method.ident.0;
    //     let method_iid = self.idents.insert(method_span);
    //     let method_key = DeclarationKey::new(self.scope, method_iid);
    //     let method_did =
    //         self.declarations
    //             .insert(method_key, method_span, &Item::Procedure(method.clone()));
    //     let method_scope = self.declarations.dinfo(method_did).scope;
    //     self.enter_scope(method_scope);
    //
    //     let mut arguments = HashMap::new();
    //
    //     for arg in &method.args {
    //         let (iid, sid) = self.register_field(arg);
    //         arguments.insert(iid, sid);
    //     }
    //
    //     let return_ty = method.return_value.as_ref().map(|ty| self.symbol_type(ty));
    //
    //     let method_info = self.declarations.get_dinfo(method_did);
    //     method_info.ty = DeclarationType::Resolved(Resolved::Procedure {
    //         arguments,
    //         return_ty,
    //     });
    //     self.exit_scope();
    //
    //     (method_iid, method_did)
    // }
    //
    // pub fn register_procedure(&mut self, procedure: &Procedure) -> (IdentId, DeclarationId) {
    //     let ident_span = procedure.ident.0;
    //     let procedure_iid = self.idents.insert(ident_span);
    //     let procedure_did = self.did(ident_span);
    //     let procedure_scope = self.declarations.dinfo(procedure_did).scope;
    //     self.enter_scope(procedure_scope);
    //
    //     let mut arguments = HashMap::new();
    //
    //     for arg in &procedure.args {
    //         let (iid, sid) = self.register_field(arg);
    //         arguments.insert(iid, sid);
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
    //     self.exit_scope();
    //
    //     (procedure_iid, procedure_did)
    // }

    // pub fn register_api(&mut self, api: &Api) -> DeclarationId {
    //     //1. this treats same named apis and types with the same 'did', which is correct as we dont
    //     //   need to have duplicates, which this will catch at the time of collecting
    //     //2. check if we have super api's defined -> self.declarations.first_declaration -> should
    //     //   be pending api or resolved api
    //     //3. collect the procedures in the api, and now yield
    //     let api_did = self.did(api.ident.0);
    //
    //     let mut super_apis = Vec::new();
    //
    //     for api in &api.super_api {
    //         let s_span = api.0;
    //         let s_iid = self.idents.insert(s_span);
    //
    //         let s_did = match self.find_declaration(s_iid) {
    //             Some(did) => did,
    //             None => {
    //                 self.errors.push(
    //                     AnalysisError::UndefinedApi {
    //                         span: s_span.into(),
    //                     }
    //                     .into(),
    //                 );
    //                 continue;
    //             }
    //         };
    //
    //         let s_info = self.declarations.first_declaration(s_did);
    //         if !matches!(
    //             s_info.ty,
    //             DeclarationType::Pending(Pending::Api)
    //                 | DeclarationType::Resolved(Resolved::Api { .. })
    //         ) {
    //             self.errors.push(
    //                 AnalysisError::NotAnApi {
    //                     span: s_span.into(),
    //                     item_span: s_info.span.into(),
    //                 }
    //                 .into(),
    //             );
    //             continue;
    //         }
    //
    //         super_apis.push(s_did);
    //     }
    //
    //     let api_scope = self.declarations.dinfo(api_did).scope;
    //     self.enter_scope(api_scope);
    //
    //     let mut symbols = HashMap::new();
    //
    //     for procedure in &api.procedures {
    //         let (iid, did) = self.register_method(procedure);
    //         symbols.insert(iid, did);
    //     }
    //
    //     self.resolve(
    //         api_did,
    //         Resolved::Api {
    //             super_apis,
    //             procedures: symbols,
    //         },
    //     );
    //     self.exit_scope();
    //
    //     api_did
    // }

    pub fn register_packing(&mut self, packing: &Packing) -> DeclarationId {
        let packing_did = self.did(packing.ident.0);
        let packing_scope = self.declarations.dinfo(packing_did).scope_id;
        self.enter_scope(packing_scope);

        let mut symbols = HashMap::new();

        for field in &packing.fields {
            let (iid, sid) = self.register_field(field);
            symbols.insert(iid, sid);
        }

        self.resolve(
            packing_did,
            DeclarationType::Packing(Some(declaration::Packing {
                fields: symbols,
                methods: None,
                requires: None,
            })),
        );
        self.exit_scope();

        packing_did
    }

    pub fn register_aor(&mut self, aor: &Aor) -> DeclarationId {
        let aor_did = self.did(aor.ident.0);
        let aor_scope = self.declarations.dinfo(aor_did).scope_id;
        self.enter_scope(aor_scope);

        let mut symbols = HashMap::new();

        for variant in &aor.variants {
            match variant {
                Variant::Field(field) => {
                    //TODO: make register_variant which pushes a duplicate variant not duplicate
                    //field error
                    let (iid, sid) = self.register_variant(field);
                    symbols.insert(iid, sid);
                }
                Variant::SpannedIdent(spanned_ident) => {
                    let variant_span = spanned_ident.0;
                    let variant_iid = self.idents.insert(variant_span);
                    let variant_key = SymbolKey::new(self.scope, variant_iid);
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

        self.resolve(
            aor_did,
            DeclarationType::Aor(Some(declaration::Aor {
                variants: symbols,
                methods: None,
                requires: None,
            })),
        );
        self.exit_scope();

        aor_did
    }

    ///doesnt check if the field is already initialized, so the caller must make sure thats
    ///not the case
    ///NOTE: calls get_sinfo once, thus moving the 'at' pointer for field declaration by 1
    fn register_field(&mut self, field: &Field) -> (IdentId, SymbolId) {
        let field_span = field.ident.0;
        let field_iid = self.idents.insert(field_span);
        let field_key = SymbolKey::new(self.scope, field_iid);

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
        }

        //TODO: insert the symbol regardless, as we only consider the first invokation of the symbol
        let field_sid = self.symbols.insert(field_key, field_span);
        let field_type = self.symbol_type(&field.ty);
        let field_info = self.symbols.get_sinfo(field_sid);
        field_info.ty = field_type;

        (field_iid, field_sid)
    }

    fn register_variant(&mut self, variant: &Field) -> (IdentId, SymbolId) {
        let variant_span = variant.ident.0;
        let variant_iid = self.idents.insert(variant_span);
        let variant_key = SymbolKey::new(self.scope, variant_iid);

        if let Some(variant_sid) = self.symbols.get_sid(variant_key) {
            //here we need to actually store dup variants, as they may belong to dup items
            let already_declared_span = self.symbols.first_declaration(variant_sid).span.into();
            self.errors.push(
                AnalysisError::DuplicateVariant {
                    already_declared_span,
                    duplicate_span: variant_span.into(),
                }
                .into(),
            );
        }

        //TODO: insert the symbol regardless, as we only consider the first invokation of the symbol
        let variant_sid = self.symbols.insert(variant_key, variant_span);
        let variant_type = self.symbol_type(&variant.ty);
        let variant_info = self.symbols.get_sinfo(variant_sid);
        variant_info.ty = variant_type;

        (variant_iid, variant_sid)
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

                let adt_did = match self.find_declaration(adt_iid) {
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

                let adt_info = self.declarations.first_declaration(adt_did);
                if !matches!(
                    adt_info.ty,
                    DeclarationType::Aor(_) | DeclarationType::Packing(_)
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
    fn resolve(&mut self, did: DeclarationId, resolved: DeclarationType) {
        let info = self.declarations.get_dinfo(did);
        info.ty = resolved;
    }

    ///enters the given scope
    fn enter_scope(&mut self, scope: ScopeId) {
        self.scope = scope;
    }

    ///exits the current scope, and returns to the parent scope
    fn exit_scope(&mut self) {
        self.scope = self.declarations.parent_scope(self.scope);
    }

    ///starts from the current scope to the outermost scope, looking for a declaration matching the
    ///ident
    fn find_declaration(&self, iid: IdentId) -> Option<DeclarationId> {
        let mut scope = self.scope;
        loop {
            let key = DeclarationKey::new(scope, iid);
            match self.declarations.get_did(key) {
                Some(did) => return Some(did),
                None => {
                    let parent_scope = self.declarations.parent_scope(scope);
                    if scope == parent_scope {
                        break;
                    }
                    scope = parent_scope;
                }
            };
        }
        None
    }
}
