use crate::lexer::token::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    pub items: Vec<Item>,
    pub intern: Intern<'a>,
}

#[derive(Debug, Clone)]
pub struct Intern<'a> {
    pub db: Vec<String>,
    pub ids: HashMap<&'a str, usize>,
}

impl Intern<'_> {
    pub fn new() -> Self {
        Self {
            db: Vec::new(),
            ids: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Item {
    Packing(Packing),
    Aor(Aor),
    Procedure(Procedure),
    Methods(Methods),
    Api(Api),
    Require(Require),
    Get(Get),
}

#[derive(Debug, Clone)]
pub struct Block(pub Vec<BlockItem>);

#[derive(Debug, Clone)]
pub enum BlockItem {
    Item(Item),
    Stmt(Stmt),
}
#[derive(Debug, Clone)]
pub enum S {
    Atom(Atom),
    Cons(Operator, Vec<S>),
}
#[derive(Debug, Clone)]
pub enum Atom {
    String(Span),
    Ident(usize),
    Number(Span),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Declaration {
        ty: IdentTy,
        var: Ident,
        value: S,
    },
    Seed {
        return_value: S,
    },
    Conditional {
        ifs: Conditional,
        elseifs: Vec<Conditional>,
        elses: Option<Block>,
    },
    For {
        ident: Ident,
        ty: IdentTy,
        range: Option<(S, S, Option<S>)>,
        block: Block,
    },
    While {
        condition: S,
        block: Block,
    },
    Expr(S),
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: S,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct Packing {
    pub ident: Ident,
    pub fields: Vec<Field>,
}
#[derive(Debug, Clone)]
pub struct Aor {
    pub ident: Ident,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub enum Variant {
    Field(Field),
    Ident(Ident),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub ident: Ident,
    pub ty: IdentTy,
}

#[derive(Debug, Clone)]
pub struct Ident(pub usize);

#[derive(Debug, Clone)]
pub struct LiteralString(pub Span);

#[derive(Debug, Clone)]
pub struct Get {
    pub imports: Vec<Ident>,
    pub module: LiteralString,
}

#[derive(Debug, Clone)]
pub enum IdentTy {
    Type(Ty),
    Ident(Ident),
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub ident: Ident,
    pub args: Vec<Field>,
    pub return_value: Option<IdentTy>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Methods {
    pub ident: Ident,
    pub procedures: Vec<Procedure>,
}

#[derive(Debug, Clone)]
pub struct Api {
    pub ident: Ident,
    pub super_api: Vec<Ident>,
    pub procedures: Vec<Procedure>,
}

#[derive(Debug, Clone)]
pub struct Require {
    pub ident: Ident,
    pub api: Ident,
    pub procedures: Vec<Procedure>,
}
