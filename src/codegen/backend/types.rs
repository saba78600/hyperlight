use codegen_api::SimpleCodegenApi;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug)]
pub enum VarKind {
    Int,
    Float,
}

pub struct BackendState {
    /// The compiler-facing, Inkwell-encapsulating API instance (owns its Context).
    pub api: SimpleCodegenApi<'static>,
    /// Track variable kinds (int vs float) for coercions and allocations.
    pub var_kinds: HashMap<String, VarKind>,
}
