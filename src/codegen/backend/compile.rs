use crate::ast::Stmt;
use anyhow::Result;
use std::path::Path;

use super::lower::Backend;

/// Convenience: compile AST statements to IR using the backend
pub fn compile_stmts_to_ir(stmts: &[Stmt]) -> Result<String> {
    let mut backend = Backend::new("hyperlight_module");
    backend.compile_to_ir(stmts)
}

/// Convenience wrapper that writes IR to a path
pub fn compile_and_write_ir(stmts: &[Stmt], out_path: &Path) -> Result<()> {
    let ir = compile_stmts_to_ir(stmts)?;
    std::fs::write(out_path, ir)?;
    Ok(())
}

/// Compile statements, emit an object file, and link it into a native executable.
pub fn compile_and_link_executable(stmts: &[Stmt], out_path: &Path) -> Result<()> {
    let mut backend = Backend::new("hyperlight_module");
    let _ = backend.compile_to_ir(stmts)?;

    // emit object file
    let obj_path = out_path.with_extension("o");
    backend.emit_object(&obj_path)?;

    // link using system linker (cc)
    let exe_path = out_path;
    let status = std::process::Command::new("cc")
        .arg(&obj_path)
        .arg("-o")
        .arg(exe_path)
        .status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("linker failed with status: {}", status));
    }
    Ok(())
}
