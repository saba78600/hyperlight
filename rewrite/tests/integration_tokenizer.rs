use hyperlight::*;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn tokenize_keywords_and_idents() {
    let src = "int foo = 42; return foo;";
    let f = File { name: Some("<test>".to_string()), file_no: 0, contents: Some(src.to_string()), display_name: Some("<test>".to_string()), line_delta: 0 };
    let head = tokenize(Rc::new(RefCell::new(f))).expect("tokenize returned tokens");
    // Walk tokens and collect kinds and text
    let mut cur = Some(head.clone());
    let mut kinds = Vec::new();
    let mut texts = Vec::new();
    while let Some(t) = cur {
        let b = t.borrow();
        kinds.push(b.kind.clone());
        texts.push(b.loc.clone().unwrap_or_default());
        cur = b.next.clone();
    }
    // Expect sequence includes INT (keyword), IDENT foo, PUNCT '=', NUM 42, PUNCT ';', KEYWORD return, IDENT foo, PUNCT ';', EOF
    assert!(kinds.contains(&crate::token::TokenKind::TK_KEYWORD));
    assert!(texts.iter().any(|s| s == "foo"));
}
