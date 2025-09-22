use hyperlight::*;
use std::rc::Rc;
use std::cell::RefCell;

// Focused tests for stringize (#) and token-paste (##) edge cases

#[test]
fn stringize_basic() {
    init_macros();
    // M11(x) -> #x
    crate::preproc::define_function_macro("M11", vec!["x".to_string()], "#x");
    let src = "M11(a!b  `\"\"c)"; // mimics tricky tokens from C test
    let tokens = crate::token::tokenize(Rc::new(RefCell::new(crate::File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 }))).unwrap();
    let res = crate::preproc::preprocess(tokens).unwrap();
    // collect string token content
    let mut cur_opt = Some(res);
    let mut found = false;
    while let Some(cur) = cur_opt {
        let b = cur.borrow();
        if b.kind == crate::token::token::TokenKind::TK_STR {
            if let Some(s) = &b.str_lit { assert!(s.contains("a!b")); found = true; break; }
        }
        if b.kind == crate::token::token::TokenKind::TK_EOF { break; }
        cur_opt = b.next.clone();
    }
    assert!(found, "stringize did not produce a string token");
}

#[test]
fn paste_variants() {
    init_macros();
    crate::preproc::define_macro("X1", "100");
    crate::preproc::define_function_macro("paste", vec!["x".to_string(), "y".to_string()], "x##y");
    let src = "paste(1,5);";
    let tokens = crate::token::tokenize(Rc::new(RefCell::new(crate::File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 }))).unwrap();
    let res = crate::preproc::preprocess(tokens).unwrap();
    // collect textual output
    let mut s = String::new();
    let mut cur = Some(res);
    while let Some(t) = cur {
        let b = t.borrow();
        if let Some(loc) = &b.loc { s.push_str(loc); }
        if b.kind == crate::token::token::TokenKind::TK_EOF { break; }
        cur = b.next.clone();
    }
    assert!(s.contains("15"));

    // test P(X,1) -> X1 -> 100
    crate::preproc::define_function_macro("P", vec!["a".to_string(), "b".to_string()], "a ## b");
    let src2 = "P(X,1);";
    let tokens2 = crate::token::tokenize(Rc::new(RefCell::new(crate::File { name: None, file_no: 0, contents: Some(src2.to_string()), display_name: None, line_delta: 0 }))).unwrap();
    let res2 = crate::preproc::preprocess(tokens2).unwrap();
    let mut s2 = String::new();
    let mut cur2 = Some(res2);
    while let Some(t) = cur2 {
        let b = t.borrow();
        if let Some(loc) = &b.loc { s2.push_str(loc); }
        if b.kind == crate::token::token::TokenKind::TK_EOF { break; }
        cur2 = b.next.clone();
    }
    assert!(s2.contains("100"));
}
