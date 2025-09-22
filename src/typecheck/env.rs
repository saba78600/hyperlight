use crate::ast::Type;

#[derive(Debug)]
pub enum TypeError {
    UnknownIdentifier(String),
    Mismatch { expected: Type, found: Type },
}

pub struct TypeEnv {
    pub(crate) vars: std::collections::HashMap<String, Type>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self { vars: Default::default() }
    }

    pub fn insert(&mut self, name: String, ty: Type) {
        self.vars.insert(name, ty);
    }

    pub fn get(&self, name: &str) -> Option<&Type> {
        self.vars.get(name)
    }
}
