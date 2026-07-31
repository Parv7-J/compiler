use crate::{
    analysis::analyzer::Analyzer,
    parser::ast::{Atom, Expr},
};

impl Analyzer<'_> {
    pub fn expr(&mut self, expr: &Expr) {
        #[allow(unused)]
        match expr {
            Expr::Atom(atom) => todo!(),
            Expr::Prefix { op, operand } => todo!(),
            Expr::Infix { op, lhs, rhs } => todo!(),
            Expr::List(expr_list) => todo!(),
            Expr::Call {
                function,
                arguments,
            } => todo!(),
            Expr::Access { lhs, rhs } => todo!(),
        }
    }

    pub fn atom(&mut self, atom: &Atom) {
        match atom {
            Atom::String(_span) => todo!(),
            Atom::Number(_span) => todo!(),
            Atom::Boolean(_spanned_boolean) => todo!(),
            Atom::This(_span) => todo!(),
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

// pub enum Ty {
//     String,
//     Number,
// }
