//! Centralized syntax registry for keywords, types and builtins.
//!
//! This small module provides a global `Registry` (stored in a OnceLock)
//! which the lexer and other compiler phases can consult. It makes adding
//! or removing keywords and builtins easy — update `register_defaults`
//! or construct a custom `Registry` and call `register` during startup.
use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeywordKind {
    Let,
    Fn,
    Return,
    If,
    Else,
    While,
    True,
    False,
}

#[derive(Debug, Clone)]
pub struct BuiltinSig {
    pub name: String,
    pub params: usize,
    pub returns_void: bool,
}

#[derive(Debug, Clone)]
pub struct Registry {
    pub keywords: HashMap<String, KeywordKind>,
    pub types: HashSet<String>,
    pub builtins: HashMap<String, BuiltinSig>,
}

impl Registry {
    pub fn new() -> Self {
        Self { keywords: HashMap::new(), types: HashSet::new(), builtins: HashMap::new() }
    }
}

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn register(reg: Registry) {
    let _ = REGISTRY.set(reg);
}

pub fn get() -> Option<&'static Registry> {
    REGISTRY.get()
}

/// Convenience: check if a string is a registered keyword and return its kind.
pub fn is_keyword(s: &str) -> Option<KeywordKind> {
    get().and_then(|r| r.keywords.get(s).copied())
}

/// Convenience: check if a name is a registered builtin and return its signature.
pub fn get_builtin(name: &str) -> Option<BuiltinSig> {
    get().and_then(|r| r.builtins.get(name).cloned())
}

/// Convenience: check if a name is a registered type.
pub fn is_type(name: &str) -> bool {
    get().map(|r| r.types.contains(name)).unwrap_or(false)
}

pub fn register_defaults() {
    let mut r = Registry::new();
    r.keywords.insert("let".into(), KeywordKind::Let);
    r.keywords.insert("fn".into(), KeywordKind::Fn);
    r.keywords.insert("return".into(), KeywordKind::Return);
    r.keywords.insert("if".into(), KeywordKind::If);
    r.keywords.insert("else".into(), KeywordKind::Else);
    r.keywords.insert("while".into(), KeywordKind::While);
    r.keywords.insert("true".into(), KeywordKind::True);
    r.keywords.insert("false".into(), KeywordKind::False);

    r.types.insert("int".into());
    r.types.insert("uint".into());
    r.types.insert("float".into());
    r.types.insert("bool".into());

    r.builtins.insert("print".into(), BuiltinSig { name: "print".into(), params: 1, returns_void: true });

    register(r);
}
