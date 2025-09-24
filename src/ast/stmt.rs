use crate::ast::Expr;

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<super::Type>,
        value: Expr,
    },
    Assign {
        name: String,
        value: Expr,
    },
    /// Function definition: fn name(params) { body }
    FnDef {
        name: String,
        params: Vec<(String, Option<super::Type>)>,
        body: Vec<Stmt>,
    },
    /// Return from a function with an optional expression
    Return(Option<Expr>),
    If {
        cond: Expr,
        then_block: Vec<Stmt>,
        else_block: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Expr(Expr),
}
