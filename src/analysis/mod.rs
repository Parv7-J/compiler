use crate::parser::ast::*;
use std::sync::Arc;

mod analyzer;
mod error;
mod recurse;
mod stmt;
mod store;

use analyzer::*;
use error::*;
use store::*;

pub struct SemanticAnalysis<'a> {
    ast: Ast<'a>,
    analyzer: Analyzer<'a>,
}

impl<'a> SemanticAnalysis<'a> {
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

    pub fn analyze(mut self) -> (Ast<'a>, Analyzer<'a>) {
        let items = self.ast.items.iter().collect::<Vec<_>>();
        self.analyzer.collect(&items);
        self.analyzer.recurse(&items);

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

        (self.ast, self.analyzer)
    }
}
