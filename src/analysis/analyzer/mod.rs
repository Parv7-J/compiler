//TODO: methods should get 'this' symbol too
use crate::analysis::declaration::DeclarationType;
use miette::Report;

use super::*;

mod api;
mod methods;
mod procedure;
mod require;
mod types;

#[derive(Debug)]
pub struct Analyzer<'a> {
    pub scope: ScopeId,
    pub idents: IdentStore<'a>,
    pub declarations: DeclarationStore,
    pub symbols: SymbolStore,
    pub errors: Vec<Report>,
}

impl Analyzer<'_> {
    pub fn symbol_type(&mut self, ty: &IdentTy) -> SymbolType {
        match ty {
            IdentTy::Type(spanned_ty) => SymbolType::BuiltInType(*spanned_ty),
            IdentTy::Ident(spanned_ident) => {
                let adt_span = spanned_ident.0;
                let adt_iid = self.idents.insert(adt_span);

                let adt_did = match self.declarations.find(self.scope, adt_iid) {
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

                let adt_info = self.declarations.refer(adt_did);
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

    ///enters the given scope
    pub fn enter_scope(&mut self, scope: ScopeId) {
        self.scope = scope;
    }

    ///exits the current scope, and returns to the parent scope
    pub fn exit_scope(&mut self) {
        self.scope = self.declarations.parent_scope(self.scope);
    }
}
