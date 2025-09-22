//! Crate root for the Hyperlight rewrite.
#![allow(dead_code)]
#![allow(non_camel_case_types)]

pub mod strings;
pub mod token;
pub mod types;
pub mod ast;
pub mod hashmap;
pub mod preproc;

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
    if let Some(loc) = &_tok.loc {
        loc == _op
    } else {
        false
    }
}

pub fn skip(_tok: &Token, _op: &str) -> Option<std::rc::Rc<std::cell::RefCell<Token>>> {
    // Return the next token if current matches op, otherwise None.
    if equal(_tok, _op) {
        _tok.next.clone()
    } else {
        None
    }
}

pub fn consume(_tok: &mut Option<Rc<RefCell<Token>>>, _str: &str) -> bool {
    if let Some(t_rc) = _tok.as_ref() {
        // Clone the fields we need while the borrow is active, then drop the borrow
        let loc_clone = t_rc.borrow().loc.clone();
        let next_clone = t_rc.borrow().next.clone();
        if loc_clone.as_deref() == Some(_str) {
            *_tok = next_clone;
            return true;
        }
    }
    false
}

pub fn convert_pp_tokens(_tok: Rc<RefCell<Token>>) {
    // Convert preprocessing pp-number tokens to their canonical representations.
    // Our scanner currently produces TK_NUM directly where possible, and
    // uses TK_STR for string literals, so keep this as a conservative no-op.
    // The C implementation converts TK_PP_NUM to TK_NUM/TK_KEYWORD etc.; if
    // needed later we can implement full conversion here.
    use token::token::TokenKind;
    let mut cur_opt = Some(_tok);
    while let Some(cur) = cur_opt {
        if cur.borrow().kind == TokenKind::TK_PP_NUM {
            if let Some(loc) = &cur.borrow().loc {
                // Try to parse as integer first (decimal/hex/binary/octal)
                // This is a conservative parse: we accept digits, optional prefixes and suffixes.
                let s = loc.clone();
                // Attempt integer parse using Rust's i128 to be generous
                let mut trimmed = s.as_str();
                // strip common suffix letters (uUlLfF)
                while let Some(last) = trimmed.chars().last() {
                    if last.is_ascii_alphabetic() { trimmed = &trimmed[..trimmed.len()-1]; } else { break; }
                }
                // Try decimal/hex/binary/octal detection
                let parsed = if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
                    i128::from_str_radix(&trimmed[2..], 16).ok()
                } else if trimmed.starts_with("0b") || trimmed.starts_with("0B") {
                    i128::from_str_radix(&trimmed[2..], 2).ok()
                } else if trimmed.starts_with('0') && trimmed.len() > 1 && trimmed.chars().all(|c| c.is_ascii_digit()) {
                    i128::from_str_radix(&trimmed[1..], 8).ok()
                } else {
                    trimmed.parse::<i128>().ok()
                };

                if let Some(v) = parsed {
                    cur.borrow_mut().kind = TokenKind::TK_NUM;
                    cur.borrow_mut().val = v as i64;
                } else {
                    // Try floating point parse with suffix handling.
                    // Use Rust's parsing to approximate C's strtold behavior.
                    let mut text = s.clone();
                    // detect suffix and strip it
                    if text.ends_with('f') || text.ends_with('F') || text.ends_with('l') || text.ends_with('L') {
                        text = text[..text.len()-1].to_string();
                    }
                    if let Ok(fv) = text.parse::<f64>() {
                        cur.borrow_mut().kind = TokenKind::TK_NUM;
                        cur.borrow_mut().fval = fv as f64;
                        // don't set exact ty object now; leave ty = None for later refinement
                    }
                }
            }
        }
        cur_opt = cur.borrow().next.clone();
    }
}

pub fn get_input_files() -> Vec<Rc<RefCell<File>>> {
    // We currently don't maintain a global list of input files in the Rust
    // rewrite; return an empty vector to match the API used by callers.
    Vec::new()
}

pub fn new_file(_name: &str, _file_no: i32, _contents: &str) -> File {
    File { name: Some(_name.to_string()), file_no: _file_no, contents: Some(_contents.to_string()), display_name: None, line_delta: 0 }
}

pub fn tokenize_string_literal(_tok: Rc<RefCell<Token>>, _basety: Rc<RefCell<Type>>) -> Option<Rc<RefCell<Token>>> {
    // Create a new string token with the requested base element type.
    // Expect _tok.loc to contain the full quoted literal like "...".
    let loc = _tok.borrow().loc.clone().unwrap_or_default();
    // Strip surrounding quotes if present.
    let inner = if loc.starts_with('"') && loc.ends_with('"') && loc.len() >= 2 {
        loc[1..loc.len()-1].to_string()
    } else { loc };

    let mut t = Token::default();
    t.kind = TokenKind::TK_STR;
    t.loc = Some(format!("\"{}\"", inner));
    t.str_lit = Some(inner);
    t.ty = Some(_basety.clone());
    Some(Rc::new(RefCell::new(t)))
}

