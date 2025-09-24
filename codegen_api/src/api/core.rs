use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::{FunctionValue, PointerValue};
use std::collections::HashMap;

use crate::owner::ContextOwner;

/// Beginner-friendly facade that wraps Context/Module/Builder and provides
/// a small, easy-to-understand surface.
pub struct SimpleCodegenApi<'ctx> {
    pub(crate) ctx: &'ctx Context,
    pub(crate) module: Module<'ctx>,
    pub(crate) builder: Builder<'ctx>,
    /// If we created the Context ourselves we store an owner here so the
    /// Context is reclaimed after the Module/Builder are dropped. Placing
    /// this field after `module`/`builder` ensures its Drop runs later.
    #[allow(dead_code)]
    pub(crate) ctx_owner: Option<ContextOwner>,
    /// Named functions created in this module.
    pub(crate) functions: std::collections::HashMap<String, FunctionValue<'ctx>>,
    /// Named basic blocks, keyed by "<fn>::<block>".
    pub(crate) blocks: std::collections::HashMap<String, inkwell::basic_block::BasicBlock<'ctx>>,
    /// Current function name (if any).
    pub(crate) current_fn_name: Option<String>,
    /// Saved insert blocks stack to allow save/restore semantics.
    pub(crate) saved_blocks: Vec<inkwell::basic_block::BasicBlock<'ctx>>,
    pub(crate) current_fn: Option<FunctionValue<'ctx>>,
    pub(crate) locals: HashMap<String, PointerValue<'ctx>>,
}

impl SimpleCodegenApi<'static> {
    /// Create a fresh API instance. The Context is leaked to avoid lifetime
    /// complexity (ok for short-lived compiler runs and learning).
    pub fn new(module_name: &str) -> Self {
        // Allocate Context on the heap and keep the raw pointer so we can
        // reclaim it on Drop. We create a stable reference for Module/Builder
        // construction but avoid permanently leaking the Context.
        let boxed = Box::new(Context::create());
        let ptr = Box::into_raw(boxed);
        let ctx_ref: &'static Context = unsafe { &*ptr };
        let module = ctx_ref.create_module(module_name);
        let builder = ctx_ref.create_builder();
        Self {
            ctx: ctx_ref,
            module,
            builder,
            ctx_owner: Some(ContextOwner(ptr)),
            functions: HashMap::new(),
            blocks: HashMap::new(),
            current_fn_name: None,
            saved_blocks: Vec::new(),
            current_fn: None,
            locals: HashMap::new(),
        }
    }
}

impl<'ctx> Drop for SimpleCodegenApi<'ctx> {
    fn drop(&mut self) {
        // We intentionally do not reclaim the raw Context pointer here. The
        // `ContextOwner` field will be dropped after the Module/Builder fields
        // (due to declaration order), and its Drop impl reclaims the Context
        // safely once LLVM values have been destroyed.
    }
}
