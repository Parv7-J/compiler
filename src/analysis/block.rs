use crate::{
    analysis::{analyzer::Analyzer, store::declaration::DeclarationInfo},
    parser::ast::Procedure,
};

use super::*;
use crate::analysis::store::declaration::{DeclarationKey, DeclarationType};

impl Analyzer<'_> {
    pub fn dinfo_at(&mut self, id: DeclarationId) -> &DeclarationInfo {
        let entry = &mut self.declarations.db[id.0];
        //SAFETY: starts at 0, and thus always starts at 1, and 1 - 1 = 0
        //OVERFLOW: can overflow, if we change the type for memory efficiency
        entry.at += 1;
        &mut entry.info[entry.at - 1]
    }

    pub fn get_procedure(&mut self, procedure: &Procedure) {
        let ident_span = procedure.ident.0;
        let procedure_did = self.did(ident_span);
        let procedure_scope = self.dinfo_at(procedure_did).scope_id;
        self.enter_scope(procedure_scope);
        self.block(&procedure.body);
        self.exit_scope();
    }

    pub fn block(&mut self, block: &Block) {
        let items = block.0.iter().filter_map(|i| match i {
            BlockItem::Item(item) => {
                if !matches!(item, Item::Methods(_) | Item::Require(_)) {
                    Some(item)
                } else {
                    None
                }
            }
            BlockItem::Stmt(_stmt) => None,
        });
        for item in items {
            let item_span = item.span();
            let item_iid = self.idents.insert(item_span);

            let item_key = DeclarationKey::new(self.scope, item_iid);

            if let Some(item_did) = self.declarations.get_did(item_key) {
                let declared_span = self.declarations.first_declaration(item_did).span;
                self.errors.push(
                    AnalysisError::DuplicateItem {
                        declared_span: declared_span.into(),
                        duplicate_span: item_span.into(),
                    }
                    .into(),
                );
            }

            let ty = match item {
                Item::Packing(_) => DeclarationType::packing(),
                Item::Aor(_) => DeclarationType::aor(),
                Item::Procedure(_) => DeclarationType::Procedure(None),
                Item::Api(_) => DeclarationType::Api(None),
                _ => unreachable!("filtered"),
            };

            self.declarations.insert(item_key, item_span, ty);
        }

        let items = block.0.iter().filter_map(|i| match i {
            BlockItem::Item(item) => Some(item),
            BlockItem::Stmt(_stmt) => None,
        });

        for item in items {
            match item {
                Item::Packing(packing) => {
                    self.register_packing(packing);
                }
                Item::Aor(aor) => {
                    self.register_aor(aor);
                }
                Item::Procedure(procedure) => {
                    self.register_procedure(procedure);
                }
                Item::Methods(methods) => {
                    self.register_methods(methods);
                }
                Item::Api(api) => {
                    self.register_api(api);
                }
                Item::Require(_require) => todo!(),
                Item::Get(_get) => todo!(),
                Item::Block(_) => unreachable!(),
            };
        }
    }
}