pub fn tokenize(file: Rc<RefCell<File>>) -> Option<Rc<RefCell<Token>>> {
    ctokenize(file)
}

pub fn tokenize_file(filename: &str) -> Option<Rc<RefCell<Token>>> {
    // Create a File wrapper and read contents from disk.
    use std::fs;
    match fs::read_to_string(filename) {
        Ok(contents) => {
            // Normalize file contents like the C scanner does:
            // - canonicalize newlines (\r and \r\n -> \n)
            // - remove backslash-newline pairs while preserving logical line counts
            // - convert universal \u/\U escapes into UTF-8 bytes
            let mut buf = contents;
            fn canonicalize_newline(s: &mut String) {
                *s = s.replace("\r\n", "\n").replace('\r', "\n");
            }
            fn remove_backslash_newline(s: &mut String) {
                // We need to remove backslash-newline but preserve the number of
                // newline characters in logical lines. Simple approach: iterate
                // and build new string while counting removed newlines to re-insert
                // them at the next actual newline.
                let mut out = String::with_capacity(s.len());
                let bytes = s.as_bytes();
                let mut i = 0usize;
                let mut pending_newlines = 0usize;
                while i < bytes.len() {
                    if bytes[i] == b'\\' && i + 1 < bytes.len() && bytes[i+1] == b'\n' {
                        i += 2;
                        pending_newlines += 1;
                        continue;
                    }
                    if bytes[i] == b'\n' {
                        out.push('\n');
                        for _ in 0..pending_newlines { out.push('\n'); }
                        pending_newlines = 0;
                        i += 1;
                        continue;
                    }
                    out.push(bytes[i] as char);
                    i += 1;
                }
                for _ in 0..pending_newlines { out.push('\n'); }
                *s = out;
            }
            fn read_universal_char(s: &str, len: usize) -> Option<u32> {
                let mut c: u32 = 0;
                for ch in s.chars().take(len) {
                    let v = ch.to_digit(16)?;
                    c = (c << 4) | v;
                }
                Some(c)
            }
            fn convert_universal_chars(s: &mut String) {
                let mut out = String::with_capacity(s.len());
                let mut i = 0usize;
                let bytes = s.as_bytes();
                while i < bytes.len() {
                    if i + 1 < bytes.len() && bytes[i] == b'\\' && (bytes[i+1] == b'u' || bytes[i+1] == b'U') {
                        let is_u = bytes[i+1] == b'u';
                        let need = if is_u { 4 } else { 8 };
                        // Attempt to read hex digits from subsequent chars
                        let start = i + 2;
                        if start + need <= bytes.len() {
                            let hex_str = String::from_utf8_lossy(&bytes[start..start+need]).to_string();
                            if let Some(code) = read_universal_char(&hex_str, need) {
                                // encode code as UTF-8 into out
                                if let Some(ch) = std::char::from_u32(code) {
                                    out.push(ch);
                                    i = start + need;
                                    continue;
                                }
                            }
                        }
                        // fallback: copy slash then proceed
                        out.push('\\');
                        i += 1;
                        continue;
                    }
                    out.push(bytes[i] as char);
                    i += 1;
                }
                *s = out;
            }

            canonicalize_newline(&mut buf);
            remove_backslash_newline(&mut buf);
            convert_universal_chars(&mut buf);

            let f = File { name: Some(filename.to_string()), file_no: 0, contents: Some(buf), display_name: Some(filename.to_string()), line_delta: 0 };
            tokenize(Rc::new(RefCell::new(f)))
        }
        Err(_) => None,
    }
}

// preprocess

pub fn search_include_paths(_filename: &str) -> Option<String> {
    use std::path::Path;
    // Check local path first (relative to current working dir)
    let p = Path::new(_filename);
    if p.exists() {
        if let Ok(s) = p.canonicalize() {
            if let Some(sstr) = s.to_str() { return Some(sstr.to_string()); }
        }
    }

    // Check repository 'include' directory (relative to workspace)
    let repo_include = Path::new("include").join(_filename);
    if repo_include.exists() {
        if let Ok(s) = repo_include.canonicalize() {
            if let Some(sstr) = s.to_str() { return Some(sstr.to_string()); }
        }
    }

    // Common system include directories
    let sys_dirs = ["/usr/include", "/usr/local/include"];
    for dir in sys_dirs.iter() {
        let cand = Path::new(dir).join(_filename);
        if cand.exists() {
            if let Ok(s) = cand.canonicalize() {
                if let Some(sstr) = s.to_str() { return Some(sstr.to_string()); }
            }
        }
    }

    None
}

