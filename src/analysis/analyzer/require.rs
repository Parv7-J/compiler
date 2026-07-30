use std::collections::{HashMap, hash_map::Entry};

use super::*;
use crate::{
    Span,
    analysis::store::declaration::{DeclarationType, ScopedMethods},
};

//IMPL: look up trait definitoins while resolving method calls, not the require blocks

impl Analyzer<'_> {
    pub fn attach_api(&mut self, require: &Require) {
        let type_span = require.ident.0;
        let type_key = Key::new(self.scope, self.idents.insert(type_span));
        let is_type = self.is_type(type_key, type_span);

        let api_span = require.api.0;
        let api_key = Key::new(self.scope, self.idents.insert(api_span));
        let is_api = self.is_api(api_key, api_span);

        let new_scope = self.declarations.new_scope(self.scope);
        self.enter_scope(new_scope);

        self.register_procedures(&require.procedures);
        let mut implementing_methods = HashMap::new();
        for procedure in &require.procedures {
            let reg = self.initialize_procedure(procedure);
            implementing_methods.insert(reg.0, reg.1);
        }

        if is_api {
            let api_did = self.declarations.get(api_key).unwrap();
            let api_info = self.declarations.refer(api_did);
            let DeclarationType::Api(api) = &api_info.ty else {
                unreachable!()
            };

            for not_impl in self.unmatched_methods(&api.methods, &implementing_methods) {
                let method_span = self.declarations.refer(*not_impl.1).span;
                self.errors.push(
                    AnalysisError::UnimplementedMethod {
                        method_span: method_span.into(),
                        require_span: type_span.into(),
                        api_span: api_span.into(),
                    }
                    .into(),
                );
            }

            for extra_impl in self.unmatched_methods(&implementing_methods, &api.methods) {
                let method_span = self.declarations.refer(*extra_impl.1).span;
                self.errors.push(
                    AnalysisError::ExtraMethod {
                        method_span: method_span.into(),
                        require_span: type_span.into(),
                        api_span: api_span.into(),
                    }
                    .into(),
                );
            }

            if is_type {
                let required_subapis = &api.supers;
                let type_did = self.declarations.get(type_key).unwrap();
                let type_info = self.declarations.refer(type_did);

                let implemented_subapis = match &type_info.ty {
                    DeclarationType::Packing(packing) => &packing.requires,
                    DeclarationType::Aor(aor) => &aor.requires,
                    _ => unreachable!(),
                };

                required_subapis
                    .iter()
                    .filter(|api| !implemented_subapis.contains_key(*api))
                    .for_each(|unimplemented_api| {
                        self.errors.push(
                            AnalysisError::UnimplementedSuperApi {
                                require_span: type_span.into(),
                                api_span: api_info.span.into(),
                                super_api: self.declarations.refer(*unimplemented_api).span.into(),
                            }
                            .into(),
                        );
                    });

                let type_info = self.declarations.refer_mut(type_did);
                let subapis = match &mut type_info.ty {
                    DeclarationType::Packing(packing) => &mut packing.requires,
                    DeclarationType::Aor(aor) => &mut aor.requires,
                    _ => unreachable!(),
                };

                let methods = ScopedMethods {
                    scope: self.scope,
                    methods: implementing_methods,
                };

                match subapis.entry(api_did) {
                    Entry::Occupied(mut o) => o.get_mut().push(methods),
                    Entry::Vacant(v) => {
                        v.insert(vec![methods]);
                    }
                }
            } else {
                todo!()
            }

            self.exit_scope();
        }
    }

    fn is_type(&mut self, type_key: Key, type_span: Span) -> bool {
        match self.declarations.get(type_key) {
            Some(did) => {
                let info = self.declarations.refer(did);
                if !matches!(
                    info.ty,
                    DeclarationType::Packing(_) | DeclarationType::Aor(_)
                ) {
                    self.errors.push(
                        AnalysisError::InvalidType {
                            span: type_span.into(),
                            item_span: Some(info.span.into()),
                        }
                        .into(),
                    );
                    return false;
                }
            }
            None => {
                self.errors.push(
                    AnalysisError::InvalidType {
                        span: type_span.into(),
                        item_span: None,
                    }
                    .into(),
                );
                return false;
            }
        };
        true
    }

    fn is_api(&mut self, api_key: Key, api_span: Span) -> bool {
        match self.declarations.get(api_key) {
            Some(did) => {
                let info = self.declarations.refer(did);
                if !matches!(info.ty, DeclarationType::Api(_)) {
                    self.errors.push(
                        AnalysisError::InvalidApi {
                            span: api_span.into(),
                            item_span: Some(info.span.into()),
                        }
                        .into(),
                    );
                    return false;
                }
            }
            None => {
                self.errors.push(
                    AnalysisError::InvalidApi {
                        span: api_span.into(),
                        item_span: None,
                    }
                    .into(),
                );
                return false;
            }
        };
        true
    }

    fn unmatched_methods<'a>(
        &self,
        methods: &'a HashMap<IdentId, DeclarationId>,
        filter: &'a HashMap<IdentId, DeclarationId>,
    ) -> Vec<(&'a IdentId, &'a DeclarationId)> {
        methods
            .iter()
            .filter(|(iid, did)| {
                if filter.contains_key(iid) {
                    let info = self.declarations.refer(**did);
                    let DeclarationType::Procedure(proca) = &info.ty else {
                        unreachable!()
                    };
                    let fnd = filter.get(iid).copied().unwrap();
                    let info = self.declarations.refer(fnd);
                    let DeclarationType::Procedure(procb) = &info.ty else {
                        unreachable!()
                    };
                    proca != procb
                } else {
                    true
                }
            })
            .collect()
    }
}
