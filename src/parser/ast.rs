use crate::lexer::token::*;

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    pub items: Vec<Item>,
    pub input: &'a str,
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

impl Item {
    pub fn span(&self) -> Span {
        match self {
            Item::Packing(packing) => packing.ident.0,
            Item::Aor(aor) => aor.ident.0,
            Item::Procedure(procedure) => procedure.ident.0,
            Item::Methods(methods) => methods.ident.0,
            Item::Api(api) => api.ident.0,
            Item::Require(require) => require.ident.0,
            Item::Get(get) => get.module.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block(pub Vec<BlockItem>);

#[derive(Debug, Clone)]
pub enum BlockItem {
    Item(Item),
    Stmt(Stmt),
}
#[derive(Debug, Clone)]
pub enum Expr {
    Atom(Atom),
    Cons(SpannedOperator, Vec<Expr>),
    List(Vec<Expr>),
    Call(Vec<Expr>),
    Access(Vec<Expr>),
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
    Boolean(SpannedBoolean),
}

#[derive(Debug, Clone)]
pub struct SpannedBoolean {
    pub boolean: Boolean,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Declaration {
        ty: IdentTy,
        var: SpannedIdent,
        value: Expr,
    },
    Seed {
        return_value: Expr,
    },
    Conditional {
        ifs: Conditional,
        elseifs: Vec<Conditional>,
        elses: Option<Block>,
    },
    For {
        ty: SpannedTy,
        ident: SpannedIdent,
        range: Option<(Expr, Expr, Option<Expr>)>,
        inn: Option<SpannedIdent>,
        block: Block,
    },
    While {
        condition: Expr,
        block: Block,
    },
    Break(Span),
    Continue(Span),
    Expr(Expr),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpannedTy {
    pub ty: Ty,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Conditional {
    pub condition: Expr,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpannedIdent(pub Span);

#[derive(Debug, Clone)]
pub struct LiteralString(pub Span);

#[derive(Debug, Clone)]
pub struct Get {
    pub imports: Vec<SpannedIdent>,
    pub module: LiteralString,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdentTy {
    Type(SpannedTy),
    Ident(SpannedIdent),
    Arr(SpannedArr),
    Ptr(SpannedPtr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedPtr {
    pub ptr: Span,
    pub ty: Box<IdentTy>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedArr {
    pub arr_ty: ArrType,
    pub inner_ty: Box<IdentTy>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ArrType {
    Arr,
    HeapArr,
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
