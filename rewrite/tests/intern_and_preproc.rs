use hyperlight::*;
use std::sync::Arc;

#[test]
fn interner_roundtrip() {
    let a = strings::intern("hello");
    let b = strings::intern("hello");
    assert!(Arc::ptr_eq(&a, &b));
}

#[test]
fn preproc_define_get() {
    init_macros();
    // Ensure any prior test state is cleared for the macro name used here.
    // Some tests run in parallel; undef before define to make this test deterministic.
    let _ = preproc::undef_macro("X");
    define_macro("X", "1");
    let val = preproc::get_macro("X");
    assert_eq!(val.map(|m| m.body), Some("1".to_string()));
}

use std::sync::Once;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn simple_macro_expansion() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        init_macros();
    });

    // define macro X -> 1 + 2
    define_macro("X", "1 + 2");

    // create token list for: X ; EOF
    let mut t = Token::default();
    t.kind = TokenKind::TK_IDENT;
    t.loc = Some("X".to_string());
    let t_rc = Rc::new(RefCell::new(t));

    let mut eof = Token::default();
    eof.kind = TokenKind::TK_EOF;
    let eof_rc = Rc::new(RefCell::new(eof));
    t_rc.borrow_mut().next = Some(eof_rc.clone());

    let out = preproc::preprocess(t_rc.clone()).expect("preprocess returned None");

    // Walk output tokens and collect their printed forms
    let mut names = Vec::new();
    let mut cur = Some(out);
    while let Some(c) = cur {
        let b = c.borrow();
        match b.kind {
            TokenKind::TK_NUM | TokenKind::TK_PUNCT | TokenKind::TK_IDENT => {
                if let Some(loc) = &b.loc { names.push(loc.clone()); }
            }
            TokenKind::TK_EOF => { names.push("<eof>".to_string()); break; }
            _ => {}
        }
        cur = b.next.clone();
    }

    // Expect sequence: 1 + 2 <eof>
    assert_eq!(names, vec!["1".to_string(), "+".to_string(), "2".to_string(), "<eof>".to_string()]);
}
