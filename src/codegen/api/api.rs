use anyhow::Result;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{BasicValueEnum, FunctionValue, PointerValue, IntValue, FloatValue};
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use std::collections::HashMap;

/// High-level wrapper that hides Inkwell details and provides simple helpers.
pub struct CodegenApi<'ctx> {
    pub ctx: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    current_fn: Option<FunctionValue<'ctx>>,
    locals: HashMap<String, PointerValue<'ctx>>,
}

impl CodegenApi<'static> {
    /// Create a new CodegenApi bound to a leaked Context (lives for program lifetime).
    /// This avoids lifetime gymnastics and is fine for short-lived compiler runs.
    pub fn new(module_name: &str) -> Self {
        // leak the Context so we can store references with a stable lifetime
        let boxed = Box::leak(Box::new(Context::create()));
        let module = boxed.create_module(module_name);
        let builder = boxed.create_builder();
        Self { ctx: boxed, module, builder, current_fn: None, locals: HashMap::new() }
    }
}

impl<'ctx> CodegenApi<'ctx> {
    /// Create (or replace) an entry function named `main` that returns i64 and has no params.
    pub fn create_entry(&mut self) -> FunctionValue<'ctx> {
        let i64t = self.ctx.i64_type();
        let fn_type = i64t.fn_type(&[], false);
        let function = self.module.add_function("main", fn_type, None);
        let bb = self.ctx.append_basic_block(function, "entry");
        self.builder.position_at_end(bb);
        self.current_fn = Some(function);
        function
    }

    /// Allocate an i64 local (alloca in entry block) and optionally initialize it.
    pub fn alloc_local_i64(&mut self, name: &str, init: Option<IntValue<'ctx>>) -> Result<PointerValue<'ctx>> {
        let function = self.current_fn.expect("no current function; call create_entry first");
        let entry = function.get_first_basic_block().unwrap();
        let tmp = self.ctx.create_builder();
        if let Some(first) = entry.get_first_instruction() {
            tmp.position_before(&first);
        } else {
            tmp.position_at_end(entry);
        }
        let alloca = tmp.build_alloca(self.ctx.i64_type(), name)?;
        if let Some(iv) = init {
            self.builder.build_store(alloca, iv)?;
        }
        self.locals.insert(name.to_string(), alloca);
        Ok(alloca)
    }

    /// Store an i64 value into a previously allocated local.
    pub fn store_local_i64(&mut self, name: &str, val: IntValue<'ctx>) -> Result<()> {
        let ptr = *self.locals.get(name).ok_or_else(|| anyhow::anyhow!("unknown local"))?;
        self.builder.build_store(ptr, val)?;
        Ok(())
    }

    /// Load an i64 local and return the IntValue.
    pub fn load_local_i64(&mut self, name: &str) -> Result<IntValue<'ctx>> {
        let ptr = *self.locals.get(name).ok_or_else(|| anyhow::anyhow!("unknown local"))?;
        let loaded = self.builder.build_load(self.ctx.i64_type(), ptr, &format!("load_{}", name))?;
        Ok(loaded.into_int_value())
    }

    /// Convenience: create an i64 constant
    pub fn const_i64(&self, v: i64) -> IntValue<'ctx> {
        self.ctx.i64_type().const_int(v as u64, true)
    }

    /// Convenience: create an f64 constant
    pub fn const_f64(&self, v: f64) -> FloatValue<'ctx> {
        self.ctx.f64_type().const_float(v)
    }

    /// Build integer binary operation (arithmetic or comparison). Returns a BasicValueEnum.
    fn int_predicate_for(op: &crate::ast::BinOp) -> Option<IntPredicate> {
        match op {
            crate::ast::BinOp::Eq => Some(IntPredicate::EQ),
            crate::ast::BinOp::Ne => Some(IntPredicate::NE),
            crate::ast::BinOp::Lt => Some(IntPredicate::SLT),
            crate::ast::BinOp::Le => Some(IntPredicate::SLE),
            crate::ast::BinOp::Gt => Some(IntPredicate::SGT),
            crate::ast::BinOp::Ge => Some(IntPredicate::SGE),
            _ => None,
        }
    }

    /// Build float comparison predicate mapper
    fn float_predicate_for(op: &crate::ast::BinOp) -> Option<FloatPredicate> {
        match op {
            crate::ast::BinOp::Eq => Some(FloatPredicate::OEQ),
            crate::ast::BinOp::Ne => Some(FloatPredicate::ONE),
            crate::ast::BinOp::Lt => Some(FloatPredicate::OLT),
            crate::ast::BinOp::Le => Some(FloatPredicate::OLE),
            crate::ast::BinOp::Gt => Some(FloatPredicate::OGT),
            crate::ast::BinOp::Ge => Some(FloatPredicate::OGE),
            _ => None,
        }
    }

    /// Generic int op builder that returns a BasicValueEnum
    fn build_int_binop(&mut self, op: crate::ast::BinOp, a: IntValue<'ctx>, b: IntValue<'ctx>) -> Result<BasicValueEnum<'ctx>> {
        if let Some(pred) = Self::int_predicate_for(&op) {
            let cmp = self.builder.build_int_compare(pred, a, b, "cmptmp")?;
            let zext = self.builder.build_int_z_extend(cmp, self.ctx.i64_type(), "bool_to_i64")?;
            return Ok(zext.into());
        }

        let val = match op {
            crate::ast::BinOp::Add => self.builder.build_int_add(a, b, "addtmp")?,
            crate::ast::BinOp::Sub => self.builder.build_int_sub(a, b, "subtmp")?,
            crate::ast::BinOp::Mul => self.builder.build_int_mul(a, b, "multmp")?,
            crate::ast::BinOp::Div => self.builder.build_int_signed_div(a, b, "divtmp")?,
            crate::ast::BinOp::Mod => self.builder.build_int_signed_rem(a, b, "modtmp")?,
            _ => return Err(anyhow::anyhow!("unsupported int op")),
        };
        Ok(val.into())
    }

    /// Convert int to float
    pub fn sitofp(&mut self, v: IntValue<'ctx>) -> Result<FloatValue<'ctx>> {
        Ok(self.builder.build_signed_int_to_float(v, self.ctx.f64_type(), "sitofp")?)
    }

    /// Build float binary op — returns a BasicValueEnum (either float or extended i64 for comparisons)
    fn build_float_binop(&mut self, op: crate::ast::BinOp, a: FloatValue<'ctx>, b: FloatValue<'ctx>) -> Result<BasicValueEnum<'ctx>> {
        if let Some(pred) = Self::float_predicate_for(&op) {
            let cmp = self.builder.build_float_compare(pred, a, b, "fcmp")?;
            let zext = self.builder.build_int_z_extend(cmp, self.ctx.i64_type(), "bool_to_i64")?;
            return Ok(zext.into());
        }

        match op {
            crate::ast::BinOp::Add => {
                let r = self.builder.build_float_add(a, b, "faddtmp")?;
                Ok(r.into())
            }
            crate::ast::BinOp::Sub => {
                let r = self.builder.build_float_sub(a, b, "fsubtmp")?;
                Ok(r.into())
            }
            crate::ast::BinOp::Mul => {
                let r = self.builder.build_float_mul(a, b, "fmultmp")?;
                Ok(r.into())
            }
            crate::ast::BinOp::Div => {
                let r = self.builder.build_float_div(a, b, "fdivtmp")?;
                Ok(r.into())
            }
            _ => Err(anyhow::anyhow!("unsupported float op")),
        }
    }

    /// Generic binary operator that accepts two BasicValueEnum operands.
    /// It dispatches to int or float implementations and performs simple coercions
    /// (int -> float) when operands differ.
    pub fn build_binop(&mut self, op: crate::ast::BinOp, a: BasicValueEnum<'ctx>, b: BasicValueEnum<'ctx>) -> Result<BasicValueEnum<'ctx>> {
        match (a, b) {
            (BasicValueEnum::IntValue(ai), BasicValueEnum::IntValue(bi)) => self.build_int_binop(op, ai, bi),
            (BasicValueEnum::FloatValue(af), BasicValueEnum::FloatValue(bf)) => self.build_float_binop(op, af, bf),
            // mixed: int -> float coercion
            (BasicValueEnum::IntValue(ai), BasicValueEnum::FloatValue(bf)) => {
                let af = self.builder.build_signed_int_to_float(ai, self.ctx.f64_type(), "sitofp")?;
                self.build_float_binop(op, af, bf)
            }
            (BasicValueEnum::FloatValue(af), BasicValueEnum::IntValue(bi)) => {
                let bf = self.builder.build_signed_int_to_float(bi, self.ctx.f64_type(), "sitofp")?;
                self.build_float_binop(op, af, bf)
            }
            // other kinds are not supported by this simple API
            _ => Err(anyhow::anyhow!("unsupported operand types for build_binop")),
        }
    }

    /// A free helper that performs the same binop lowering but accepts an explicit
    /// Context and Builder. This makes it easy for other modules (like the Backend)
    /// to reuse the same logic without having to construct a full CodegenApi instance.
    pub fn binop_with_builder<'b>(ctx: &'b Context, builder: &Builder<'b>, op: crate::ast::BinOp, a: BasicValueEnum<'b>, b: BasicValueEnum<'b>) -> Result<BasicValueEnum<'b>> {
        // we recreate minimal logic from the instance methods but using supplied ctx/builder
        use inkwell::values::BasicValueEnum as BVE;
        match (a, b) {
            (BVE::IntValue(ai), BVE::IntValue(bi)) => {
                if let Some(pred) = CodegenApi::int_predicate_for(&op) {
                    let cmp = builder.build_int_compare(pred, ai, bi, "cmptmp")?;
                    let zext = builder.build_int_z_extend(cmp, ctx.i64_type(), "bool_to_i64")?;
                    return Ok(zext.into());
                }
                let val = match op {
                    crate::ast::BinOp::Add => builder.build_int_add(ai, bi, "addtmp")?,
                    crate::ast::BinOp::Sub => builder.build_int_sub(ai, bi, "subtmp")?,
                    crate::ast::BinOp::Mul => builder.build_int_mul(ai, bi, "multmp")?,
                    crate::ast::BinOp::Div => builder.build_int_signed_div(ai, bi, "divtmp")?,
                    crate::ast::BinOp::Mod => builder.build_int_signed_rem(ai, bi, "modtmp")?,
                    _ => return Err(anyhow::anyhow!("unsupported int op")),
                };
                Ok(val.into())
            }
            (BVE::FloatValue(af), BVE::FloatValue(bf)) => {
                if let Some(pred) = CodegenApi::float_predicate_for(&op) {
                    let cmp = builder.build_float_compare(pred, af, bf, "fcmp")?;
                    let zext = builder.build_int_z_extend(cmp, ctx.i64_type(), "bool_to_i64")?;
                    return Ok(zext.into());
                }
                let val = match op {
                    crate::ast::BinOp::Add => builder.build_float_add(af, bf, "faddtmp")?,
                    crate::ast::BinOp::Sub => builder.build_float_sub(af, bf, "fsubtmp")?,
                    crate::ast::BinOp::Mul => builder.build_float_mul(af, bf, "fmultmp")?,
                    crate::ast::BinOp::Div => builder.build_float_div(af, bf, "fdivtmp")?,
                    _ => return Err(anyhow::anyhow!("unsupported float op")),
                };
                Ok(val.into())
            }
            (BVE::IntValue(ai), BVE::FloatValue(bf)) => {
                let af = builder.build_signed_int_to_float(ai, ctx.f64_type(), "sitofp")?;
                CodegenApi::binop_with_builder(ctx, builder, op, af.into(), bf.into())
            }
            (BVE::FloatValue(af), BVE::IntValue(bi)) => {
                let bf = builder.build_signed_int_to_float(bi, ctx.f64_type(), "sitofp")?;
                CodegenApi::binop_with_builder(ctx, builder, op, af.into(), bf.into())
            }
            _ => Err(anyhow::anyhow!("unsupported operand types for binop_with_builder")),
        }
    }

    /// Finish by returning an i64 from current function
    pub fn build_return_i64(&mut self, v: IntValue<'ctx>) -> Result<()> {
        self.builder.build_return(Some(&v))?;
        Ok(())
    }

    /// Produce the textual IR
    pub fn emit_ir(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Write IR to file
    pub fn write_ir_to(&self, path: &std::path::Path) -> Result<()> {
        std::fs::write(path, self.emit_ir())?;
        Ok(())
    }
}
