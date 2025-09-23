use anyhow::Result;
use crate::ast::Stmt as S;
use crate::codegen::backend::lower::expr::lower_expr;
use crate::codegen::backend::types::BackendState;
use inkwell::values::BasicValueEnum;
use crate::codegen::backend::types::VarKind;

pub fn lower_stmt<'ctx>(state: &mut BackendState<'ctx>, stmt: &S) -> Result<()> {
    match stmt {
        S::Let { name, ty: _ty, value } => {
            let val = lower_expr(state, value)?;
            let function = state
                .builder
                .get_insert_block()
                .unwrap()
                .get_parent()
                .unwrap();
            let entry = function.get_first_basic_block().unwrap();
            let tmp = state.ctx.create_builder();
            if let Some(first) = entry.get_first_instruction() {
                tmp.position_before(&first);
            } else {
                tmp.position_at_end(entry);
            }
            let (alloca, kind) = match val {
                BasicValueEnum::IntValue(_) => (tmp.build_alloca(state.ctx.i64_type(), name)?, VarKind::Int),
                BasicValueEnum::FloatValue(_) => (tmp.build_alloca(state.ctx.f64_type(), name)?, VarKind::Float),
                _ => return Err(anyhow::anyhow!("unsupported let initializer type")),
            };
            state.builder.build_store(alloca, val)?;
            state.locals.insert(name.clone(), (alloca, kind));
            Ok(())
        }
        S::Assign { name, value } => {
            let val = lower_expr(state, value)?;
            let (ptr, kind) = *state.locals.get(name).ok_or_else(|| anyhow::anyhow!("unknown local"))?;
            match kind { VarKind::Int | VarKind::Float => { let _ = state.builder.build_store(ptr, val)?; } }
            Ok(())
        }
        S::If { cond, then_block, else_block } => {
            let parent = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let then_bb = state.ctx.append_basic_block(parent, "then");
            let else_bb = state.ctx.append_basic_block(parent, "else");
            let cont_bb = state.ctx.append_basic_block(parent, "ifcont");
            let condv = lower_expr(state, cond)?;
            let cond_bool = match condv {
                BasicValueEnum::IntValue(i) => { let zero = i.get_type().const_int(0, false); state.builder.build_int_compare(inkwell::IntPredicate::NE, i, zero, "tobool")? }
                BasicValueEnum::FloatValue(fv) => { let zerof = fv.get_type().const_float(0.0); state.builder.build_float_compare(inkwell::FloatPredicate::ONE, fv, zerof, "tobool")? }
                _ => return Err(anyhow::anyhow!("condition must be numeric")),
            };
            state.builder.build_conditional_branch(cond_bool, then_bb, else_bb)?;
            state.builder.position_at_end(then_bb); for s in then_block { lower_stmt(state, s)?; } if state.builder.get_insert_block().unwrap().get_terminator().is_none() { state.builder.build_unconditional_branch(cont_bb)?; }
            state.builder.position_at_end(else_bb); if let Some(eb) = else_block { for s in eb { lower_stmt(state, s)?; } } if state.builder.get_insert_block().unwrap().get_terminator().is_none() { state.builder.build_unconditional_branch(cont_bb)?; }
            state.builder.position_at_end(cont_bb);
            Ok(())
        }
        S::FnDef { name, params, body } => {
            let mut param_types = Vec::new();
            for (_pname, pty) in params.iter() { match pty { Some(crate::ast::Type::Float) => param_types.push(state.ctx.f64_type().into()), _ => param_types.push(state.ctx.i64_type().into()), } }
            let fn_type = state.ctx.i64_type().fn_type(&param_types, false);
            let function = state.module.add_function(name, fn_type, None);
            let bb = state.ctx.append_basic_block(function, "entry");
            let saved_block = state.builder.get_insert_block(); state.builder.position_at_end(bb);
            for (i, (pname, pty)) in params.iter().enumerate() {
                let pv = function.get_nth_param(i as u32).unwrap();
                let (alloca, kind) = match pty { Some(crate::ast::Type::Float) => (state.ctx.create_builder().build_alloca(state.ctx.f64_type(), pname)?, VarKind::Float), _ => (state.ctx.create_builder().build_alloca(state.ctx.i64_type(), pname)?, VarKind::Int), };
                let _ = state.builder.build_store(alloca, pv);
                state.locals.insert(pname.clone(), (alloca, kind));
            }
            for s in body { lower_stmt(state, s)?; }
            if state.builder.get_insert_block().unwrap().get_terminator().is_none() { let z = state.ctx.i64_type().const_int(0, false); state.builder.build_return(Some(&z))?; }
            if let Some(b) = saved_block { state.builder.position_at_end(b); }
            Ok(())
        }
        S::Return(opt) => {
            if opt.is_none() { let z = state.ctx.i64_type().const_int(0, false); state.builder.build_return(Some(&z))?; return Ok(()); }
            let expr = opt.as_ref().unwrap(); let v = lower_expr(state, expr)?;
            match v { BasicValueEnum::IntValue(iv) => { let it = iv.get_type(); if it.get_bit_width() < 64 { let sext = state.builder.build_int_s_extend(iv, state.ctx.i64_type(), "ret_ext")?; state.builder.build_return(Some(&sext))?; } else { state.builder.build_return(Some(&iv))?; } Ok(()) }, BasicValueEnum::FloatValue(fv) => { let conv = state.builder.build_float_to_signed_int(fv, state.ctx.i64_type(), "fptosi")?; state.builder.build_return(Some(&conv))?; Ok(()) }, _ => Err(anyhow::anyhow!("unsupported return value type")), }
        }
        S::While { cond, body } => {
            let parent = state.builder.get_insert_block().unwrap().get_parent().unwrap();
            let loop_bb = state.ctx.append_basic_block(parent, "loop");
            let body_bb = state.ctx.append_basic_block(parent, "loopbody");
            let cont_bb = state.ctx.append_basic_block(parent, "loopcont");
            state.builder.build_unconditional_branch(loop_bb)?; state.builder.position_at_end(loop_bb);
            let condv = lower_expr(state, cond)?;
            let cond_bool = match condv { BasicValueEnum::IntValue(i) => { let zero = i.get_type().const_int(0, false); state.builder.build_int_compare(inkwell::IntPredicate::NE, i, zero, "tobool")? } BasicValueEnum::FloatValue(fv) => { let zerof = fv.get_type().const_float(0.0); state.builder.build_float_compare(inkwell::FloatPredicate::ONE, fv, zerof, "tobool")? } _ => return Err(anyhow::anyhow!("condition must be numeric")), };
            state.builder.build_conditional_branch(cond_bool, body_bb, cont_bb)?;
            state.builder.position_at_end(body_bb); for s in body { lower_stmt(state, s)?; } if state.builder.get_insert_block().unwrap().get_terminator().is_none() { state.builder.build_unconditional_branch(loop_bb)?; }
            state.builder.position_at_end(cont_bb);
            Ok(())
        }
        S::Expr(e) => { let _ = lower_expr(state, e)?; Ok(()) }
    }
}
