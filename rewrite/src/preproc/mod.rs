use crate::hashmap::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Macro {
    pub body: String,
    pub params: Option<Vec<String>>,
}

static MACROS: OnceLock<Mutex<HashMap<Macro>>> = OnceLock::new();

fn init_map() -> Mutex<HashMap<Macro>> {
    Mutex::new(HashMap::new())
}

pub fn init_macros() {
    MACROS.get_or_init(|| init_map());
}

pub fn define_macro(name: &str, body: &str) {
    let m = MACROS.get_or_init(|| init_map());
    let mut guard = m.lock().unwrap();
    guard.put(name, Macro { body: body.to_string(), params: None });
}

pub fn define_function_macro(name: &str, params: Vec<String>, body: &str) {
    let m = MACROS.get_or_init(|| init_map());
    let mut guard = m.lock().unwrap();
    guard.put(name, Macro { body: body.to_string(), params: Some(params) });
}

pub fn undef_macro(name: &str) {
    if let Some(m) = MACROS.get() {
        let mut guard = m.lock().unwrap();
        guard.delete(name);
    }
}

pub fn get_macro(name: &str) -> Option<Macro> {
    if let Some(m) = MACROS.get() {
        let guard = m.lock().unwrap();
        guard.get(name)
    } else {
        None
    }
}

// pragma_once bookkeeping: filename -> bool
static PRAGMA_ONCE: OnceLock<Mutex<HashMap<bool>>> = OnceLock::new();

pub fn pragma_once_get(path: &str) -> bool {
    if let Some(m) = PRAGMA_ONCE.get() {
        let g = m.lock().unwrap();
        g.get(path).is_some()
    } else { false }
}

pub fn pragma_once_put(path: &str) {
    let m = PRAGMA_ONCE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut g = m.lock().unwrap();
    g.put(path, true);
}

// include guards: path -> guard macro name
static INCLUDE_GUARDS: OnceLock<Mutex<HashMap<String>>> = OnceLock::new();

pub fn include_guard_get(path: &str) -> Option<String> {
    if let Some(m) = INCLUDE_GUARDS.get() {
        let g = m.lock().unwrap();
        g.get(path)
    } else { None }
}

pub fn include_guard_put(path: &str, guard: &str) {
    let m = INCLUDE_GUARDS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut g = m.lock().unwrap();
    g.put(path, guard.to_string());
}
// Builtin dynamic macro handlers are implemented in engine.rs. Expose a small
// registration helper to allow engine to register builtins with names.
pub fn add_builtin(name: &str, body: &str) {
    define_macro(name, body);
}
mod engine;
pub use engine::preprocess;
pub use engine::preprocess2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn define_and_get() {
        init_macros();
        define_macro("FOO", "42");
        let v = get_macro("FOO");
        assert_eq!(v.map(|m| m.body), Some("42".to_string()));
        undef_macro("FOO");
        assert_eq!(get_macro("FOO"), None);
    }
}
