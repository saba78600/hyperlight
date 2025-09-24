use crate::ast::Expr;
use crate::codegen::backend::types::BackendState;
use anyhow::Result;

pub fn lower_expr(
    state: &mut BackendState,
    expr: &Expr,
) -> Result<codegen_api::SimpleValue<'static>> {
    match expr {
        Expr::Number(n) => match n {
            crate::token::NumberLit::Int(i) => Ok(state.api.const_i64(*i as i64)),
            crate::token::NumberLit::Float(f) => Ok(state.api.const_f64(*f)),
        },
        Expr::Bool(b) => Ok(state.api.const_i64(if *b { 1 } else { 0 })),
        Expr::Ident(s) => {
            let kind = *state
                .var_kinds
                .get(s)
                .ok_or_else(|| anyhow::anyhow!("unknown ident"))?;
            match kind {
                crate::codegen::backend::types::VarKind::Int => Ok(state.api.load_local_i64(s)?),
                crate::codegen::backend::types::VarKind::Float => {
                    Ok(state.api.load_local_f64(s)?)
                }
            }
        }
        Expr::Call { callee, args } => {
            if callee == "print" {
                if args.len() != 1 {
                    return Err(anyhow::anyhow!("print expects 1 arg"));
                }
                let aval = lower_expr(state, &args[0])?;
                return state.api.call_printf(&aval);
            }
            let mut vals = Vec::new();
            for a in args {
                vals.push(lower_expr(state, a)?);
            }
            let arg_refs: Vec<&codegen_api::SimpleValue<'static>> = vals.iter().collect();
            let rv = state.api.build_call(callee, &arg_refs)?;
            Ok(rv)
        }
        Expr::Binary { op, left, right } => {
            let l = lower_expr(state, left)?;
            let r = lower_expr(state, right)?;
            // map AST BinOp to codegen_api::Op at the compiler level
            let cop = match op {
                crate::ast::BinOp::Add => codegen_api::Op::Add,
                crate::ast::BinOp::Sub => codegen_api::Op::Sub,
                crate::ast::BinOp::Mul => codegen_api::Op::Mul,
                crate::ast::BinOp::Div => codegen_api::Op::Div,
                crate::ast::BinOp::Mod => codegen_api::Op::Mod,
                crate::ast::BinOp::Eq => codegen_api::Op::Eq,
                crate::ast::BinOp::Ne => codegen_api::Op::Ne,
                crate::ast::BinOp::Lt => codegen_api::Op::Lt,
                crate::ast::BinOp::Le => codegen_api::Op::Le,
                crate::ast::BinOp::Gt => codegen_api::Op::Gt,
                crate::ast::BinOp::Ge => codegen_api::Op::Ge,
            };
            let res = state.api.build_binop(cop, &l, &r)?;
            Ok(res)
        }
    }
}
