use crate::parser::ast::*;
use std::sync::Arc;

mod analyzer;
mod block;
mod error;
mod stmt;
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
        let items = self.ast.items.iter().collect::<Vec<_>>();
        self.analyzer.collect(items.as_ref());
        self.analyzer.recurse(items.as_ref());

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
}
