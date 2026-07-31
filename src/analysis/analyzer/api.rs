use std::collections::HashMap;

use super::*;
use crate::{analysis::store::declaration::DeclarationType, lexer::token::Span};

impl Analyzer<'_> {
    pub fn initialize_api(&mut self, api: &Api) -> DeclarationId {
        let api_iid = self.idents.insert(api.ident.unwrap().0);
        let api_key = Key::new(self.scope, api_iid);
        let api_did = self.declarations.get(api_key).unwrap();

        let mut supers = Vec::new();
        for api in &api.super_api {
            match self.api_did(api.0) {
                Some(did) => supers.push(did),
                None => continue,
            }
        }

        let api_scope = self.declarations.scope_from_id(api_did);
        self.enter_scope(api_scope);

        self.register_procedures(&api.procedures);

        let mut methods = HashMap::new();
        for procedure in &api.procedures {
            let (iid, did) = self.initialize_procedure(procedure);
            //only one iid - did pair exists
            methods.insert(iid, did);
        }

        self.declarations.initialize(api_did, |info| {
            if let DeclarationType::Api(ref mut api) = info.ty {
                api.set_supers(supers);
                api.set_methods(methods);
            }
        });

        self.exit_scope();

        api_did
    }

    fn api_did(&mut self, s_span: Span) -> Option<DeclarationId> {
        let s_iid = self.idents.insert(s_span);

        let s_did = match self.declarations.find(self.scope, s_iid) {
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

        let s_info = self.declarations.refer(s_did);
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
}
