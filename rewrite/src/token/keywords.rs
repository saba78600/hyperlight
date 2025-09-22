use crate::hashmap::HashMap;
use std::sync::{Mutex, OnceLock};

static KEYWORDS: OnceLock<Mutex<HashMap<usize>>> = OnceLock::new();

fn populate_map(map: &mut HashMap<usize>) {
    let kws = [
        "return", "if", "else", "for", "while", "int", "sizeof", "char",
        "struct", "union", "short", "long", "void", "typedef", "_Bool",
        "enum", "static", "goto", "break", "continue", "switch", "case",
        "default", "extern", "_Alignof", "_Alignas", "do", "signed",
        "unsigned", "const", "volatile", "auto", "register", "restrict",
        "__restrict", "__restrict__", "_Noreturn", "float", "double",
        "typeof", "asm", "_Thread_local", "__thread", "_Atomic",
        "__attribute__",
    ];
    for (i, &k) in kws.iter().enumerate() {
        map.put(k, i);
    }
}

fn init_keywords() {
    let mut m = HashMap::new();
    populate_map(&mut m);
    let _ = KEYWORDS.set(Mutex::new(m));
}

pub fn is_keyword(s: &str) -> bool {
    let m = KEYWORDS.get_or_init(|| {
        let mut map = HashMap::new();
        populate_map(&mut map);
        Mutex::new(map)
    });
    let guard = m.lock().unwrap();
    guard.get(s).is_some()
}
