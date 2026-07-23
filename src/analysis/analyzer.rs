//TODO: methods should get 'this' symbol too
use miette::Report;
use std::collections::HashMap;

use super::*;
use crate::{
    analysis::store::declaration::{DeclarationKey, DeclarationType, UnknownType},
    lexer::token::Span,
};

#[derive(Debug)]
pub struct Analyzer<'a> {
    pub scope: ScopeId,
    pub idents: IdentStore<'a>,
    pub declarations: DeclarationStore,
    pub symbols: SymbolStore,
    pub errors: Vec<Report>,
}

impl Analyzer<'_> {
    pub fn register_methods(&mut self, methods: &Methods) {
        let type_span = methods.ident.0;
        let type_iid = self.idents.insert(type_span);
        let type_key = DeclarationKey::new(self.scope, type_iid);
        match self.declarations.get_did(type_key) {
            Some(did) => {
                let type_info = self.declarations.first_declaration(did);
                if !matches!(
                    type_info.ty,
                    DeclarationType::Packing(_) | DeclarationType::Aor(_)
                ) {
                    self.errors.push(
                        AnalysisError::MethodsForNotAType {
                            span: type_span.into(),
                            item_span: type_info.span.into(),
                        }
                        .into(),
                    );
                }
            }
            None => {
                self.errors.push(
                    AnalysisError::MethodsForUndefinedType {
                        span: type_span.into(),
                    }
                    .into(),
                );
            }
        };

        let methods_scope = self.declarations.new_scope(self.scope);
        self.enter_scope(methods_scope);
        self.register_method_list(&methods.procedures);
        let procedures = self.check_init(&methods.procedures, type_span);

        let type_info = self
            .declarations
            .get_did(type_key)
            .map(|did| self.declarations.first_declaration_mut(did));

        if let Some(info) = type_info {
            match &mut info.ty {
                DeclarationType::Packing(packing) => {
                    packing.add_methods(procedures);
                    self.exit_scope();
                    return;
                }
                DeclarationType::Aor(aor) => {
                    aor.add_methods(procedures);
                    self.exit_scope();
                    return;
                }
                _ => {}
            }
        }

        self.exit_scope();

        self.declarations.insert_unknown(
            DeclarationKey::new(self.scope, type_iid),
            type_span,
            UnknownType::UnknownMethods(procedures),
        );
    }

    pub fn check_init(
        &mut self,
        procedures: &[Procedure],
        span: Span,
    ) -> HashMap<IdentId, DeclarationId> {
        let mut has_init = false;
        let mut inner_methods = HashMap::new();
        for procedure in procedures {
            let (iid, did) = self.register_procedure(procedure);
            if self.idents.get(iid).unwrap() == "init" {
                has_init = true;
            }
            inner_methods.insert(iid, did);
        }
        if !has_init {
            self.errors
                .push(AnalysisError::InitMethodUndefined { span: span.into() }.into());
        }
        inner_methods
    }

    pub fn register_api(&mut self, api: &Api) -> DeclarationId {
        let api_did = self.did(api.ident.0);

        let mut supers = Vec::new();
        for api in &api.super_api {
            match self.api_did(api.0) {
                Some(did) => supers.push(did),
                None => continue,
            }
        }

        let api_scope = self.declarations.dinfo(api_did).scope_id;
        self.enter_scope(api_scope);
        self.register_method_list(&api.procedures);

        let mut methods = HashMap::new();
        for procedure in &api.procedures {
            let (iid, did) = self.register_procedure(procedure);
            //only one iid - did pair exists
            methods.insert(iid, did);
        }

        self.resolve(api_did, DeclarationType::api(supers, methods));
        self.exit_scope();

        api_did
    }

    fn api_did(&mut self, s_span: Span) -> Option<DeclarationId> {
        let s_iid = self.idents.insert(s_span);

        let s_did = match self.find_declaration(s_iid) {
            Some(did) => did,
            None => {
                self.errors.push(
                    AnalysisError::UndefinedApi {
                        span: s_span.into(),
                    }
                    .into(),
                );
                return None;
            }
        };

        let s_info = self.declarations.first_declaration(s_did);
        if !matches!(s_info.ty, DeclarationType::Api(_)) {
            self.errors.push(
                AnalysisError::NotAnApi {
                    span: s_span.into(),
                    item_span: s_info.span.into(),
                }
                .into(),
            );
            return None;
        }

        Some(s_did)
    }

    fn register_method_list(&mut self, procedures: &[Procedure]) {
        for procedure in procedures {
            let procedure_span = procedure.ident.0;
            let procedure_iid = self.idents.insert(procedure_span);

            let procedure_key = DeclarationKey::new(self.scope, procedure_iid);

            if let Some(procedure_did) = self.declarations.get_did(procedure_key) {
                let declared_span = self.declarations.first_declaration(procedure_did).span;
                self.errors.push(
                    AnalysisError::DuplicateMethod {
                        declared_span: declared_span.into(),
                        duplicate_span: procedure_span.into(),
                    }
                    .into(),
                );
            }

            self.declarations.insert(
                procedure_key,
                procedure_span,
                DeclarationType::Procedure(None),
            );
        }
    }

    pub fn register_procedure(&mut self, procedure: &Procedure) -> (IdentId, DeclarationId) {
        let ident_span = procedure.ident.0;
        let procedure_iid = self.idents.insert(ident_span);
        let procedure_did = self.did(ident_span);
        let procedure_scope = self.declarations.dinfo(procedure_did).scope_id;
        self.enter_scope(procedure_scope);

        let mut arguments = HashMap::new();

        for arg in &procedure.args {
            let (iid, sid) = self.register_field(arg);
            arguments.insert(iid, sid);
        }

        let return_ty = procedure
            .return_value
            .as_ref()
            .map(|ty| self.symbol_type(ty));

        self.resolve(
            procedure_did,
            DeclarationType::procedure(arguments, return_ty),
        );
        self.exit_scope();

        (procedure_iid, procedure_did)
    }

    pub fn register_packing(&mut self, packing: &Packing) -> DeclarationId {
        let packing_did = self.did(packing.ident.0);
        let packing_scope = self.declarations.dinfo(packing_did).scope_id;
        self.enter_scope(packing_scope);

        let mut fields = HashMap::new();

        for field in &packing.fields {
            let (iid, sid) = self.register_field(field);
            fields.insert(iid, sid);
        }

        let info = self.declarations.get_dinfo(packing_did);
        if let DeclarationType::Packing(ref mut packing) = info.ty {
            packing.set_fields(fields);
        } else {
            unreachable!("a packing cant be anything else")
        }

        self.exit_scope();

        packing_did
    }

    pub fn register_aor(&mut self, aor: &Aor) -> DeclarationId {
        let aor_did = self.did(aor.ident.0);
        let aor_scope = self.declarations.dinfo(aor_did).scope_id;
        self.enter_scope(aor_scope);

        let mut variants = HashMap::new();
        for variant in &aor.variants {
            match variant {
                Variant::Field(field) => {
                    let (iid, sid) = self.register_variant(field.ident.0);
                    let s_ty = self.symbol_type(&field.ty);
                    let s_info = self.symbols.get_sinfo(sid);
                    s_info.ty = s_ty;
                    variants.insert(iid, sid);
                }
                Variant::SpannedIdent(ident) => {
                    let (iid, sid) = self.register_variant(ident.0);
                    let s_info = self.symbols.get_sinfo(sid);
                    s_info.ty = SymbolType::Variant;
                    variants.insert(iid, sid);
                }
            }
        }

        let info = self.declarations.get_dinfo(aor_did);
        if let DeclarationType::Aor(ref mut aor) = info.ty {
            aor.set_variants(variants);
        } else {
            unreachable!("an aor cant be anything else")
        }

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
            let declared_span = self.symbols.first_declaration(field_sid).span.into();
            self.errors.push(
                AnalysisError::DuplicateField {
                    declared_span,
                    duplicate_span: field_span.into(),
                }
                .into(),
            );
        }

        let field_sid = self.symbols.insert(field_key, field_span);
        let field_type = self.symbol_type(&field.ty);
        let field_info = self.symbols.get_sinfo(field_sid);
        field_info.ty = field_type;

        (field_iid, field_sid)
    }

    fn register_variant(&mut self, variant_span: Span) -> (IdentId, SymbolId) {
        let variant_iid = self.idents.insert(variant_span);
        let variant_key = SymbolKey::new(self.scope, variant_iid);

        if let Some(variant_sid) = self.symbols.get_sid(variant_key) {
            let declared_span = self.symbols.first_declaration(variant_sid).span.into();
            self.errors.push(
                AnalysisError::DuplicateVariant {
                    declared_span,
                    duplicate_span: variant_span.into(),
                }
                .into(),
            );
        }

        let variant_sid = self.symbols.insert(variant_key, variant_span);
        (variant_iid, variant_sid)
    }

    fn symbol_type(&mut self, ty: &IdentTy) -> SymbolType {
        match ty {
            IdentTy::Type(spanned_ty) => SymbolType::BuiltInType(*spanned_ty),
            IdentTy::Ident(spanned_ident) => {
                let adt_span = spanned_ident.0;
                let adt_iid = self.idents.insert(adt_span);

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
    pub fn did(&mut self, span: Span) -> DeclarationId {
        let iid = self.idents.contains(span).unwrap();
        let key = DeclarationKey::new(self.scope, iid);
        self.declarations.get_did(key).unwrap()
    }

    ///doesnt check if the packing is already initialized, so the caller must make sure thats
    ///not the case
    ///NOTE: calls get_dinfo once, thus moving the 'at' pointer for packing declaration by 1
    fn resolve(&mut self, did: DeclarationId, resolved: DeclarationType) {
        //TODO: this is wrong
        let info = self.declarations.get_dinfo(did);
        info.ty = resolved;
    }

    ///enters the given scope
    pub fn enter_scope(&mut self, scope: ScopeId) {
        self.scope = scope;
    }

    ///exits the current scope, and returns to the parent scope
    pub fn exit_scope(&mut self) {
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
