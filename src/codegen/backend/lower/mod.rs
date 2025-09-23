use super::types::BackendState;
use crate::ast::Stmt;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub mod stmt;
pub mod expr;

pub use stmt::lower_stmt;
pub use expr::lower_expr;

pub struct Backend<'ctx> {
    pub state: BackendState<'ctx>,
}

impl<'ctx> Backend<'ctx> {
    pub fn new(ctx: &'ctx inkwell::context::Context, name: &str) -> Self {
        let module = ctx.create_module(name);
        let builder = ctx.create_builder();
        let state = BackendState {
            ctx,
            module,
            builder,
            locals: HashMap::new(),
        };
        Self { state }
    }

    pub fn compile_to_ir(&mut self, stmts: &[Stmt]) -> Result<String> {
        // create main
        let i64_type = self.state.ctx.i64_type();
        let fn_type = i64_type.fn_type(&[], false);
        let function = self.state.module.add_function("main", fn_type, None);
        let bb = self.state.ctx.append_basic_block(function, "entry");
        self.state.builder.position_at_end(bb);

        for s in stmts {
            self.codegen_stmt(s)?;
        }

        let zero = i64_type.const_int(0, false);
        self.state.builder.build_return(Some(&zero))?;
        Ok(self.state.module.print_to_string().to_string())
    }

    pub fn emit_object(&self, out: &Path) -> Result<()> {
        crate::codegen::backend::emitter::emit_object_for_module(&self.state, out)
    }

    fn codegen_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        crate::codegen::backend::lower::lower_stmt(&mut self.state, stmt)
    }
}
