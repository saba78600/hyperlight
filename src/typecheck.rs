use crate::ast::{Expr, Stmt, Type};

#[derive(Debug)]
pub enum TypeError {
    UnknownIdentifier(String),
    Mismatch { expected: Type, found: Type },
}

pub struct TypeEnv {
    vars: std::collections::HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self { vars: Default::default() }
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }
}

pub fn check(stmts: &[Stmt]) -> Result<(), TypeError> {
    let mut env = TypeEnv::new();
    for s in stmts {
        match s {
            Stmt::Let { name, ty, value } => {
                let val_ty = infer_type(value, &env)?;
                if let Some(expected) = ty {
                    if !is_assignable(&val_ty, expected) {
                        return Err(TypeError::Mismatch { expected: expected.clone(), found: val_ty });
                    }
                    env.insert(name.clone(), expected.clone());
                } else {
                    env.insert(name.clone(), val_ty);
                }
            }
            Stmt::Assign { name, value } => {
                let val_ty = infer_type(value, &env)?;
                let var_ty = env.get(name).ok_or(TypeError::UnknownIdentifier(name.clone()))?.clone();
                if !is_assignable(&val_ty, &var_ty) {
                    return Err(TypeError::Mismatch { expected: var_ty, found: val_ty });
                }
            }
            Stmt::If { cond, then_block, else_block } => {
                let cty = infer_type(cond, &env)?;
                if cty != Type::Bool { return Err(TypeError::Mismatch { expected: Type::Bool, found: cty }); }
                check(then_block)?;
                if let Some(eb) = else_block { check(eb)?; }
            }
            Stmt::While { cond, body } => {
                let cty = infer_type(cond, &env)?;
                if cty != Type::Bool { return Err(TypeError::Mismatch { expected: Type::Bool, found: cty }); }
                check(body)?;
            }
            Stmt::Expr(e) => { infer_type(e, &env)?; }
        }
    }
    Ok(())
}

fn infer_type(expr: &Expr, env: &TypeEnv) -> Result<Type, TypeError> {
    match expr {
        Expr::Number(n) => match n {
            crate::token::NumberLit::Int(_) => Ok(Type::Int),
            crate::token::NumberLit::Float(_) => Ok(Type::Float),
        },
        Expr::Bool(_) => Ok(Type::Bool),
        Expr::Ident(name) => env.get(name).cloned().ok_or(TypeError::UnknownIdentifier(name.clone())),
        Expr::Binary { op, left, right } => {
            let lt = infer_type(left, env)?;
            let rt = infer_type(right, env)?;
            use crate::ast::BinOp;
            match op {
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div => {
                    // allow int/int -> int, float involvement -> float
                    match (lt, rt) {
                        (Type::Int, Type::Int) => Ok(Type::Int),
                        (Type::Float, Type::Float) => Ok(Type::Float),
                        (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Float),
                        (a, b) => Err(TypeError::Mismatch { expected: a, found: b }),
                    }
                }
                BinOp::Mod => {
                    // modulo only defined for integers
                    match (lt, rt) {
                        (Type::Int, Type::Int) => Ok(Type::Int),
                        (a, b) => Err(TypeError::Mismatch { expected: Type::Int, found: if a != Type::Int { a } else { b } }),
                    }
                }
                BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    // comparisons require matching numeric types (int/uint/float) and return Bool
                    match (lt, rt) {
                        (Type::Int, Type::Int) | (Type::UInt, Type::UInt) | (Type::Float, Type::Float) | (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Type::Bool),
                        (a, b) => Err(TypeError::Mismatch { expected: a, found: b }),
                    }
                }
            }
        }
    }
}

fn is_assignable(src: &Type, dst: &Type) -> bool {
    match (src, dst) {
        (Type::Int, Type::Int) => true,
        (Type::UInt, Type::UInt) => true,
        (Type::Float, Type::Float) => true,
        (Type::Int, Type::Float) | (Type::UInt, Type::Float) => true,
        (a, b) => a == b,
    }
}
