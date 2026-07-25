use super::*;
use crate::analysis::store::declaration::DeclarationType;

impl Analyzer<'_> {
    //TODO: change errors
    pub fn register_require(&mut self, require: &Require) {
        let type_span = require.ident.0;
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

        let api_span = require.api.0;
        let api_iid = self.idents.insert(api_span);
        let api_key = Key::new(self.scope, api_iid);
        match self.declarations.get(api_key) {
            Some(did) => {
                let api_info = self.declarations.refer(did);
                if !matches!(api_info.ty, DeclarationType::Api(_)) {
                    self.errors.push(
                        AnalysisError::MethodsForNotAType {
                            span: api_span.into(),
                            item_span: api_info.span.into(),
                        }
                        .into(),
                    );
                }
            }
            None => {
                self.errors.push(
                    AnalysisError::MethodsForUndefinedType {
                        span: api_span.into(),
                    }
                    .into(),
                );
            }
        };

        //1. we have to know if subapi are implemented for the type -> so we first have to collect
        //   all require declarations, lol -> wait we can skip this, and check this at the end
        //2. if we are sure of this, then we need to check if exactly all the functions of the api
        //   have been implemented, no more and no less
        //3. attach the require whatsoever, because we need the other functions to properly work

        // let type_info = self
        //     .declarations
        //     .get(type_key)
        //     .map(|did| self.declarations.refer_mut(did));
    }
}
