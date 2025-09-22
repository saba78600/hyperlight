#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    TY_VOID,
    TY_BOOL,
    TY_CHAR,
    TY_SHORT,
    TY_INT,
    TY_LONG,
    TY_FLOAT,
    TY_DOUBLE,
    TY_LDOUBLE,
    TY_ENUM,
    TY_PTR,
    TY_FUNC,
    TY_ARRAY,
    TY_VLA,
    TY_STRUCT,
    TY_UNION,
}
