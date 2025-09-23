use inkwell::values::PointerValue;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum VarKind {
    Int,
    Float,
}

pub struct BackendState<'ctx> {
    pub ctx: &'ctx inkwell::context::Context,
    pub module: inkwell::module::Module<'ctx>,
    pub builder: inkwell::builder::Builder<'ctx>,
    /// symbol table: local variable name -> (alloca pointer, kind)
    pub locals: HashMap<String, (PointerValue<'ctx>, VarKind)>,
}
