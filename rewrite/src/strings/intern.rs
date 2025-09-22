use std::sync::{Arc, Mutex, OnceLock};
use crate::hashmap::HashMap;

static INTERNER: OnceLock<Mutex<HashMap<Arc<String>>>> = OnceLock::new();

fn init_intern() -> Mutex<HashMap<Arc<String>>> {
    Mutex::new(HashMap::new())
}

pub fn intern(s: &str) -> Arc<String> {
    let map = INTERNER.get_or_init(|| init_intern());
    let mut guard = map.lock().unwrap();
    if let Some(rc) = guard.get(s) {
        return rc;
    }
    let rc = Arc::new(s.to_string());
    guard.put(s, rc.clone());
    rc
}

pub fn intern_owned(s: String) -> Arc<String> {
    intern(&s)
}
