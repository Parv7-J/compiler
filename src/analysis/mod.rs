//TODO: right now duplicate declarations are cooking us, as the analyzer is thinking there is only
//one declaration while doing symbol resolution
//i.e it doesnt understand the diff between a same named field in two diff structs of the same name,
//and a same named field in the same struct -> as symbols are being ided by sid, iid, did
//maybe the fix is to have the struct store a mapping of iid -> vec<sid>
//not a big bug but needs to be fixed

use crate::parser::ast::*;
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

//packing Foo {a(i32), b(Bar)} -> no need for the span of 'packing', we need the span of 'Foo', we
//need the span of 'a' and 'b', and we also need the span of 'i32', 'Bar' -> empty () will be caught
//at parsetime, and any errors like undelimited ), or ident missing etc. etc. will be also caught at
//parse time. so we now dont need spans of anything else as they are not necessary. But do we need
//spans of keywords like i32 and Foo??? at parse time we remove the spans of things like delims,
//puncts, etc. but we do track the spans of ty using spannedty and of idents using spannedident, so
//lets keep the spans intact
//packing Bar {brr(u32)}

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
            let item_span = item.span();
            let item_iid = analyzer.idents.insert(item_span);

            let is_ty = if matches!(item, Item::Packing(_) | Item::Aor(_)) {
                IsTy::Yes
            } else {
                IsTy::No
            };

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

            analyzer
                .declarations
                .insert(item_key, analyzer.scope, item_span, is_ty);
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
                Item::Methods(methods) => {
                    todo!()
                    // analyzer.register_methods(methods);
                }
                Item::Api(_api) => todo!(),
                Item::Require(_require) => todo!(),
                Item::Get(_get) => todo!(),
                Item::Block(_) => unreachable!(),
            };
        }
    }
}
