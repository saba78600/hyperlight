use crate::ast::Stmt as S;
use crate::codegen::backend::lower::expr::lower_expr;
use crate::codegen::backend::types::BackendState;
use crate::codegen::backend::types::VarKind;
use anyhow::Result;

pub fn lower_stmt(state: &mut BackendState, stmt: &S) -> Result<()> {
    match stmt {
        S::Let {
            name,
            ty: _ty,
            value,
        } => {
            let val = lower_expr(state, value)?;
            // allocate in entry via API
            match val.as_basic() {
                inkwell::values::BasicValueEnum::IntValue(_) => {
                    state.api.alloc_local_i64(name, Some(&val))?;
                    state.var_kinds.insert(name.clone(), VarKind::Int);
                }
                inkwell::values::BasicValueEnum::FloatValue(_) => {
                    state.api.alloc_local_f64(name, Some(&val))?;
                    state.var_kinds.insert(name.clone(), VarKind::Float);
                }
                _ => return Err(anyhow::anyhow!("unsupported let initializer type")),
            }
            Ok(())
        }
        S::Assign { name, value } => {
            let val = lower_expr(state, value)?;
            match state
                .var_kinds
                .get(name)
                .ok_or_else(|| anyhow::anyhow!("unknown local"))?
            {
                VarKind::Int => {
                    state.api.store_local_i64(name, &val)?;
                }
                VarKind::Float => {
                    state.api.store_local_f64(name, &val)?;
                }
            }
            Ok(())
        }
        S::If {
            cond,
            then_block,
            else_block,
        } => {
            // create blocks
            state.api.append_basic_block("then")?;
            state.api.append_basic_block("else")?;
            state.api.append_basic_block("ifcont")?;
            let condv = lower_expr(state, cond)?;
            state.api.build_conditional_branch(&condv, "then", "else")?;
            // then
            state.api.position_at_end("then")?;
            for s in then_block {
                lower_stmt(state, s)?;
            }
            if !state.api.current_block_has_terminator() {
                state.api.build_unconditional_branch("ifcont")?;
            }
            // else
            state.api.position_at_end("else")?;
            if let Some(eb) = else_block {
                for s in eb {
                    lower_stmt(state, s)?;
                }
            }
            if !state.api.current_block_has_terminator() {
                state.api.build_unconditional_branch("ifcont")?;
            }
            // cont
            state.api.position_at_end("ifcont")?;
            Ok(())
        }
        S::FnDef { name, params, body } => {
            let mut param_is_float = Vec::new();
            for (_pname, pty) in params.iter() {
                match pty {
                    Some(crate::ast::Type::Float) => param_is_float.push(true),
                    _ => param_is_float.push(false),
                }
            }
            state.api.add_function(name, &param_is_float)?;
            state.api.set_current_function(name)?;
            state.api.append_basic_block("entry")?;
            state.api.position_at_end("entry")?;
            // initialize params as locals
            for (i, (pname, pty)) in params.iter().enumerate() {
                // allocate and store param
                let is_float = matches!(pty, Some(crate::ast::Type::Float));
                if is_float {
                    state.api.alloc_local_f64(pname, None)?;
                    state.var_kinds.insert(pname.clone(), VarKind::Float);
                } else {
                    state.api.alloc_local_i64(pname, None)?;
                    state.var_kinds.insert(pname.clone(), VarKind::Int);
                }
                // store nth parameter into the local via API helper
                state.api.store_param_into_local(name, i as u32, pname)?;
            }
            for s in body {
                lower_stmt(state, s)?;
            }
            if !state.api.current_block_has_terminator() {
                let z = state.api.const_i64(0);
                state.api.build_return_i64(&z)?;
            }
            Ok(())
        }
        S::Return(opt) => {
            if opt.is_none() {
                let z = state.api.const_i64(0);
                state.api.build_return(&z)?;
                return Ok(());
            }
            let expr = opt.as_ref().unwrap();
            let v = lower_expr(state, expr)?;
            state.api.build_return(&v)?;
            Ok(())
        }
        S::While { cond, body } => {
            state.api.append_basic_block("loop")?;
            state.api.append_basic_block("loopbody")?;
            state.api.append_basic_block("loopcont")?;
            state.api.build_unconditional_branch("loop")?;
            state.api.position_at_end("loop")?;
            let condv = lower_expr(state, cond)?;
            state
                .api
                .build_conditional_branch(&condv, "loopbody", "loopcont")?;
            state.api.position_at_end("loopbody")?;
            for s in body {
                lower_stmt(state, s)?;
            }
            if !state.api.current_block_has_terminator() {
                state.api.build_unconditional_branch("loop")?;
            }
            state.api.position_at_end("loopcont")?;
            Ok(())
        }
        S::Expr(e) => {
            let _ = lower_expr(state, e)?;
            Ok(())
        }
    }
}