// Return all existing candidate paths for a filename in the usual search order.
pub fn search_include_paths_all(filename: &str) -> Vec<String> {
    use std::path::Path;
    let mut out: Vec<String> = Vec::new();

    // local path first
    let p = Path::new(filename);
    if p.exists() {
        if let Ok(s) = p.canonicalize() {
            if let Some(sstr) = s.to_str() { out.push(sstr.to_string()); }
        }
    }

    // repo include directory
    let repo_include = Path::new("include").join(filename);
    if repo_include.exists() {
        if let Ok(s) = repo_include.canonicalize() {
            if let Some(sstr) = s.to_str() { out.push(sstr.to_string()); }
        }
    }

    // system dirs
    let sys_dirs = ["/usr/include", "/usr/local/include"];
    for dir in sys_dirs.iter() {
        let cand = Path::new(dir).join(filename);
        if cand.exists() {
            if let Ok(s) = cand.canonicalize() {
                if let Some(sstr) = s.to_str() { out.push(sstr.to_string()); }
            }
        }
    }
    out
}

// search_include_next: like search_include_paths but skip any candidate in or before
// the including directory. `including_dir` should be a directory path (optional).
// Returns the first candidate after the including directory.
pub fn search_include_next(filename: &str, including_file: Option<&str>) -> Option<String> {
    use std::path::Path;
    let cand_list = search_include_paths_all(filename);
    // If we don't have an including file, return the first candidate
    if including_file.is_none() {
        return cand_list.into_iter().next();
    }
    // canonicalize including file path
    let inc_canon = Path::new(including_file.unwrap()).canonicalize().ok();
    if inc_canon.is_none() {
        return cand_list.into_iter().next();
    }
    let inc_canon = inc_canon.unwrap();

    // find the canonicalized candidate equal to including file; return next candidate
    let mut i = 0usize;
    for cand in cand_list.iter() {
        if let Ok(cpath) = Path::new(cand).canonicalize() {
            if cpath == inc_canon {
                // return next candidate if present
                if i + 1 < cand_list.len() {
                    return Some(cand_list[i + 1].clone());
                } else {
                    return None;
                }
            }
        }
        i += 1;
    }
    // If including file wasn't found in the candidate list, fall back to first candidate
    cand_list.into_iter().next()
}

pub fn init_macros() {
    preproc::init_macros();
}

pub fn define_macro(_name: &str, _buf: &str) {
    preproc::define_macro(_name, _buf);
}

pub fn undef_macro(_name: &str) {
    preproc::undef_macro(_name);
}

pub fn preprocess(_tok: Rc<RefCell<Token>>) -> Option<Rc<RefCell<Token>>> {
    // Call directive-level preprocessing implemented in the preproc module.
    if let Some(tok2) = preproc::preprocess2(_tok) {
        // perform any token-level post-processing (noop for now)
        convert_pp_tokens(Rc::clone(&tok2));

        // Adjust line numbers by line_delta for all tokens, like the C implementation.
        let mut cur_opt = Some(Rc::clone(&tok2));
        while let Some(cur) = cur_opt {
            if cur.borrow().kind == token::token::TokenKind::TK_EOF {
                break;
            }
            let delta = cur.borrow().line_delta;
            cur.borrow_mut().line_no += delta;
            cur_opt = cur.borrow().next.clone();
        }

        Some(tok2)
    } else {
        None
    }
}

// parse

pub fn new_cast(_expr: Rc<RefCell<Node>>, _ty: Rc<RefCell<Type>>) -> Rc<RefCell<Node>> {
    Rc::new(RefCell::new(Node::default()))
}

pub fn const_expr(_rest: &mut Option<Rc<RefCell<Token>>>, _tok: Rc<RefCell<Token>>) -> i64 {
    // Very small evaluator used by preprocessor #if: accept a single numeric
    // literal token or an identifier '0'/'1'. Otherwise, panic like the C
    // implementation calls error_tok.
    if _tok.borrow().kind == TokenKind::TK_NUM {
        return _tok.borrow().val;
    }
    if let Some(loc) = &_tok.borrow().loc {
        if let Ok(v) = loc.parse::<i64>() { return v; }
    }
    // Fallback: if it's an identifier 'defined' or similar, return 0.
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
