use crate::token;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(token::NumberLit),
    Ident(String),
    Bool(bool),
    Binary {
        op: super::BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}
