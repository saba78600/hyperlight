pub mod env;
pub mod check;

pub use env::{TypeEnv, TypeError};
pub use check::check;
