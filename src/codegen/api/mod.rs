pub mod api;
pub mod io;

pub use api::CodegenApi;
pub use io::{compile_stmts_to_ir, compile_and_write_ir, compile_and_link_executable};
