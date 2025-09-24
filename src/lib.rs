//! Hyperlight language core: tokenizer (lexer), AST, and parser.
//!
//! This crate intentionally provides only front-end components (no
//! codegen or interpreter yet). Syntax is externalized into `.syntax`
//! files under the `syntaxes/` directory so they can be edited without
//! recompiling the parser code.

pub mod ast;
pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod syntax;
pub mod token;
pub mod typecheck;

pub use lexer::tokenize;
pub use parser::*;
pub use typecheck::TypeEnv;
