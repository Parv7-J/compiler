//TODO: fix methods, requires structures inside the DeclarationStore, as it is right now a pain to
//find them
//TODO: add other procedures collection, which are inside methods and requires, but can only be done
//after completing the previous todo
use crate::{
    analysis::{analyzer::Analyzer, store::declaration::DeclarationInfo},
    parser::ast::Procedure,
};

use super::*;
use crate::analysis::store::declaration::{DeclarationKey, DeclarationType};

impl Analyzer<'_> {
    pub fn collect(&mut self, items: &[&Item]) {
        let item_iter = items
            .iter()
            .filter(|item| !matches!(item, Item::Methods(_) | Item::Require(_)));

        for item in item_iter {
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

        //the logic is correct, but unnecessary setting of 'at' for already done scopes
        self.declarations.db.iter_mut().for_each(|ent| {
            ent.at = 0;
        });
        self.symbols.db.iter_mut().for_each(|ent| {
            ent.at = 0;
        });
        self.declarations.unknown.values_mut().for_each(|ent| {
            ent.at = 0;
        });
    }

    pub fn recurse(&mut self, items: &[&Item]) {
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

    pub fn dinfo_at(&mut self, id: DeclarationId) -> &DeclarationInfo {
        let entry = &mut self.declarations.db[id.0];
        //SAFETY: starts at 0, and thus always starts at 1, and 1 - 1 = 0
        //OVERFLOW: can overflow, if we change the type for memory efficiency
        entry.at += 1;
        &entry.info[entry.at - 1]
    }

    pub fn get_procedure(&mut self, procedure: &Procedure) {
        let ident_span = procedure.ident.0;
        let procedure_did = self.did(ident_span);
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
        self.collect(items.as_ref());
        //dfs
        self.recurse(items.as_ref());
        //here we need to call self.register_stmts
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
