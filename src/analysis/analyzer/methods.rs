//TODO: methods should get 'this' symbol too
use std::collections::HashMap;

use super::*;
use crate::{
    analysis::store::declaration::{DeclarationType, UnknownType},
    lexer::token::Span,
};

impl Analyzer<'_> {
    pub fn register_methods(&mut self, methods: &Methods) {
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
        self.register_method_list(&methods.procedures);
        let procedures = self.check_init(&methods.procedures, type_span);

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

        self.exit_scope();

        self.declarations.insert_unknown(
            Key::new(self.scope, type_iid),
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
}
