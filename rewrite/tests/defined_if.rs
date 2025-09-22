use hyperlight::token::token::TokenKind;
use hyperlight::tokenize as tokenize_fn;
use std::rc::Rc;
use std::cell::RefCell;
use hyperlight::preprocess as preprocess_fn;
use hyperlight::init_macros;
use hyperlight::define_macro;

#[test]
fn test_defined_function_form() {
    // Define macro M, then test `#if defined(M)` includes block
    init_macros();
    define_macro("M", "1");
    let src = "#if defined(M)\nX\n#endif\n";
    let file = hyperlight::File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 };
    let toks = tokenize_fn(Rc::new(RefCell::new(file))).unwrap();
    let out = preprocess_fn(toks).unwrap();
    // Collect tokens to string
    let mut s = String::new();
    let mut cur = Some(out);
    while let Some(t) = cur {
        if t.borrow().kind == TokenKind::TK_EOF { break; }
        if let Some(loc) = &t.borrow().loc { s.push_str(loc); }
        cur = t.borrow().next.clone();
    }
    assert!(s.contains("X"));
}

#[test]
fn test_defined_keyword_form() {
    init_macros();
    define_macro("N", "1");
    let src = "#if defined N\nY\n#endif\n";
    let file = hyperlight::File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 };
    let toks = tokenize_fn(Rc::new(RefCell::new(file))).unwrap();
    let out = preprocess_fn(toks).unwrap();
    let mut s = String::new();
    let mut cur = Some(out);
    while let Some(t) = cur {
        if t.borrow().kind == TokenKind::TK_EOF { break; }
        if let Some(loc) = &t.borrow().loc { s.push_str(loc); }
        cur = t.borrow().next.clone();
    }
    assert!(s.contains("Y"));
}

#[test]
fn test_defined_undefined_case() {
    init_macros();
    let src = "#if defined(UNDEF)\nZ\n#else\nW\n#endif\n";
    let file = hyperlight::File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 };
    let toks = tokenize_fn(Rc::new(RefCell::new(file))).unwrap();
    let out = preprocess_fn(toks).unwrap();
    let mut s = String::new();
    let mut cur = Some(out);
    while let Some(t) = cur {
        if t.borrow().kind == TokenKind::TK_EOF { break; }
        if let Some(loc) = &t.borrow().loc { s.push_str(loc); }
        cur = t.borrow().next.clone();
    }
    assert!(s.contains("W"));
}
