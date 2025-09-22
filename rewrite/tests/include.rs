use hyperlight::*;
use std::fs::File as StdFile;
use std::io::Write;
use std::rc::Rc;
use std::cell::RefCell;
// use the crate File type via fully-qualified name to avoid name clash with std::fs::File

#[test]
fn include_basic_file() {
    // create a temporary file in the current directory
    let name = "tmp_included_test.h";
    let mut f = StdFile::create(name).expect("create temp include file");
    // contents that will tokenize into an identifier and a number
    writeln!(f, "// header comment").expect("write header comment");
    writeln!(f, "FOO 42").expect("write contents");
    // source that includes that file
    let src = format!("#include \"{}\"\nFOO;", name);
    let vf = hyperlight::File { name: None, file_no: 0, contents: Some(src.clone()), display_name: None, line_delta: 0 };
    let toks = tokenize(Rc::new(RefCell::new(vf))).expect("tokenize source");
    let res = preproc::preprocess2(toks).expect("preprocess2");
    // collect token locs
    let mut cur_opt = Some(res);
    let mut found = Vec::new();
    while let Some(c) = cur_opt {
        let b = c.borrow();
        if let Some(loc) = &b.loc { found.push(loc.clone()); }
        if b.kind == TokenKind::TK_EOF { break; }
        cur_opt = b.next.clone();
    }
    // cleanup
    let _ = std::fs::remove_file(name);
    // expect to find FOO and 42 among tokens
    let joined = found.join(" ");
    assert!(joined.contains("FOO") || joined.contains("42"));
}
