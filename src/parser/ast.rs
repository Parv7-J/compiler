use crate::lexer::token::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Ast<'a> {
    pub declarations: Vec<Declaration>,
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
pub enum Declaration {
    Packing(Packing),
    Aor(Aor),
    Procedure(Procedure),
    Methods(Methods),
    Api(Api),
    Require(Require),
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
pub enum IdentTy {
    Type(Ty),
    Ident(Ident),
}

#[derive(Debug, Clone)]
pub struct Procedure {
    pub ident: Ident,
    pub args: Vec<Field>,
    pub return_value: Option<IdentTy>,
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
