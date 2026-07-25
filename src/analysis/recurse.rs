//TODO: fix methods, requires structures inside the DeclarationStore, as it is right now a pain to
//find them
//TODO: add other procedures collection, which are inside methods and requires, but can only be done
//after completing the previous todo
use crate::{
    analysis::{analyzer::Analyzer, store::declaration::DeclarationInfo},
    parser::ast::Procedure,
};

use super::*;
use crate::analysis::store::declaration::DeclarationType;

impl Analyzer<'_> {
    pub fn collect<'a>(&mut self, items: impl AsRef<[&'a Item]>) {
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
                    self.register_packing(packing);
                }
                Item::Aor(aor) => {
                    self.register_aor(aor);
                }
                Item::Procedure(procedure) => {
                    self.register_procedure(procedure);
                }
                Item::Api(api) => {
                    self.register_api(api);
                }
                _ => unreachable!("filtered"),
            };
        }

        let method_and_require = items
            .iter()
            .filter(|item| matches!(item, Item::Methods(_) | Item::Require(_)));

        for item in method_and_require {
            match item {
                Item::Methods(methods) => self.register_methods(methods),
                Item::Require(require) => self.register_require(require),
                _ => unreachable!("filtered"),
            }
        }

        self.clear();
    }

    pub fn recurse<'a>(&mut self, items: impl AsRef<[&'a Item]>) {
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
        let procedure_iid = self.idents.insert(procedure.ident.0);
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
        self.collect(&items);
        self.recurse(&items);
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

    // pub fn get_methods(&mut self, methods: &Methods) {
    //     //we do need the methods scope
    //     let type_span = methods.ident.0;
    //     let type_iid = self.idents.contains(type_span).unwrap();
    //     let type_key = DeclarationKey::new(self.scope, type_iid);
    //     match self.declarations.get_did(type_key) {
    //         Some(did) => {
    //             let first = self.declarations.first_declaration(did);
    //             let methods_scope = match first.ty {
    //                 DeclarationType::Packing(packing) => packing.methods.0,
    //                 DeclarationType::Aor(aor) => aor.methods.0,
    //                 _ => unreachable("methods for types only"),
    //             };
    //         }
    //         None => todo!(),
    //     }
    //     for procedure in &methods.procedures {
    //         self.get_procedure(procedure);
    //     }
    //     //as the last exit will result in the exit to the methods scope
    //     if !methods.procedures.is_empty() {
    //         self.exit_scope();
    //     }
    // }
}
