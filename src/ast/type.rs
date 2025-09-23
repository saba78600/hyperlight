#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    UInt,
    Float,
    Bool,
    Void,
    Custom(String),
}
