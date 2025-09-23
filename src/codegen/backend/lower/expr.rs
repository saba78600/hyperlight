use anyhow::Result;
use crate::codegen::backend::types::BackendState;
use crate::ast::Expr;
use inkwell::values::BasicValueEnum;

pub fn lower_expr<'ctx>(state: &BackendState<'ctx>, expr: &Expr) -> Result<BasicValueEnum<'ctx>> {
    match expr {
        Expr::Number(n) => match n {
            crate::token::NumberLit::Int(i) => Ok(state.ctx.i64_type().const_int(*i as u64, true).into()),
            crate::token::NumberLit::Float(f) => Ok(state.ctx.f64_type().const_float(*f).into()),
        },
        Expr::Bool(b) => Ok(state.ctx.i64_type().const_int(if *b { 1 } else { 0 }, false).into()),
        Expr::Ident(s) => {
            let (ptr, kind) = *state.locals.get(s).ok_or_else(|| anyhow::anyhow!("unknown ident"))?;
            match kind {
                crate::codegen::backend::types::VarKind::Int => {
                    let loaded = state.builder.build_load(state.ctx.i64_type(), ptr, &format!("load_{}", s))?;
                    Ok(loaded.into())
                }
                crate::codegen::backend::types::VarKind::Float => {
                    let loaded = state.builder.build_load(state.ctx.f64_type(), ptr, &format!("load_{}", s))?;
                    Ok(loaded.into())
                }
            }
        }
        Expr::Call { callee, args } => {
            if callee == "print" {
                if args.len() != 1 { return Err(anyhow::anyhow!("print expects 1 arg")); }
                let aval = lower_expr(state, &args[0])?;
                let i32_t = state.ctx.i32_type();
                let printf_ty = i32_t.fn_type(&[state.ctx.ptr_type(inkwell::AddressSpace::default()).into()], true);
                let printf_fn = match state.module.get_function("printf") { Some(f) => f, None => state.module.add_function("printf", printf_ty, None), };
                match aval {
                    BasicValueEnum::IntValue(iv) => {
                        let fmt_g = state.builder.build_global_string_ptr("%lld\n", "fmt_i64")?;
                        let fmt_ptr = fmt_g.as_pointer_value();
                        let _ = state.builder.build_call(printf_fn, &[fmt_ptr.into(), iv.into()], "call_printf");
                        return Ok(state.ctx.i64_type().const_int(0, false).into());
                    }
                    BasicValueEnum::FloatValue(fv) => {
                        let fmt_g = state.builder.build_global_string_ptr("%f\n", "fmt_f64")?;
                        let fmt_ptr = fmt_g.as_pointer_value();
                        let _ = state.builder.build_call(printf_fn, &[fmt_ptr.into(), fv.into()], "call_printf");
                        return Ok(state.ctx.i64_type().const_int(0, false).into());
                    }
                    _ => return Err(anyhow::anyhow!("print: unsupported argument type")),
                }
            }
            let mut vals = Vec::new();
            for a in args { vals.push(lower_expr(state, a)?); }
            let function = match state.module.get_function(callee) { Some(f) => f, None => return Err(anyhow::anyhow!("unknown function: {}", callee)), };
            use inkwell::values::BasicMetadataValueEnum;
            let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
            for (i, pv) in function.get_param_iter().enumerate() {
                let expected = pv.get_type();
                let provided = vals.get(i).ok_or_else(|| anyhow::anyhow!("too few args for call"))?;
                let arg_val: BasicMetadataValueEnum<'ctx> = match (provided, expected) {
                    (&BasicValueEnum::IntValue(iv), t) if t.is_int_type() => iv.into(),
                    (&BasicValueEnum::FloatValue(fv), t) if t.is_float_type() => fv.into(),
                    (&BasicValueEnum::IntValue(iv), t) if t.is_float_type() => {
                        let conv = state.builder.build_signed_int_to_float(iv, state.ctx.f64_type(), "sitofp")?;
                        conv.into()
                    }
                    (&BasicValueEnum::FloatValue(_fv), t) if t.is_int_type() => {
                        return Err(anyhow::anyhow!("cannot coerce float to int for call arg"));
                    }
                    _ => return Err(anyhow::anyhow!("unsupported arg coercion")),
                };
                call_args.push(arg_val);
            }
            let call_site = state.builder.build_call(function, &call_args, "calltmp")?;
            if let Some(rv) = call_site.try_as_basic_value().left() { Ok(rv) } else { Ok(state.ctx.i64_type().const_int(0, false).into()) }
        }
        Expr::Binary { op, left, right } => {
            let l = lower_expr(state, left)?;
            let r = lower_expr(state, right)?;
            let res = crate::codegen::api::CodegenApi::binop_with_builder(state.ctx, &state.builder, op.clone(), l, r)?;
            Ok(res)
        }
    }
}
