use super::types::BackendState;
use crate::ast::Stmt;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

pub mod expr;
pub mod stmt;

pub use expr::lower_expr;
pub use stmt::lower_stmt;

pub struct Backend {
    pub state: BackendState,
}

impl Backend {
    pub fn new(name: &str) -> Self {
        let api = codegen_api::SimpleCodegenApi::new(name);
        let state = BackendState {
            api,
            var_kinds: HashMap::new(),
        };
        Self { state }
    }

    pub fn compile_to_ir(&mut self, stmts: &[Stmt]) -> Result<String> {
        self.state.api.create_entry();
        for s in stmts {
            self.codegen_stmt(s)?;
        }
        let zero = self.state.api.const_i64(0);
        let _ = self.state.api.build_return(&zero);
        Ok(self.state.api.emit_ir())
    }

    pub fn emit_object(&self, out: &Path) -> Result<()> {
        self.state.api.emit_object_for_path(out)
    }

    fn codegen_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        crate::codegen::backend::lower::lower_stmt(&mut self.state, stmt)
    }
}
