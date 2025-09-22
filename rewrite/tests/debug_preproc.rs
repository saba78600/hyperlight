use hyperlight::*;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn debug_preproc_expand_x() {
    init_macros();
    define_macro("X", "1 + 2");
    // create token list X -> EOF
    let mut t = Token::default();
    t.kind = TokenKind::TK_IDENT;
    t.loc = Some("X".to_string());
    let t_rc = Rc::new(RefCell::new(t));
    let mut eof = Token::default();
    eof.kind = TokenKind::TK_EOF;
    let eof_rc = Rc::new(RefCell::new(eof));
    t_rc.borrow_mut().next = Some(eof_rc.clone());

    let out = preproc::preprocess(t_rc.clone()).expect("preprocess returned None");
    let mut cur_opt = Some(out);
    let mut outv = Vec::new();
    while let Some(c) = cur_opt {
        let b = c.borrow();
        if let Some(loc) = &b.loc { outv.push(format!("{:?}:{}", b.kind, loc)); }
        if b.kind == TokenKind::TK_EOF { break; }
        cur_opt = b.next.clone();
    }
    println!("expanded tokens: {:?}", outv);
}
