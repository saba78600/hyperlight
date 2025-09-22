//! Crate root for the Hyperlight rewrite.
#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub mod strings;
pub mod token;
pub mod types;
pub mod ast;
pub mod hashmap;

pub use strings::string_array::StringArray;
pub use token::token::Token;
pub use token::token::TokenKind;
pub use token::file::File;
pub use types::r#type::Type;
pub use types::type_kind::TypeKind;
pub use ast::node::Node;
pub use ast::node_kind::NodeKind;
pub use ast::obj::Obj;
pub use ast::member::Member;
pub use ast::relocation::Relocation;
pub use token::tokenize as ctokenize;

use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

// Function stubs (kept at crate root for easy global access)
pub fn error(fmt_str: &str, _args: fmt::Arguments) -> ! {
    // Print to stderr then exit the process with a failure code.
    // We avoid unwind-style panic to keep behavior deterministic during translation.
    use std::io::Write;
    let _ = writeln!(std::io::stderr(), "error: {}", fmt_str);
    std::process::exit(1)
}
pub fn error_at(loc: &str, fmt: &str, _args: fmt::Arguments) -> ! {
    use std::io::Write;
    let _ = writeln!(std::io::stderr(), "{}: error: {}", loc, fmt);
    std::process::exit(1)
}

pub fn error_tok(tok: &Token, fmt: &str, _args: fmt::Arguments) -> ! {
    use std::io::Write;
    if let Some(filename) = &tok.filename {
        let _ = writeln!(std::io::stderr(), "{}:{}: error: {}", filename, tok.line_no, fmt);
    } else if let Some(file) = &tok.file {
        let borrow = file.borrow();
        if let Some(name) = &borrow.name {
            let _ = writeln!(std::io::stderr(), "{}:{}: error: {}", name, tok.line_no, fmt);
        } else {
            let _ = writeln!(std::io::stderr(), "<input>:{}: error: {}", tok.line_no, fmt);
        }
    } else {
        let _ = writeln!(std::io::stderr(), "<unknown>:{}: error: {}", tok.line_no, fmt);
    }
    std::process::exit(1)
}

pub fn warn_tok(_tok: &Token, _fmt: &str, _args: fmt::Arguments) {
    use std::io::Write;
    let _ = writeln!(std::io::stderr(), "warning: {}", _fmt);
}

pub fn equal(_tok: &Token, _op: &str) -> bool {
    false
}

pub fn skip(_tok: &Token, _op: &str) -> Option<std::rc::Rc<std::cell::RefCell<Token>>> {
    None
}

pub fn consume(_tok: &mut Option<Rc<RefCell<Token>>>, _str: &str) -> bool {
    false
}

pub fn convert_pp_tokens(_tok: Rc<RefCell<Token>>) {
}

pub fn get_input_files() -> Vec<Rc<RefCell<File>>> {
    Vec::new()
}

pub fn new_file(_name: &str, _file_no: i32, _contents: &str) -> File {
    File::default()
}

pub fn tokenize_string_literal(_tok: Rc<RefCell<Token>>, _basety: Rc<RefCell<Type>>) -> Option<Rc<RefCell<Token>>> {
    None
}

pub fn tokenize(file: Rc<RefCell<File>>) -> Option<Rc<RefCell<Token>>> {
    ctokenize(file)
}

pub fn tokenize_file(filename: &str) -> Option<Rc<RefCell<Token>>> {
    // Create a File wrapper and read contents from disk.
    use std::fs;
    match fs::read_to_string(filename) {
        Ok(contents) => {
            let f = File { name: Some(filename.to_string()), file_no: 0, contents: Some(contents), display_name: Some(filename.to_string()), line_delta: 0 };
            tokenize(Rc::new(RefCell::new(f)))
        }
        Err(_) => None,
    }
}

// preprocess

pub fn search_include_paths(_filename: &str) -> Option<String> {
    None
}

pub fn init_macros() {
}

pub fn define_macro(_name: &str, _buf: &str) {
}

pub fn undef_macro(_name: &str) {
}

pub fn preprocess(_tok: Rc<RefCell<Token>>) -> Option<Rc<RefCell<Token>>> {
    None
}

// parse

pub fn new_cast(_expr: Rc<RefCell<Node>>, _ty: Rc<RefCell<Type>>) -> Rc<RefCell<Node>> {
    Rc::new(RefCell::new(Node::default()))
}

pub fn const_expr(_rest: &mut Option<Rc<RefCell<Token>>>, _tok: Rc<RefCell<Token>>) -> i64 {
    0
}

pub fn parse(_tok: Rc<RefCell<Token>>) -> Option<Rc<RefCell<Obj>>> {
    None
}

// type helpers

