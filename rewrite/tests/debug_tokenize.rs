use hyperlight::*;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn debug_tokenize_body() {
    let src = "1 + 2";
    let f = File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 };
    let toks = tokenize(Rc::new(RefCell::new(f))).expect("tokenize failed");
    let mut cur = Some(toks);
    let mut out = Vec::new();
    while let Some(c) = cur {
        let b = c.borrow();
        if let Some(loc) = &b.loc { out.push(format!("{:?}:{}", b.kind, loc)); }
        if b.kind == TokenKind::TK_EOF { break; }
        cur = b.next.clone();
    }
    println!("tokens: {:?}", out);
}
