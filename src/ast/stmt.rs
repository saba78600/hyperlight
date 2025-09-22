use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let { name: String, ty: Option<super::Type>, value: Expr },
    Assign { name: String, value: Expr },
    If { cond: Expr, then_block: Vec<Stmt>, else_block: Option<Vec<Stmt>> },
    While { cond: Expr, body: Vec<Stmt> },
    Expr(Expr),
}