pub fn is_integer(ty: Rc<RefCell<Type>>) -> bool { types::r#type::is_integer(ty) }
pub fn is_flonum(ty: Rc<RefCell<Type>>) -> bool { types::r#type::is_flonum(ty) }
pub fn is_numeric(ty: Rc<RefCell<Type>>) -> bool { types::r#type::is_numeric(ty) }
pub fn is_compatible(t1: Rc<RefCell<Type>>, t2: Rc<RefCell<Type>>) -> bool {
    // Recursive compatibility check inspired by common C rules.
    // Covers: identical types, typedef origins, pointers (including void*),
    // arrays, functions (return type + params), structs/unions (by name or members),
    // and numeric types.

    fn compat(a: &Rc<RefCell<Type>>, b: &Rc<RefCell<Type>>) -> bool {
        // Pointer equality fast-path
        if std::rc::Rc::ptr_eq(a, b) {
            return true;
        }

        // Follow typedef origins: if either has an origin, compare that origin
        if let Some(orig) = &a.borrow().origin {
            if compat(orig, b) {
                return true;
            }
        }
        if let Some(orig) = &b.borrow().origin {
            if compat(a, orig) {
                return true;
            }
        }

        let ka = a.borrow().kind;
        let kb = b.borrow().kind;

        if ka == kb {
            match ka {
                TypeKind::TY_PTR => {
                    let ba = a.borrow().base.clone();
                    let bb = b.borrow().base.clone();
                    // If either pointer has no base information, assume compatible
                    if ba.is_none() || bb.is_none() {
                        return true;
                    }
                    let ba = ba.unwrap();
                    let bb = bb.unwrap();
                    // void * is compatible with any object pointer
                    if ba.borrow().kind == TypeKind::TY_VOID || bb.borrow().kind == TypeKind::TY_VOID {
                        return true;
                    }
                    return compat(&ba, &bb);
                }
                TypeKind::TY_ARRAY | TypeKind::TY_VLA => {
                    let ba = a.borrow().base.clone();
                    let bb = b.borrow().base.clone();
                    if ba.is_none() || bb.is_none() {
                        return false;
                    }
                    let ba = ba.unwrap();
                    let bb = bb.unwrap();
                    if !compat(&ba, &bb) {
                        return false;
                    }
                    // If lengths are identical or one is flexible/unspecified, accept
                    let la = a.borrow().array_len;
                    let lb = b.borrow().array_len;
                    if la == lb || a.borrow().is_flexible || b.borrow().is_flexible {
                        return true;
                    }
                    return false;
                }
                TypeKind::TY_FUNC => {
                    // Compare return types
                    let ra = a.borrow().return_ty.clone();
                    let rb = b.borrow().return_ty.clone();
                    if ra.is_none() || rb.is_none() {
                        return false;
                    }
                    if !compat(&ra.unwrap(), &rb.unwrap()) {
                        return false;
                    }
                    // Compare params list
                    let mut pa = a.borrow().params.clone();
                    let mut pb = b.borrow().params.clone();
                    while pa.is_some() && pb.is_some() {
                        let aa = pa.take().unwrap();
                        let bb = pb.take().unwrap();
                        if !compat(&aa, &bb) {
                            return false;
                        }
                        pa = aa.borrow().next.clone();
                        pb = bb.borrow().next.clone();
                    }
                    // If one has extra parameters, require both to be variadic
                    if pa.is_some() || pb.is_some() {
                        return a.borrow().is_variadic && b.borrow().is_variadic;
                    }
                    return true;
                }
                TypeKind::TY_STRUCT | TypeKind::TY_UNION => {
                    // Prefer named compatibility
                    let na = a.borrow().name.clone();
                    let nb = b.borrow().name.clone();
                    if let (Some(sa), Some(sb)) = (na, nb) {
                        return sa == sb;
                    }
                    // Fall back to members pointer equality
                    if let (Some(ma), Some(mb)) = (a.borrow().members.clone(), b.borrow().members.clone()) {
                        return std::rc::Rc::ptr_eq(&ma, &mb);
                    }
                    return false;
                }
                _ => {
                    // For scalar/numeric/basic types: require same unsignedness and size
                    let a_b = a.borrow();
                    let b_b = b.borrow();
                    return a_b.is_unsigned == b_b.is_unsigned && a_b.size == b_b.size;
                }
            }
        }

        // Different kinds: allow numeric compatibility (int/float families)
        if types::r#type::is_numeric(a.clone()) && types::r#type::is_numeric(b.clone()) {
            return true;
        }

        false
    }

    compat(&t1, &t2)
}
pub fn copy_type(ty: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> { types::r#type::copy_type(ty) }
pub fn pointer_to(base: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> { types::r#type::pointer_to(base) }
pub fn func_type(return_ty: Rc<RefCell<Type>>) -> Rc<RefCell<Type>> { types::r#type::func_type(return_ty) }
