#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(crate::token::NumberLit),
    Ident(String),
    Bool(bool),
    Binary {
        op: BinOp,
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

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, ty: Option<Type>, value: Expr },
    Assign { name: String, value: Expr },
    If { cond: Expr, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>> },
    While { cond: Expr, body: Vec<Stmt> },
    Expr(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    UInt,
    Float,
    Bool,
    Custom(String),
}
