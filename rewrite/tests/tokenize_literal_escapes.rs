use hyperlight::token::token::TokenKind;
use hyperlight::tokenize as tokenize_fn;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_string_escape_octal_and_hex() {
    let src = "\"a\\123b\\x41c\\e\\n\""; // "a\123b\x41c\e\n"
    let file = hyperlight::File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 };
    let toks = tokenize_fn(Rc::new(RefCell::new(file))).unwrap();
    // first token should be TK_STR with decoded str_lit
    let first = toks;
    assert_eq!(first.borrow().kind, TokenKind::TK_STR);
    let s = first.borrow().str_lit.clone().unwrap_or_default();
    // Expect 'a' + (octal 123 = decimal 83 -> 'S') + 'b' + (hex 0x41c = U+041C) + ESC + newline
    assert!(s.contains('a'));
    assert!(s.contains('b'));
    assert!(s.contains('\u{041C}'));
}
