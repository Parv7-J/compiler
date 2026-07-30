//TODO: methods should get 'this' symbol too
use std::collections::HashMap;

use super::*;
use crate::{
    analysis::store::declaration::{DeclarationType, ScopedMethods, UnknownType},
    lexer::token::Span,
};

impl Analyzer<'_> {
    pub fn attach_methods(&mut self, methods: &Methods) {
        let type_span = methods.ident.0;
        let type_iid = self.idents.insert(type_span);
        let type_key = Key::new(self.scope, type_iid);
        match self.declarations.get(type_key) {
            Some(did) => {
                let type_info = self.declarations.refer(did);
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

        self.register_procedures(&methods.procedures);
        let procedures = self.initialize_methods(&methods.procedures, type_span);

        let type_info = self
            .declarations
            .get(type_key)
            .map(|did| self.declarations.refer_mut(did));

        if let Some(info) = type_info {
            match &mut info.ty {
                DeclarationType::Packing(packing) => {
                    packing.add_methods(procedures, self.scope);
                    self.exit_scope();
                    return;
                }
                DeclarationType::Aor(aor) => {
                    aor.add_methods(procedures, self.scope);
                    self.exit_scope();
                    return;
                }
                _ => {}
            }
        }

        self.declarations.insert_unknown(
            Key::new(self.declarations.parent_scope(self.scope), type_iid),
            type_span,
            UnknownType::UnknownMethods(ScopedMethods {
                scope: self.scope,
                methods: procedures,
            }),
            self.scope,
        );

        self.exit_scope();
    }

    pub fn register_procedures(&mut self, procedures: &[Procedure]) {
        for procedure in procedures {
            let procedure_span = procedure.ident.0;
            let procedure_iid = self.idents.insert(procedure_span);

            let procedure_key = Key::new(self.scope, procedure_iid);

            if let Some(procedure_did) = self.declarations.get(procedure_key) {
                let declared_span = self.declarations.refer(procedure_did).span;
                self.errors.push(
                    AnalysisError::DuplicateMethod {
                        declared_span: declared_span.into(),
                        duplicate_span: procedure_span.into(),
                    }
                    .into(),
                );
            }

            self.declarations
                .insert(procedure_key, procedure_span, DeclarationType::procedure());
        }
    }
    fn initialize_methods(
        &mut self,
        procedures: &[Procedure],
        span: Span,
    ) -> HashMap<IdentId, DeclarationId> {
        let mut has_init = false;
        let mut inner_methods = HashMap::new();
        for procedure in procedures {
            let (iid, did) = self.initialize_procedure(procedure);
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
}
