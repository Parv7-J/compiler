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
    parser::ast::{Atom, Expr, IdentTy, SpannedIdent, Stmt},
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

    fn find_symbol(&self, iid: IdentId) -> Option<SymbolId> {
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

    fn atom(&mut self, atom: &Atom) {
        match atom {
            Atom::String(span) => todo!(),
            Atom::Number(span) => todo!(),
            Atom::Boolean(spanned_boolean) => todo!(),
            Atom::This(span) => todo!(),
            Atom::Ident(spanned_ident) => {
                //here we have an ident
                let span = spanned_ident.0;
                let iid = self.idents.insert(span);
                //check if the thing has a symbol
                let sid = self.find_symbol(iid).expect("suppose we find the symbol");
                //we need to get the latest value
                let symbol = self.symbols.mutable_info(sid);
                //symbols have a type and a value?
                println!("symbolinfo: {symbol:?}");
                //for ex: u32 a = 1;
                //i32 b = a + 2;
                //infix expr -> op = +, lhs = a, rhs = 2
                //type of b is u32 here
                //it is possible to assign expr to u32?
                //boils down to if lhs + rhs can be assigned to u32
                //possible if lhs and rhs sum to a good type, and sum between them is allowed
                //the allowance check is not part of the assignment check i suppose, this allows for
                //seperate concerns
                //lets first work on the allowance ones

                //we must go bottom up?
                // basic things
                // prefix op atom
                // infix atom op atom
                // 1. 'op' should be applicable on 'atom'
                // ex: !"hello" -> doesnt mean anything
                // ex: !1 -> works, but its a bitwise not => evaluates to a number
                // !true -> works, but its a logical not => evaluates to a boolean
                // Operator::Not | Operator::Minus | Operator::Star | Operator::BitwiseAnd => Some(((), 17)),
                // minus -> works only on numbers
                // first lets talk about parsing a number and assigning its type
                // -122 -> i8, as its the lowest possible type, where all the higher types are valid
                // assignments too
                // 144 -> not i16, but u8
                //order -> i8 < u8 < i16 < u16 < i32 < u32
                // a negative number only looks for the i types
                // a positive looks for both

                // -100  * 135 -> what type?
                //assign acc to the previous rule
                // -100 is i8, 135 is u8 -> i8 * u8 will fit in i16? thats the main question, it
                // will certainly fit in u16 as u8 * u8
                // an i8 & u8 -> max is -128 * 255 .. 127 * 255
                // -128 is 127 in 2's compl -> 2^n - number
                // 256 - 128 = 128 -> 10000000 * 11111111
                // 0111111110000000 -> 32640 in binary

                // #[derive(Debug, Clone)]
                // pub enum Expr {
                //     Atom(Atom),
                //     Prefix {
                //         op: SpannedOperator,
                //         operand: Box<Expr>,
                //     },
                //     Infix {
                //         op: SpannedOperator,
                //         lhs: Box<Expr>,
                //         rhs: Box<Expr>,
                //     },
                //     List(ExprList),
                //     Call {
                //         function: Box<Expr>,
                //         arguments: ExprList,
                //     },
                //     Access {
                //         lhs: Box<Expr>,
                //         rhs: Box<Expr>,
                //     },
                // }
                //
                // #[derive(Debug, Clone)]
                // pub struct ExprList(pub Vec<Expr>);
                //
                // #[derive(Debug, Clone)]
                // pub struct SpannedOperator {
                //     pub op: Operator,
                //     pub span: Span,
                // }
                //
                // #[derive(Debug, Clone)]
                // pub enum Atom {
                //     String(Span),
                //     Ident(SpannedIdent),
                //     Number(Span),
                //     Boolean(SpannedBoolean),
                //     This(Span),
                // }
                //
                // #[derive(Debug, Clone)]
                // pub struct SpannedBoolean {
                //     pub boolean: Boolean,
                //     pub span: Span,
                // }

                //expr
            }
        }
    }
}
