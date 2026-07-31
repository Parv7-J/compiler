//!start understanding stmts
//!```rust
//! pub struct Block(pub Vec<BlockItem>);
//!
//! pub enum BlockItem {
//!     Item(Item),
//!     Stmt(Stmt),
//! }
//!
//declaration -> declare a symbol 'var', with the type 'ty' and value 'expr'
//symbols always have a value -> but function arguments may not have a value defined yet
//value should resolve as a value of the type
//! pub enum Stmt {
//!     Declaration {
//!         ty: IdentTy,
//!         var: SpannedIdent,
//!         value: Expr,
//!     },
//return values should have the value that evaluates to the return type
//!     Seed {
//!         return_value: Expr,
//!     },
//contionals should always evaluate to a bool
//!     If {
//!         ifs: Conditional,
//!         elseifs: Vec<Conditional>,
//!         elses: Option<Block>,
//!     },
//!     For {
//!         ty: SpannedTy,
//!         ident: SpannedIdent,
//!         range: Option<Range>,
//!         collection: Option<SpannedIdent>,
//!         block: Block,
//!     },
//!     While {
//!         condition: Expr,
//!         block: Block,
//!     },
//break only occurs inside loops
//!     Break(Span),
//continue only occurs inside loops
//!     Continue(Span),
//expression itself is not a symbol, so it can evaluate to any value, the bounds are then checked by
//the caller
//!     Expr(Expr),
//! }
//!
//we can fix this by saying, the operator can have a number of args only, instead of vec<expr>
//cons can be divided to prefix infix postfix
//call to function and arguments -> arguments is a list
//access to only left and right
//this is very much better to do -> because we are capable of doing this at parse time

//some more rules -> like what operator works on what expression, for ex && works on left and right
//evaluating to a bool
//! pub enum Expr {
//see atom
//!     Atom(Atom),
//operator should work on the expression list
//!     Cons(SpannedOperator, Vec<Expr>),
//each element of the list should be of the same type, type checking of that type is done by the
//caller, which may include conversion
//!     List(Vec<Expr>),
//function call evaluates to a type, not a value
//!     Call(Vec<Expr>),
//field access evaluates to a value
//!     Access(Vec<Expr>),
//! }
//!
//! pub struct SpannedOperator {
//!     pub op: Operator,
//!     pub span: Span,
//! }
//!
//string, number, boolean all evaluate to a value and conversion is done by the caller
//ident and this are evaluated to symbols
//! pub enum Atom {
//!     String(Span),
//!     Ident(SpannedIdent),
//!     Number(Span),
//!     Boolean(SpannedBoolean),
//!     This(Span),
//! }
//!
//!
//! pub struct Conditional {
//!     pub condition: Expr,
//!     pub block: Block,
//! }
//!
//start, end, and step should be correct semantically, i.e should be iterable, and resolve to the
//type defined in the loop
//! pub struct Range {
//!     pub start: Expr,
//!     pub end: Expr,
//!     pub step: Option<Expr>,
//! }
//!
//! pub struct SpannedBoolean {
//!     pub boolean: Boolean,
//!     pub span: Span,
//! }
//!
//! ```

//number -> depends on the other operand, and if not depends on the operator it is with
//ex: 1*-2 -> we can say 1 is not unsigned, but signed
//ex: 1 && true -> 1 becomes boolean
//should we allow implicit conversions???

use crate::{
    analysis::{
        analyzer::Analyzer,
        store::{IdentId, SymbolId, SymbolKey, SymbolType},
    },
    parser::ast::{Expr, IdentTy, SpannedIdent, Stmt},
};

#[allow(unused_variables)]
impl Analyzer<'_> {
    pub fn register_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Declaration { ty, var, value } => self.register_declaration(ty, var, value),
            Stmt::Seed { return_value } => todo!(),
            Stmt::If {
                ifs,
                elseifs,
                elses,
            } => todo!(),
            Stmt::For {
                ty,
                ident,
                range,
                collection,
                block,
            } => todo!(),
            Stmt::While { condition, block } => todo!(),
            Stmt::Break(span) => todo!(),
            Stmt::Continue(span) => todo!(),
            Stmt::Expr(expr) => todo!(),
        }
    }

    fn register_declaration(&mut self, ty: &IdentTy, var: &SpannedIdent, value: &Expr) {
        let var_type = self.symbol_type(ty);
        let var_span = var.0;
        let var_iid = self.idents.insert(var_span);
        let var_key = SymbolKey::new(self.scope, var_iid);
        let var_sid = self.symbols.insert(var_key, var_span, SymbolType::Pending);

        //now we need to modify it, we will just call a function later on

        println!("expr: {value:?}");
        match value {
            Expr::Atom(atom) => self.atom(atom),
            Expr::Prefix { op, operand } => todo!(),
            Expr::Infix { op, lhs, rhs } => todo!(),
            Expr::List(expr_list) => todo!(),
            Expr::Call {
                function,
                arguments,
            } => todo!(),
            Expr::Access { lhs, rhs } => todo!(),
        }
        //for each thing, we need value and type
        //for strings,numbers, we just need the value, as the type is inferred
        //for ident we need to check if we have a symbol in scope with that name , if yes we deliver
        //that symbol, then let the caller decide if it is okay to operate on it, for ex foo * 5 is
        //valid for us, but if foo is a function name, then its not
    }

    pub fn find_symbol(&self, iid: IdentId) -> Option<SymbolId> {
        let mut scope = self.scope;
        loop {
            let key = SymbolKey::new(scope, iid);
            match self.symbols.get(key) {
                Some(sid) => return Some(sid),
                None => {
                    let parent_scope = self.declarations.parent_scope(scope);
                    if scope == parent_scope {
                        break;
                    }
                    scope = parent_scope;
                }
            };
        }
        None
    }
}
