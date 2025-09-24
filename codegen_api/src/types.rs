use inkwell::values::BasicValueEnum;

#[derive(Clone, Copy, Debug)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub struct SimpleValue<'ctx> {
    inner: BasicValueEnum<'ctx>,
}

impl<'ctx> SimpleValue<'ctx> {
    pub fn from_basic(b: BasicValueEnum<'ctx>) -> Self {
        Self { inner: b }
    }
    pub fn as_basic(&self) -> BasicValueEnum<'ctx> {
        self.inner
    }
}
