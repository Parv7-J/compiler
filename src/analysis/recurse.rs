use crate::{
    analysis::{analyzer::Analyzer, store::declaration::DeclarationInfo},
    parser::ast::Procedure,
};

use super::*;
use crate::analysis::store::declaration::DeclarationType;

impl Analyzer<'_> {
    pub fn collect_and_recurse<'a>(&mut self, items: impl AsRef<[&'a Item]>) {
        self.collect_declarations(&items);
        self.recurse_into_declarations(&items);
    }

    fn collect_declarations<'a>(&mut self, items: impl AsRef<[&'a Item]>) {
        let items = items.as_ref();

        let item_iter = items
            .iter()
            .filter(|item| !matches!(item, Item::Methods(_) | Item::Require(_)));

        for item in item_iter.clone() {
            let item_span = item.span();
            let item_iid = self.idents.insert(item_span);

            let item_key = Key::new(self.scope, item_iid);

            if let Some(item_did) = self.declarations.get(item_key) {
                let declared_span = self.declarations.refer(item_did).span;
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
                Item::Procedure(_) => DeclarationType::procedure(),
                Item::Api(_) => DeclarationType::api(),
                _ => unreachable!("filtered"),
            };

            self.declarations.insert(item_key, item_span, ty);
        }

        for item in item_iter {
            match item {
                Item::Packing(packing) => {
                    self.initialize_packing(packing);
                }
                Item::Aor(aor) => {
                    self.initialize_aor(aor);
                }
                Item::Procedure(procedure) => {
                    self.initialize_procedure(procedure);
                }
                Item::Api(api) => {
                    self.initialize_api(api);
                }
                _ => unreachable!("filtered"),
            };
        }

        let method_and_require = items
            .iter()
            .filter(|item| matches!(item, Item::Methods(_) | Item::Require(_)));

        for item in method_and_require {
            match item {
                Item::Methods(methods) => self.attach_methods(methods),
                Item::Require(require) => self.attach_api(require),
                _ => unreachable!("filtered"),
            }
        }

        self.clear();
    }

    fn recurse_into_declarations<'a>(&mut self, items: impl AsRef<[&'a Item]>) {
        let items = items.as_ref();
        for item in items {
            #[allow(clippy::single_match)]
            match item {
                Item::Procedure(procedure) => {
                    self.get_procedure(procedure);
                }
                Item::Methods(_methods) => {
                    // self.get_methods(methods);
                }
                // Item::Api(_api) => todo!(),
                // Item::Require(_require) => todo!(),
                // Item::Get(_get) => todo!(),
                // Item::Block(_block) => todo!(),
                _ => {}
            }
        }
    }

    pub fn clear(&mut self) {
        //the logic is correct, but unnecessary setting of 'current_entry' for already done scopes
        self.declarations.db.iter_mut().for_each(|ent| {
            ent.current_entry = 0;
        });

        self.declarations.unknown.values_mut().for_each(|ent| {
            ent.current_entry = 0;
        });
    }

    pub fn dinfo_at(&mut self, id: DeclarationId) -> &DeclarationInfo {
        let dentry = &mut self.declarations.db[id.0];
        //SAFETY: starts current_entry 0, and thus always starts current_entry 1, and 1 - 1 = 0
        //OVERFLOW: can overflow, if we change the type for memory efficiency
        dentry.current_entry += 1;
        &dentry.entries[dentry.current_entry - 1]
    }

    pub fn get_procedure(&mut self, procedure: &Procedure) {
        let procedure_iid = self.idents.insert(procedure.ident.unwrap().0);
        let procedure_key = Key::new(self.scope, procedure_iid);
        let procedure_did = self.declarations.get(procedure_key).unwrap();
        let procedure_scope = self.dinfo_at(procedure_did).scope_id;

        self.enter_scope(procedure_scope);
        let items = procedure
            .body
            .0
            .iter()
            .filter_map(|i| match i {
                BlockItem::Item(item) => Some(item),
                BlockItem::Stmt(_stmt) => None,
            })
            .collect::<Vec<_>>();

        self.collect_and_recurse(items);

        let stmts = procedure
            .body
            .0
            .iter()
            .filter_map(|i| match i {
                BlockItem::Item(_item) => None,
                BlockItem::Stmt(stmt) => Some(stmt),
            })
            .collect::<Vec<_>>();

        for stmt in stmts {
            self.register_stmt(stmt);
        }

        self.exit_scope();
    }
}
