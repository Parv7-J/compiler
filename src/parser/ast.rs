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
    Cons(SpannedOperator, Vec<S>),
}

#[derive(Debug, Clone)]
pub struct SpannedOperator {
    pub op: Operator,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Atom {
    String(Span),
    Ident(SpannedIdent),
    Number(Span),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Declaration {
        ty: IdentTy,
        var: SpannedIdent,
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
        ty: SpannedTy,
        ident: SpannedIdent,
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
pub struct SpannedTy {
    pub ty: Ty,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: S,
    pub block: Block,
}

#[derive(Debug, Clone)]
pub struct Packing {
    pub ident: SpannedIdent,
    pub fields: Vec<Field>,
}
#[derive(Debug, Clone)]
pub struct Aor {
    pub ident: SpannedIdent,
    pub variants: Vec<Variant>,
}

#[derive(Debug, Clone)]
pub enum Variant {
    Field(Field),
    SpannedIdent(SpannedIdent),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub ident: SpannedIdent,
    pub ty: IdentTy,
}

#[derive(Debug, Clone)]
pub struct SpannedIdent {
    pub id: usize,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct LiteralString(pub Span);

#[derive(Debug, Clone)]
pub struct Get {
    pub imports: Vec<SpannedIdent>,
    pub module: LiteralString,
}

#[derive(Debug, Clone)]
pub enum IdentTy {
    Type(SpannedTy),
    Ident(SpannedIdent),
    Arr(SpannedArr),
    Ptr(SpannedPtr),
}

#[derive(Debug, Clone)]
pub struct SpannedPtr {
    pub ptr: Span,
    pub ty: Box<IdentTy>,
}

#[derive(Debug, Clone)]
pub struct SpannedArr {
    pub ty: Ty,
    pub inner_ty: Box<IdentTy>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub ident: SpannedIdent,
    pub args: Vec<Field>,
    pub return_value: Option<IdentTy>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct Methods {
    pub ident: SpannedIdent,
    pub procedures: Vec<Procedure>,
}

#[derive(Debug, Clone)]
pub struct Api {
    pub ident: SpannedIdent,
    pub super_api: Vec<SpannedIdent>,
    pub procedures: Vec<Procedure>,
}

#[derive(Debug, Clone)]
pub struct Require {
    pub ident: SpannedIdent,
    pub api: SpannedIdent,
    pub procedures: Vec<Procedure>,
}
