use crate::{
    analysis::store::declaration::{DeclarationKey, DeclarationType},
    parser::ast::*,
};
use std::sync::Arc;

mod analyzer;
mod error;
mod store;

use analyzer::*;
use error::*;
use store::*;

pub struct AstAnalyzer<'a> {
    ast: Ast<'a>,
    analyzer: Analyzer<'a>,
}

impl<'a> AstAnalyzer<'a> {
    pub fn new(ast: Ast<'a>) -> Self {
        let input = ast.input;
        Self {
            ast,
            analyzer: Analyzer {
                scope: ScopeId(0),
                idents: IdentStore::new(input),
                declarations: DeclarationStore::new(),
                symbols: SymbolStore::new(),
                errors: Vec::new(),
            },
        }
    }

    pub fn analyze(mut self) -> Analyzer<'a> {
        self.collect_top_level_definitions();

        let source = Arc::new(miette::NamedSource::new(
            "language",
            self.ast.input.to_string(),
        ));
        if !self.analyzer.errors.is_empty() {
            eprintln!("Found {} semantic errors ->\n", self.analyzer.errors.len());
        }
        let errors = std::mem::take(&mut self.analyzer.errors);
        for (no, report) in errors.into_iter().enumerate() {
            eprintln!(
                "Semantic Error {}:\n {:?}\n",
                no + 1,
                report.with_source_code(source.clone())
            );
        }

        self.analyzer
    }

    pub fn collect_top_level_definitions(&mut self) {
        let analyzer = &mut self.analyzer;
        let items = &self.ast.items;

        for item in items {
            if matches!(item, Item::Methods(_) | Item::Require(_)) {
                continue;
            }

            let item_span = item.span();
            let item_iid = analyzer.idents.insert(item_span);

            let item_key = DeclarationKey::new(analyzer.scope, item_iid);

            if let Some(item_did) = analyzer.declarations.get_did(item_key) {
                let already_declared_span = analyzer.declarations.first_declaration(item_did).span;
                analyzer.errors.push(
                    AnalysisError::DuplicateItem {
                        already_declared_span: already_declared_span.into(),
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
                _ => {
                    unreachable!("insert should only be called on types, functions, and interfaces")
                }
            };

            analyzer.declarations.insert(item_key, item_span, ty);
        }

        for item in items {
            match item {
                Item::Packing(packing) => {
                    analyzer.register_packing(packing);
                }
                Item::Aor(aor) => {
                    analyzer.register_aor(aor);
                }
                Item::Procedure(procedure) => {
                    analyzer.register_procedure(procedure);
                }
                Item::Methods(_methods) => {
                    todo!()
                    // analyzer.register_methods(methods);
                }
                Item::Api(api) => {
                    analyzer.register_api(api);
                }
                Item::Require(_require) => todo!(),
                Item::Get(_get) => todo!(),
                Item::Block(_) => unreachable!(),
            };
        }
    }
}
