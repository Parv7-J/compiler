use std::collections::HashMap;

use super::*;
use crate::analysis::store::declaration::DeclarationType;

impl Analyzer<'_> {
    pub fn initialize_procedure(&mut self, procedure: &Procedure) -> (IdentId, DeclarationId) {
        let procedure_iid = self.idents.insert(procedure.ident.unwrap().0);
        let procedure_key = Key::new(self.scope, procedure_iid);
        let procedure_did = self.declarations.get(procedure_key).unwrap();
        let procedure_scope = self.declarations.scope_from_id(procedure_did);

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

        self.declarations.initialize(procedure_did, |info| {
            if let DeclarationType::Procedure(ref mut procedure) = info.ty {
                procedure.set_arguments(arguments);
                procedure.set_return_ty(return_ty);
            }
        });

        self.exit_scope();

        (procedure_iid, procedure_did)
    }
}
