use std::rc::Rc;
use std::cell::RefCell;
use crate::token::token::{Token, TokenKind};
use crate::token::tokenize; // function re-exported by token mod
use crate::preproc::{get_macro, init_macros};
use crate::File;
use crate::hashmap::HashMap;
use std::fs;

// Helper: tokenize from &str using a temporary File
fn tokenize_from_str(src: &str) -> Option<Rc<RefCell<Token>>> {
    let f = File { name: None, file_no: 0, contents: Some(src.to_string()), display_name: None, line_delta: 0 };
    tokenize(Rc::new(RefCell::new(f)))
}

// Alias name used elsewhere
fn tokenize_from_string(s: &str) -> Option<Rc<RefCell<Token>>> { tokenize_from_str(s) }

// Clone a token (shallow copy of fields, new Token wrapper)
fn clone_token(tok: &Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    let t = tok.borrow();
    let mut nt = Token::default();
    nt.kind = t.kind;
    nt.loc = t.loc.clone();
    nt.len = t.len;
    nt.val = t.val;
    nt.fval = t.fval;
    nt.ty = t.ty.clone();
    nt.str_lit = t.str_lit.clone();
    nt.file = t.file.clone();
    nt.filename = t.filename.clone();
    nt.line_no = t.line_no;
    nt.line_delta = t.line_delta;
    nt.at_bol = t.at_bol;
    nt.has_space = t.has_space;
    nt.origin = t.origin.clone();
    nt.hideset = t.hideset.clone();
    Rc::new(RefCell::new(nt))
}

// Helper: join tokens into a single string (with spaces where has_space)
fn join_tokens_to_string(tok: Rc<RefCell<Token>>) -> String {
    let mut s = String::new();
    let mut cur = Some(tok);
    let mut first = true;
    while let Some(t) = cur {
        if t.borrow().kind == TokenKind::TK_EOF { break; }
        if !first && t.borrow().has_space { s.push(' '); }
        if let Some(loc) = &t.borrow().loc { s.push_str(loc); }
        first = false;
        cur = t.borrow().next.clone();
    }
    s
}

// Stringize: create a string literal token from an argument token list
fn stringize_token(arg: Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    let s = join_tokens_to_string(arg);
    let quoted = format!("\"{}\"", s.replace('"', "\\\""));
    let f = File { name: None, file_no: 0, contents: Some(quoted), display_name: None, line_delta: 0 };
    tokenize(Rc::new(RefCell::new(f))).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

// Builtin dynamic macros implementation
fn file_macro(tmpl: Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    // walk to origin without holding a RefCell borrow while assigning
    let mut t = Rc::clone(&tmpl);
    loop {
        let origin_opt = { t.borrow().origin.clone() };
        if let Some(o) = origin_opt { t = o; } else { break; }
    }
    if let Some(f_rc) = &t.borrow().file {
        let display = { f_rc.borrow().display_name.clone() };
        if let Some(name) = display {
            // return quoted filename token
            return tokenize_from_str(&format!("\"{}\"", name)).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
        }
    }
    // fallback: empty string
    tokenize_from_str("\"\"").unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

fn line_macro(tmpl: Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    let mut t = Rc::clone(&tmpl);
    loop {
        let origin_opt = { t.borrow().origin.clone() };
        if let Some(o) = origin_opt { t = o; } else { break; }
    }
    let (ln, ld) = {
        let b = t.borrow();
        (b.line_no, b.line_delta)
    };
    let i = ln + ld;
    let s = format!("{}", i);
    tokenize_from_str(&s).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

fn counter_macro(_tmpl: Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    use std::sync::atomic::{AtomicI64, Ordering};
    static COUNTER: AtomicI64 = AtomicI64::new(0);
    let v = COUNTER.fetch_add(1, Ordering::SeqCst);
    let s = format!("{}", v);
    let f = File { name: None, file_no: 0, contents: Some(s.clone()), display_name: None, line_delta: 0 };
    tokenize(Rc::new(RefCell::new(f))).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

fn timestamp_macro(tmpl: Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    use std::fs::metadata;
    let path_opt = { tmpl.borrow().file.as_ref().and_then(|f| f.borrow().name.clone()) };
    if let Some(path) = path_opt {
        if let Ok(meta) = metadata(&path) {
            if let Ok(mtime) = meta.modified() {
                let datetime: chrono::DateTime<chrono::Local> = mtime.into();
                let s = datetime.format("%a %b %d %T %Y").to_string();
                return tokenize_from_str(&format!("\"{}\"", s)).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
            }
        }
    }
    tokenize_from_str("\"\"").unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

fn base_file_macro(_tmpl: Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    tokenize_from_str("\"\"").unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

// Paste: concatenate loc strings of lhs and rhs and retokenize
fn paste_tokens(lhs: Rc<RefCell<Token>>, rhs: Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    let l = lhs.borrow().loc.clone().unwrap_or_default();
    let r = rhs.borrow().loc.clone().unwrap_or_default();
    let buf = format!("{}{}", l, r);
    let f = File { name: None, file_no: 0, contents: Some(buf), display_name: None, line_delta: 0 };
    tokenize(Rc::new(RefCell::new(f))).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

// Hideset helpers (simple linked-list semantics matching C impl)
fn new_hideset(name: &str) -> Rc<RefCell<crate::token::token::Hideset>> {
    Rc::new(RefCell::new(crate::token::token::Hideset { next: None, name: name.to_string() }))
}

fn hideset_contains(hs: &Option<Rc<RefCell<crate::token::token::Hideset>>>, s: &str) -> bool {
    let mut cur = hs.as_ref().map(Rc::clone);
    while let Some(c) = cur {
        if c.borrow().name == s { return true; }
        cur = c.borrow().next.clone();
    }
    false
}

fn hideset_union(hs1: &Option<Rc<RefCell<crate::token::token::Hideset>>>, hs2: &Option<Rc<RefCell<crate::token::token::Hideset>>>) -> Option<Rc<RefCell<crate::token::token::Hideset>>> {
    // clone hs1 list
    let mut head: Option<Rc<RefCell<crate::token::token::Hideset>>> = None;
    let mut tail: Option<Rc<RefCell<crate::token::token::Hideset>>> = None;
    let mut cur = hs1.as_ref().map(Rc::clone);
    while let Some(c) = cur {
        let n = new_hideset(&c.borrow().name);
        if head.is_none() { head = Some(Rc::clone(&n)); tail = Some(Rc::clone(&n)); } else { tail.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&n)); tail = Some(n); }
        cur = c.borrow().next.clone();
    }
    // append hs2 (shared)
    if tail.is_none() { return hs2.clone(); }
    tail.as_ref().unwrap().borrow_mut().next = hs2.clone();
    head
}

fn hideset_intersection(a: &Option<Rc<RefCell<crate::token::token::Hideset>>>, b: &Option<Rc<RefCell<crate::token::token::Hideset>>>) -> Option<Rc<RefCell<crate::token::token::Hideset>>> {
    let mut head: Option<Rc<RefCell<crate::token::token::Hideset>>> = None;
    let mut tail: Option<Rc<RefCell<crate::token::token::Hideset>>> = None;
    let mut cur = a.as_ref().map(Rc::clone);
    while let Some(c) = cur {
        if hideset_contains(b, &c.borrow().name) {
            let n = new_hideset(&c.borrow().name);
            if head.is_none() { head = Some(Rc::clone(&n)); tail = Some(Rc::clone(&n)); } else { tail.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&n)); tail = Some(n); }
        }
        cur = c.borrow().next.clone();
    }
    head
}

fn add_hideset(tok: &Rc<RefCell<Token>>, hs: &Option<Rc<RefCell<crate::token::token::Hideset>>>) -> Rc<RefCell<Token>> {
    // clone token list and union hideset
    let mut head: Option<Rc<RefCell<Token>>> = None;
    let mut tail: Option<Rc<RefCell<Token>>> = None;
    let mut cur = Some(Rc::clone(tok));
    while let Some(c) = cur {
    let cp = clone_token(&c);
        let existing = cp.borrow().hideset.clone();
        cp.borrow_mut().hideset = hideset_union(&existing, hs);
        if head.is_none() { head = Some(Rc::clone(&cp)); tail = Some(Rc::clone(&cp)); } else { tail.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&cp)); tail = Some(cp); }
        cur = c.borrow().next.clone();
    }
    head.unwrap_or_else(|| Rc::new(RefCell::new(Token::default())))
}

// Append tokens: return a cloned list of src (deep clone) and append to dest tail
fn append_tokens(dest_tail: &mut Rc<RefCell<Token>>, src: &Rc<RefCell<Token>>) -> Rc<RefCell<Token>> {
    // build clone of src list
    let mut src_clone_head: Option<Rc<RefCell<Token>>> = None;
    let mut src_clone_tail: Option<Rc<RefCell<Token>>> = None;
    let mut cur = Some(Rc::clone(src));
    while let Some(c) = cur {
        let cloned = clone_token(&c);
        if src_clone_head.is_none() {
            src_clone_head = Some(Rc::clone(&cloned));
            src_clone_tail = Some(Rc::clone(&cloned));
        } else {
            src_clone_tail.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&cloned));
            src_clone_tail = Some(Rc::clone(&cloned));
        }
        cur = c.borrow().next.as_ref().map(Rc::clone);
    }

    if let Some(head) = src_clone_head {
        dest_tail.borrow_mut().next = Some(head.clone());
        // return new tail
        let mut t = head;
        loop {
            let next_t = t.borrow().next.clone();
            if let Some(n) = next_t { t = n; } else { break; }
        }
        return t;
    }
    Rc::clone(dest_tail)
}

// Preprocess: expand object-like macros only. Input is token list head; returns new head.
pub fn preprocess(tok: Rc<RefCell<Token>>) -> Option<Rc<RefCell<Token>>> {
    init_macros();
    let head = tok;
    let dummy = Rc::new(RefCell::new(Token::default()));
    dummy.borrow_mut().kind = TokenKind::TK_EOF;
    dummy.borrow_mut().next = Some(Rc::clone(&head));
    let mut prev: Rc<RefCell<Token>> = Rc::clone(&dummy);

    // For each token, if identifier and matches macro, tokenize macro body and splice
    loop {
        let next_opt = prev.borrow().next.clone();
        let cur = match next_opt {
            Some(c) => c,
            None => break,
        };

        let is_ident = {
            let b = cur.borrow();
            matches!(b.kind, TokenKind::TK_IDENT)
        };
        if is_ident {
            let name = cur.borrow().loc.clone().unwrap_or_default();
            // If this token's hideset contains the macro name, skip expansion
            if hideset_contains(&cur.borrow().hideset, &name) {
                prev = cur;
                continue;
            }
            if let Some(m) = get_macro(&name) {
                // object-like macro
                if m.params.is_none() {
                    let body = &m.body;
                    if let Some(body_tokens) = tokenize_from_string(body) {
                        let next = cur.borrow().next.clone();
                        prev.borrow_mut().next = Some(Rc::clone(&body_tokens));
                        let mut tail = Rc::clone(&body_tokens);
                        loop {
                            let next_tail = tail.borrow().next.clone();
                            if let Some(n) = next_tail { tail = n; } else { break; }
                        }
                        tail.borrow_mut().next = next;
                        prev = tail;
                        continue;
                    }
                } else {
                    // function-like macro: expect '('
                    let maybe_lp = cur.borrow().next.clone();
                    if maybe_lp.is_none() { prev = cur; continue; }
                    let lp = maybe_lp.unwrap();
                    let is_lparen = lp.borrow().kind == TokenKind::TK_PUNCT && lp.borrow().loc.as_deref() == Some("(");
                    if !is_lparen { prev = cur; continue; }

                    // parse arguments: start at token after '('
                    let mut scan = lp.borrow().next.clone().unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
                    let mut args: Vec<Rc<RefCell<Token>>> = Vec::new();
                    let mut level: i32 = 0;
                    let mut cur_arg_head: Option<Rc<RefCell<Token>>> = None;
                    let mut cur_arg_tail: Option<Rc<RefCell<Token>>> = None;

                    loop {
                        let scan_clone = Rc::clone(&scan);
                        let kind = scan_clone.borrow().kind;
                        let loc = scan_clone.borrow().loc.clone();
                        let is_punct = kind == TokenKind::TK_PUNCT;
                        let is_comma_or_rparen = is_punct && loc.as_deref().map(|s| s == "," || s == ")").unwrap_or(false);

                        if is_comma_or_rparen && level == 0 {
                            // finalize current arg
                            if cur_arg_head.is_none() {
                                let eof = Rc::new(RefCell::new(Token::default()));
                                eof.borrow_mut().kind = TokenKind::TK_EOF;
                                cur_arg_head = Some(eof.clone());
                            } else {
                                let eof = Rc::new(RefCell::new(Token::default()));
                                eof.borrow_mut().kind = TokenKind::TK_EOF;
                                cur_arg_tail.as_ref().unwrap().borrow_mut().next = Some(eof.clone());
                            }
                            args.push(cur_arg_head.take().unwrap());
                            cur_arg_tail = None;

                            // if rparen end, advance scan to token after ')' and break
                            if loc.as_deref() == Some(")") {
                                let next_scan = scan_clone.borrow().next.clone().unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
                                scan = next_scan;
                                break;
                            }
                            // else it's a comma, advance to next token and continue
                            scan = scan_clone.borrow().next.clone().unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
                            continue;
                        }

                        // update nesting only examining the clone
                        if is_punct {
                            if loc.as_deref() == Some("(") { level += 1; }
                            else if loc.as_deref() == Some(")") { level -= 1; }
                        }

                        // append token to current arg clone
                        let nt = clone_token(&scan_clone);
                        if cur_arg_head.is_none() { cur_arg_head = Some(nt.clone()); } else { cur_arg_tail.as_ref().unwrap().borrow_mut().next = Some(nt.clone()); }
                        cur_arg_tail = Some(nt);
                        // advance
                        scan = scan_clone.borrow().next.clone().unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
                    }

                    // map params to args
                    let params = m.params.clone().unwrap_or_default();
                    let mut arg_map: Vec<(String, Rc<RefCell<Token>>)> = Vec::new();
                    for (i, pname) in params.iter().enumerate() {
                        let p_tok = if i < args.len() { args[i].clone() } else { Rc::new(RefCell::new(Token::default())) };
                        arg_map.push((pname.clone(), p_tok));
                    }

                    // tokenize macro body and substitute
                    let body_tokens = tokenize_from_string(&m.body).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
                    // build replacement list
                    let mut out_head: Option<Rc<RefCell<Token>>> = None;
                    let mut out_tail: Option<Rc<RefCell<Token>>> = None;
                    let mut bscan = Some(Rc::clone(&body_tokens));
                    while let Some(bt) = bscan {
                        if bt.borrow().kind == TokenKind::TK_EOF { break; }
                        // handle stringize operator: if this token is a '#' punct followed by an identifier
                        if bt.borrow().kind == TokenKind::TK_PUNCT {
                            if let Some(p) = bt.borrow().loc.clone() {
                                if p == "#" {
                                    // lookahead to next token
                                    if let Some(next_bt) = bt.borrow().next.clone() {
                                        if next_bt.borrow().kind == TokenKind::TK_IDENT {
                                            if let Some(name_s) = next_bt.borrow().loc.clone() {
                                                if let Some((_, argtok)) = arg_map.iter().find(|(n, _)| n == &name_s) {
                                                    // stringize the argument
                                                    let s_tok = stringize_token(argtok.clone());
                                                    if out_head.is_none() { out_head = Some(s_tok.clone()); } else { out_tail.as_ref().unwrap().borrow_mut().next = Some(s_tok.clone()); }
                                                    out_tail = Some(s_tok);
                                                    // skip both # and the identifier in body
                                                    bscan = next_bt.borrow().next.clone();
                                                    continue;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // handle token-paste '##' by detecting if current token in body has trailing '##' or an upcoming '##'
                        // We implement a simple left-associative paste: if bt is an identifier/punct and next is '##',
                        // then paste bt with token after '##'. This supports forms like 'a ## b' and 'a##b'.
                        if bt.borrow().kind == TokenKind::TK_IDENT || bt.borrow().kind == TokenKind::TK_PUNCT {
                            // check next is '##'
                            if let Some(next1) = bt.borrow().next.clone() {
                                if next1.borrow().kind == TokenKind::TK_PUNCT && next1.borrow().loc.as_deref() == Some("##") {
                                    // token after '##'
                                    if let Some(next2) = next1.borrow().next.clone() {
                                        // determine lhs token (bt) and rhs token (next2)
                                        // if either side is a parameter name, substitute the parameter tokens first (we'll use first token of arg)
                                        let lhs_tok = if bt.borrow().kind == TokenKind::TK_IDENT {
                                            if let Some(name_s) = bt.borrow().loc.clone() {
                                                if let Some((_, argtok)) = arg_map.iter().find(|(n, _)| n == &name_s) {
                                                    // use first token of arg for paste (or empty)
                                                    argtok.clone()
                                                } else { bt.clone() }
                                            } else { bt.clone() }
                                        } else { bt.clone() };

                                        let rhs_tok = if next2.borrow().kind == TokenKind::TK_IDENT {
                                            if let Some(name_s) = next2.borrow().loc.clone() {
                                                if let Some((_, argtok)) = arg_map.iter().find(|(n, _)| n == &name_s) {
                                                    argtok.clone()
                                                } else { next2.clone() }
                                            } else { next2.clone() }
                                        } else { next2.clone() };

                                        // take first token from lhs_tok and rhs_tok for textual paste
                                        let lhs_first = lhs_tok.clone();
                                        let rhs_first = rhs_tok.clone();
                                        let pasted = paste_tokens(lhs_first, rhs_first);
                                        // add the current macro name to pasted tokens' hideset
                                        let hs = Some(new_hideset(&name));
                                        let pasted_with_hs = add_hideset(&pasted, &hs);
                                        // rescanning: preprocess the pasted-with-hideset list so newly formed
                                        // identifiers can be expanded (hideset prevents re-expansion of the
                                        // macro that produced the paste)
                                        let rescanned = preprocess(Rc::clone(&pasted_with_hs)).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));

                                        // append rescanned tokens (clone them into out list)
                                        let mut p_cur = Some(Rc::clone(&rescanned));
                                        while let Some(p) = p_cur {
                                            if p.borrow().kind == TokenKind::TK_EOF { break; }
                                            let nc = clone_token(&p);
                                            if out_head.is_none() { out_head = Some(nc.clone()); } else { out_tail.as_ref().unwrap().borrow_mut().next = Some(nc.clone()); }
                                            out_tail = Some(nc);
                                            p_cur = p.borrow().next.clone();
                                        }

                                        // advance bscan to token after next2
                                        bscan = next2.borrow().next.clone();
                                        continue;
                                    }
                                }
                            }
                        }

                        if bt.borrow().kind == TokenKind::TK_IDENT {
                            if let Some(name_s) = bt.borrow().loc.clone() {
                                if let Some((_, argtok)) = arg_map.iter().find(|(n, _)| n == &name_s) {
                                    // append argtok (clone its list)
                                    let mut a_cur = Some(Rc::clone(argtok));
                                    while let Some(a) = a_cur {
                                        if a.borrow().kind == TokenKind::TK_EOF { break; }
                                        let nc = clone_token(&a);
                                        if out_head.is_none() { out_head = Some(nc.clone()); } else { out_tail.as_ref().unwrap().borrow_mut().next = Some(nc.clone()); }
                                        out_tail = Some(nc);
                                        a_cur = a.borrow().next.clone();
                                    }
                                    bscan = bt.borrow().next.clone();
                                    continue;
                                }
                            }
                        }
                        // otherwise copy token
                        let nc = clone_token(&bt);
                        if out_head.is_none() { out_head = Some(nc.clone()); } else { out_tail.as_ref().unwrap().borrow_mut().next = Some(nc.clone()); }
                        out_tail = Some(nc);
                        bscan = bt.borrow().next.clone();
                    }
                    // ensure EOF
                    let eof = Rc::new(RefCell::new(Token::default()));
                    eof.borrow_mut().kind = TokenKind::TK_EOF;
                    if out_head.is_none() { out_head = Some(eof.clone()); } else { out_tail.as_ref().unwrap().borrow_mut().next = Some(eof.clone()); }

                    // splice replacement where cur was: prev -> out_head -> rest after ')'
                    let rest_after = Some(scan.clone());
                    prev.borrow_mut().next = out_head.clone();
                    // find tail
                    let mut tail = out_head.unwrap();
                    loop {
                        let nt = tail.borrow().next.clone();
                        if let Some(n) = nt { tail = n; } else { break; }
                    }
                    tail.borrow_mut().next = rest_after;
                    prev = tail;
                    continue;
                }
            }
        }
        prev = cur;
    }

    dummy.borrow().next.clone()
}

// Preprocess2: handle directive-level processing (#define, #undef, #ifdef/#ifndef, etc.)
pub fn preprocess2(tok: Rc<RefCell<Token>>) -> Option<Rc<RefCell<Token>>> {
    init_macros();
    // Simple include bookkeeping similar to the C version.
    // pragma_once map: filename -> 1
    let _pragma_once: HashMap<i32> = HashMap::new();
    // include_guards map: path -> guard macro name (reserved for future use)
    let _include_guards: HashMap<String> = HashMap::new();
    // We'll build a new token list by walking tok
    let dummy = Rc::new(RefCell::new(Token::default()));
    let mut out_tail = Rc::clone(&dummy);
    let mut cur_opt: Option<Rc<RefCell<Token>>> = Some(tok);

    // Normalization pass: split tokens whose loc starts with '#' and contain more
    // characters (e.g. '#ifdefined(UNDEF)') into a '#' token followed by tokens
    // produced by tokenizing the remainder. This makes directive parsing simpler.
    {
        // Rebuild a normalized token stream into `normalized_head`.
        // For any token that appears at BOL and whose loc starts with '#'
        // and contains more characters (e.g. "#ifdefined(...)"), emit a
        // separate '#' punct token followed by tokens produced by
        // tokenizing the remainder string. This is a deterministic and
        // simpler normalization than doing in-place splicing.
    let normalized_dummy = Rc::new(RefCell::new(Token::default()));
    let mut norm_tail = Rc::clone(&normalized_dummy);
        let mut cur_n = cur_opt.clone();
        while let Some(t) = cur_n.clone() {
            // stop at EOF
            if t.borrow().kind == TokenKind::TK_EOF { break; }
            if t.borrow().at_bol {
                if let Some(loc) = t.borrow().loc.clone() {
                    if loc.starts_with('#') && loc.len() > 1 {
                        // emit explicit '#' token with same at_bol
                        let mut hash_tok = Token::default();
                        hash_tok.kind = TokenKind::TK_PUNCT;
                        hash_tok.loc = Some("#".to_string());
                        hash_tok.at_bol = true;
                        hash_tok.file = t.borrow().file.clone();
                        hash_tok.filename = t.borrow().filename.clone();
                        let hash_rc = Rc::new(RefCell::new(hash_tok));
                        norm_tail.borrow_mut().next = Some(Rc::clone(&hash_rc));
                        norm_tail = hash_rc;

                        // retokenize remainder and append its tokens (except EOF)
                        let rem = loc[1..].to_string();
                        if let Some(rhead) = tokenize_from_str(&rem) {
                            // iterate rhead list and append clones (preserve loc/has_space but clear at_bol)
                            let mut rcur_opt = Some(rhead);
                            while let Some(rcur) = rcur_opt.clone() {
                                if rcur.borrow().kind == TokenKind::TK_EOF { break; }
                                let nc = clone_token(&rcur);
                                nc.borrow_mut().at_bol = false;
                                norm_tail.borrow_mut().next = Some(Rc::clone(&nc));
                                norm_tail = nc;
                                rcur_opt = rcur.borrow().next.clone();
                            }
                        }
                        cur_n = t.borrow().next.clone();
                        continue;
                    }
                }
            }
            // normal token: clone and append
            let nc = clone_token(&t);
            norm_tail.borrow_mut().next = Some(Rc::clone(&nc));
            norm_tail = nc;
            cur_n = t.borrow().next.clone();
        }
        // ensure EOF appended
        let eof = Rc::new(RefCell::new(Token::default()));
        eof.borrow_mut().kind = TokenKind::TK_EOF;
        norm_tail.borrow_mut().next = Some(Rc::clone(&eof));
        // replace cur_opt with normalized head
        cur_opt = normalized_dummy.borrow().next.clone();
    }

    // Helper: collect rest of directive line into a single string and return next token after the line
    fn collect_directive_line(start: &Rc<RefCell<Token>>) -> (String, Option<Rc<RefCell<Token>>>) {
        let mut s = String::new();
        let mut cur = Some(Rc::clone(start));
        while let Some(c) = cur.clone() {
            if c.borrow().kind == TokenKind::TK_EOF { break; }
            if c.borrow().at_bol && !s.is_empty() { // reached next line
                break;
            }
            if let Some(loc) = &c.borrow().loc { if !s.is_empty() && c.borrow().has_space { s.push(' '); } s.push_str(loc); }
            cur = c.borrow().next.clone();
        }
        (s, cur)
    }

    // Read include filename from a directive line, with macro expansion.
    // Returns (filename, next_token_after_directive).
    fn read_include_filename(start: &Rc<RefCell<Token>>) -> (Option<String>, Option<Rc<RefCell<Token>>>) {
        // Reuse collect_directive_line to get the raw text for this directive
        let (line, next_tok) = collect_directive_line(start);
        // Tokenize the directive text and preprocess it to expand macros
        if let Some(toks) = tokenize_from_str(&line) {
            if let Some(expanded) = preprocess(Rc::clone(&toks)) {
                // scan for the identifier 'include'
                let mut cur = Some(expanded);
                while let Some(c) = cur.clone() {
                    if c.borrow().kind == TokenKind::TK_IDENT {
                        if let Some(loc) = &c.borrow().loc {
                            if loc == "include" {
                                // filename should be the next token
                                if let Some(ft) = c.borrow().next.clone() {
                                    // quoted string
                                    if ft.borrow().kind == TokenKind::TK_STR {
                                        if let Some(s) = &ft.borrow().loc {
                                            let name = if s.starts_with('"') && s.ends_with('"') && s.len()>=2 { s[1..s.len()-1].to_string() } else { s.clone() };
                                            return (Some(name), next_tok);
                                        }
                                    }
                                    // angle-bracket: sequence starting with '<'
                                    if ft.borrow().kind == TokenKind::TK_PUNCT && ft.borrow().loc.as_deref() == Some("<") {
                                        let mut p_opt = Some(ft.clone());
                                        let mut name = String::new();
                                        while let Some(p) = p_opt.clone() {
                                            if p.borrow().kind == TokenKind::TK_PUNCT && p.borrow().loc.as_deref() == Some(">") { break; }
                                            if let Some(loc) = &p.borrow().loc { name.push_str(loc); }
                                            p_opt = p.borrow().next.clone();
                                        }
                                        if name.starts_with('<') { name = name[1..].to_string(); }
                                        if name.ends_with('>') { name.pop(); }
                                        return (Some(name), next_tok);
                                    }
                                }
                            }
                        }
                    }
                    cur = c.borrow().next.clone();
                }
            }
        }
        (None, next_tok)
    }

    // Evaluate a preprocessing constant expression line. This follows the C implementation:
    // 1. Tokenize the directive expression.
    // 2. Preprocess it (expand macros).
    // 3. Replace remaining identifiers with numeric 0.
    // 4. Convert pp-numbers (via crate::convert_pp_tokens).
    // 5. Call crate::const_expr to evaluate the expression.
    fn eval_const_expr_line(line: &str) -> bool {
        // Extract the expression part after the 'if' directive. Handle cases like
        // '#if defined(...)', '#ifdefined(...)', or '#if    defined ...'. We find the
        // first occurrence of the substring "if" and take the remainder as the expression.
        let expr = {
            if let Some(pos) = line.find("if") {
                line[pos + 2..].trim().to_string()
            } else {
                // fallback: remove a leading '#' if present
                line.trim_start_matches('#').trim().to_string()
            }
        };
        if expr.is_empty() { return false; }
        if let Some(toks) = tokenize_from_str(&expr) {
            // Preprocess (expand macros inside the expression)
            if let Some(pre) = preprocess(Rc::clone(&toks)) {
                // First, handle the `defined` operator in two forms:
                //  - defined IDENT
                //  - defined ( IDENT )
                // We scan tokens and replace these sequences with numeric 1/0 tokens depending
                // on whether the identifier is defined in the macro registry.
                let mut cur_opt = Some(Rc::clone(&pre));
                while let Some(cur) = cur_opt.clone() {
                    if cur.borrow().kind == TokenKind::TK_EOF { break; }
                    if cur.borrow().kind == TokenKind::TK_IDENT {
                        if let Some(kw) = cur.borrow().loc.clone() {
                            if kw == "defined" {
                                // Look ahead to next meaningful token
                                let next_tok = cur.borrow().next.clone();
                                if let Some(nt) = next_tok {
                                    // Case: defined ( IDENT )
                                    if nt.borrow().kind == TokenKind::TK_PUNCT && nt.borrow().loc.as_deref() == Some("(") {
                                        if let Some(id_tok) = nt.borrow().next.clone() {
                                            if id_tok.borrow().kind == TokenKind::TK_IDENT {
                                                let name = id_tok.borrow().loc.clone().unwrap_or_default();
                                                let defined = crate::preproc::get_macro(&name).is_some();
                                                // replace cur..id_tok (inclusive) with a numeric token
                                                cur.borrow_mut().kind = TokenKind::TK_NUM;
                                                cur.borrow_mut().val = if defined { 1 } else { 0 };
                                                cur.borrow_mut().loc = Some(if defined { "1".to_string() } else { "0".to_string() });
                                                // splice out the following two tokens '(' and identifier and optional ')'
                                                // set cur.next = token after id_tok.next if that is ')', else after id_tok
                                                let after_id = id_tok.borrow().next.clone();
                                                if let Some(after) = after_id.clone() {
                                                    if after.borrow().kind == TokenKind::TK_PUNCT && after.borrow().loc.as_deref() == Some(")") {
                                                        cur.borrow_mut().next = after.borrow().next.clone();
                                                    } else {
                                                        cur.borrow_mut().next = after_id;
                                                    }
                                                } else {
                                                    cur.borrow_mut().next = None;
                                                }
                                                // continue scanning from cur.next
                                                cur_opt = cur.borrow().next.clone();
                                                continue;
                                            }
                                        }
                                    }
                                    // Case: defined IDENT
                                    if nt.borrow().kind == TokenKind::TK_IDENT {
                                        let name = nt.borrow().loc.clone().unwrap_or_default();
                                        let defined = crate::preproc::get_macro(&name).is_some();
                                        cur.borrow_mut().kind = TokenKind::TK_NUM;
                                        cur.borrow_mut().val = if defined { 1 } else { 0 };
                                        cur.borrow_mut().loc = Some(if defined { "1".to_string() } else { "0".to_string() });
                                        // splice out the following identifier
                                        cur.borrow_mut().next = nt.borrow().next.clone();
                                        cur_opt = cur.borrow().next.clone();
                                        continue;
                                    }
                                }
                            }
                        }
                    }
                    cur_opt = cur.borrow().next.clone();
                }

                // After processing defined(), replace any remaining identifiers with 0
                let mut cur_opt = Some(Rc::clone(&pre));
                while let Some(cur) = cur_opt.clone() {
                    if cur.borrow().kind == TokenKind::TK_EOF { break; }
                    if cur.borrow().kind == TokenKind::TK_IDENT {
                        // convert to numeric 0 token
                        cur.borrow_mut().kind = TokenKind::TK_NUM;
                        cur.borrow_mut().val = 0;
                        cur.borrow_mut().loc = Some("0".to_string());
                    }
                    cur_opt = cur.borrow().next.clone();
                }

                // Convert pp-numbers if any
                crate::convert_pp_tokens(Rc::clone(&pre));

                // Evaluate using crate::const_expr
                let mut rest: Option<Rc<RefCell<Token>>> = pre.borrow().next.clone();
                let val = crate::const_expr(&mut rest, Rc::clone(&pre));
                return val != 0;
            }
        }
        false
    }

    // Detect include guard pattern in a tokenized file: returns Some(macro_name)
    fn detect_include_guard(tok: Rc<RefCell<Token>>) -> Option<String> {
        // Look for: #ifndef NAME  followed by #define NAME
        let cur = Some(tok);
        // Check first directive
        if cur.is_none() { return None; }
        let start = cur.unwrap();
        if !(start.borrow().at_bol && start.borrow().kind == TokenKind::TK_PUNCT && start.borrow().loc.as_deref() == Some("#")) { return None; }
        let next = start.borrow().next.clone();
        if next.is_none() || next.as_ref().unwrap().borrow().loc.as_deref() != Some("ifndef") { return None; }
        let name_tok = next.unwrap().borrow().next.clone();
        if name_tok.is_none() || name_tok.as_ref().unwrap().borrow().kind != TokenKind::TK_IDENT { return None; }
        let name = name_tok.as_ref().unwrap().borrow().loc.clone().unwrap_or_default();

        // Now find the next directive and check for `#define name`
        let mut scan = name_tok.unwrap().borrow().next.clone();
        while let Some(s) = scan.clone() {
            if s.borrow().kind == TokenKind::TK_EOF { break; }
            if s.borrow().at_bol && s.borrow().kind == TokenKind::TK_PUNCT && s.borrow().loc.as_deref() == Some("#") {
                if let Some(id) = s.borrow().next.clone() {
                    if id.borrow().loc.as_deref() == Some("define") {
                        if let Some(id2) = id.borrow().next.clone() {
                            if id2.borrow().kind == TokenKind::TK_IDENT && id2.borrow().loc.as_deref() == Some(&name) {
                                return Some(name);
                            }
                        }
                    }
                }
            }
            scan = s.borrow().next.clone();
        }
        None
    }

    // Clone a token list `src`, append `rest` at the end, and return the cloned head.
    fn clone_list_and_append(src: &Rc<RefCell<Token>>, rest: Option<Rc<RefCell<Token>>>) -> Rc<RefCell<Token>> {
        let mut head: Option<Rc<RefCell<Token>>> = None;
        let mut tail: Option<Rc<RefCell<Token>>> = None;
        let mut cur = Some(Rc::clone(src));
        while let Some(c) = cur {
            if c.borrow().kind == TokenKind::TK_EOF { break; }
            let nc = clone_token(&c);
            if head.is_none() { head = Some(Rc::clone(&nc)); tail = Some(Rc::clone(&nc)); } else { tail.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&nc)); tail = Some(nc); }
            cur = c.borrow().next.clone();
        }
        // ensure EOF
        let eof = Rc::new(RefCell::new(Token::default()));
        eof.borrow_mut().kind = TokenKind::TK_EOF;
        if head.is_none() { head = Some(Rc::clone(&eof)); } else { tail.as_ref().unwrap().borrow_mut().next = Some(Rc::clone(&eof)); }
        // append rest
        if let Some(r) = rest { tail = Some(Rc::clone(&eof)); tail.as_ref().unwrap().borrow_mut().next = Some(r); }
        head.unwrap()
    }

    // include_file: open and tokenize file, respect pragma_once and include guards
    fn include_file(cur: Rc<RefCell<Token>>, path: &str, _filename_tok: Rc<RefCell<Token>>) -> Option<Rc<RefCell<Token>>> {
        // pragma_once check
        if crate::preproc::pragma_once_get(path) { return Some(cur); }

        // include_guard check
        if let Some(guard_name) = crate::preproc::include_guard_get(path) {
            if crate::preproc::get_macro(&guard_name).is_some() {
                return Some(cur);
            }
        }

        // tokenize file
        if let Some(toks) = crate::tokenize_file(path) {
            // detect include guard
            if let Some(guard) = detect_include_guard(Rc::clone(&toks)) {
                crate::preproc::include_guard_put(path, &guard);
            }
            // naive: if file header contains '#pragma once' set pragma
            // scan first few tokens for pragma once
            let mut sopt = Some(Rc::clone(&toks));
            while let Some(s) = sopt.clone() {
                if s.borrow().kind == TokenKind::TK_EOF { break; }
                if s.borrow().at_bol && s.borrow().kind == TokenKind::TK_PUNCT && s.borrow().loc.as_deref() == Some("#") {
                    if let Some(nn) = s.borrow().next.clone() {
                        if nn.borrow().kind == TokenKind::TK_IDENT && nn.borrow().loc.as_deref() == Some("pragma") {
                            if let Some(nn2) = nn.borrow().next.clone() {
                                if nn2.borrow().kind == TokenKind::TK_IDENT && nn2.borrow().loc.as_deref() == Some("once") {
                                    crate::preproc::pragma_once_put(path);
                                }
                            }
                        }
                    }
                }
                sopt = s.borrow().next.clone();
            }

            // append tokenized file to current token stream by cloning the file tokens
            let new_head = clone_list_and_append(&toks, Some(cur));
            return Some(new_head);
        } else {
            // cannot open file: return current token unchanged
            return Some(cur);
        }
    }

    loop {
        let cur = match cur_opt.clone() { Some(c) => c, None => break };
        let b = cur.borrow();
    // Accept tokens where the token text starts with '#' (e.g. '#', '#if', '#ifdefined')
    if b.at_bol && b.loc.as_ref().map(|l| l.starts_with('#')).unwrap_or(false) {
            // collect directive line
            let (line, next_tok) = collect_directive_line(&cur);
            // Normalize directive name: handle forms like '#if', '#ifdefined(...)', or tokens where '#' and name are concatenated.
            let sline = line.trim_start();
            let parts = sline.split_whitespace().collect::<Vec<&str>>();
            let mut idx = 0usize;
            if sline.starts_with('#') { idx = 1; }
            let mut directive = String::new();
            while idx < sline.len() {
                let ch = sline.as_bytes()[idx] as char;
                if ch.is_alphabetic() { directive.push(ch); idx += 1; } else { break; }
            }
            let rest = sline[idx..].trim_start();
            if !directive.is_empty() {
                // Normalize some concatenated directive forms: treat 'ifdefined' as 'if'
                let directive_key = if directive == "if" || directive == "ifdefined" { "if" } else { directive.as_str() };
                match directive_key {
                    "define" => {
                        if parts.len() >= 2 {
                            // naive parse: name may be like NAME(...) or NAME
                            // use the rest substring after the directive name
                            let rest = rest;
                            // find name
                            if let Some(paren_idx) = rest.find('(') {
                                // function-like if '(' directly after name with no space
                                let name = rest[..paren_idx].trim().to_string();
                                // find closing paren
                                if let Some(close_idx) = rest[paren_idx..].find(')') {
                                    let params_text = &rest[paren_idx+1..paren_idx+close_idx];
                                    let params = params_text.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect::<Vec<String>>();
                                    let body = rest[paren_idx+close_idx+1..].trim().to_string();
                                    crate::preproc::define_function_macro(&name, params, &body);
                                }
                            } else {
                                // object-like
                                let mut iter = rest.splitn(2, |c:char| c.is_whitespace());
                                if let Some(name) = iter.next() {
                                    let body = iter.next().unwrap_or("").trim().to_string();
                                    crate::preproc::define_macro(name, &body);
                                }
                            }
                        }
                        cur_opt = next_tok;
                        continue;
                    }
                    "undef" => {
                        if parts.len() >= 2 { crate::preproc::undef_macro(parts[1]); }
                        cur_opt = next_tok;
                        continue;
                    }
                    "include" => {
                        // Read filename (with macro expansion) and next token
                        let (maybe_name, _nt) = read_include_filename(&cur);
                        if let Some(name) = maybe_name {
                            // Determine including file directory if available (used for relative includes)
                                let mut incl_dir: Option<String> = None;
                                if let Some(file_rc) = &cur.borrow().file {
                                    if let Some(base_name) = &file_rc.borrow().name {
                                        use std::path::Path;
                                        if let Some(parent) = Path::new(base_name).parent() {
                                            if let Some(pstr) = parent.to_str() { incl_dir = Some(pstr.to_string()); }
                                        }
                                    }
                                }

                                // 1) try relative to including file directory
                                if let Some(ref idir) = incl_dir {
                                use std::path::Path;
                                let cand = Path::new(idir).join(&name);
                                if let Ok(canon) = cand.canonicalize() {
                                    if let Some(pstr) = canon.to_str() {
                                        if let Some(toks) = crate::tokenize_file(pstr) {
                                            if let Some(included) = include_file(cur.clone(), pstr, toks.clone()) {
                                                cur_opt = Some(included);
                                                continue;
                                            }
                                        }
                                    }
                                }
                            }

                            // 2) try search include paths in order
                            let candidates = crate::search_include_paths_all(&name);
                            let mut included_any = false;
                            for cand in candidates.iter() {
                                if let Some(toks) = crate::tokenize_file(cand) {
                                    if let Some(included) = include_file(cur.clone(), cand, toks.clone()) {
                                        cur_opt = Some(included);
                                        included_any = true;
                                        break;
                                    }
                                }
                            }
                            if included_any { continue; }

                            // 3) last resort: raw filename from filesystem
                            if let Ok(contents) = fs::read_to_string(&name) {
                                let f = File { name: Some(name.clone()), file_no: 0, contents: Some(contents), display_name: Some(name.clone()), line_delta: 0 };
                                if let Some(toks) = tokenize(Rc::new(RefCell::new(f))) {
                                    if let Some(included) = include_file(cur.clone(), &name, toks.clone()) {
                                        cur_opt = Some(included);
                                        continue;
                                    }
                                }
                            }
                        }
                        cur_opt = next_tok;
                        continue;
                    }
                    "ifdef" => {
                        if parts.len() >= 2 {
                            if crate::preproc::get_macro(parts[1]).is_none() {
                                // skip to next token after the directive line (simple skip until #else/#endif)
                                // naive: just advance to next_tok
                                cur_opt = next_tok;
                                // then skip tokens until find a line starting with #else or #endif
                                while let Some(scur) = cur_opt.clone() {
                                    if scur.borrow().at_bol && scur.borrow().loc.as_ref().map(|l| l.starts_with('#')).unwrap_or(false) {
                                        if let Some(nn) = scur.borrow().next.clone() {
                                            if nn.borrow().kind == TokenKind::TK_IDENT {
                                                let d = nn.borrow().loc.clone().unwrap_or_default();
                                                if d == "else" || d == "endif" { cur_opt = nn.borrow().next.clone(); break; }
                                            }
                                        }
                                    }
                                    cur_opt = scur.borrow().next.clone();
                                }
                                continue;
                            }
                        }
                        cur_opt = next_tok;
                        continue;
                    }
                    "ifndef" => {
                        if parts.len() >= 2 {
                            if crate::preproc::get_macro(parts[1]).is_some() {
                                cur_opt = next_tok;
                                while let Some(scur) = cur_opt.clone() {
                                    if scur.borrow().at_bol && scur.borrow().loc.as_ref().map(|l| l.starts_with('#')).unwrap_or(false) {
                                        if let Some(nn) = scur.borrow().next.clone() {
                                            if nn.borrow().kind == TokenKind::TK_IDENT {
                                                let d = nn.borrow().loc.clone().unwrap_or_default();
                                                if d == "else" || d == "endif" { cur_opt = nn.borrow().next.clone(); break; }
                                            }
                                        }
                                    }
                                    cur_opt = scur.borrow().next.clone();
                                }
                                continue;
                            }
                        }
                        cur_opt = next_tok;
                        continue;
                    }
                    "if" => {
                        let (line2, _) = collect_directive_line(&cur);
                        let cond = eval_const_expr_line(&line2);
                        if !cond {
                            // skip until matching #else/#elif/#endif; handle nested conditionals
                            let mut depth: i32 = 0;
                            cur_opt = next_tok;
                            while let Some(scur) = cur_opt.clone() {
                                if scur.borrow().at_bol && scur.borrow().loc.as_ref().map(|l| l.starts_with('#')).unwrap_or(false) {
                                    if let Some(nn) = scur.borrow().next.clone() {
                                        if nn.borrow().kind == TokenKind::TK_IDENT {
                                            let d = nn.borrow().loc.clone().unwrap_or_default();
                                            if d == "if" { depth += 1; cur_opt = nn.borrow().next.clone(); continue; }
                                            if d == "endif" {
                                                if depth == 0 { cur_opt = nn.borrow().next.clone(); break; }
                                                depth -= 1;
                                                cur_opt = nn.borrow().next.clone(); continue;
                                            }
                                            if depth == 0 && (d == "else" || d == "elif") { cur_opt = nn.borrow().next.clone(); break; }
                                        }
                                    }
                                }
                                cur_opt = scur.borrow().next.clone();
                            }
                            continue;
                        }
                        cur_opt = next_tok;
                        continue;
                    }
                    "#include_next" => {
                        // Implement include_next: find candidate after the including directory
                        let (maybe_name, _nt) = read_include_filename(&cur);
                        if let Some(name) = maybe_name {
                                            // build including_file path (canonical) if possible
                                            let mut including_file_path: Option<String> = None;
                                            if let Some(file_rc) = &cur.borrow().file {
                                                if let Some(base_name) = &file_rc.borrow().name {
                                                    if let Ok(canon) = std::path::Path::new(base_name).canonicalize() {
                                                        if let Some(s) = canon.to_str() { including_file_path = Some(s.to_string()); }
                                                    }
                                                }
                                            }
                                            if let Some(path) = crate::search_include_next(&name, including_file_path.as_deref()) {
                                if let Some(toks) = crate::tokenize_file(&path) {
                                    if let Some(included) = include_file(cur.clone(), &path, toks.clone()) {
                                        cur_opt = Some(included);
                                        continue;
                                    }
                                }
                            }
                        }
                        cur_opt = next_tok;
                        continue;
                    }
                    _ => {
                        cur_opt = next_tok;
                        continue;
                    }
                }
            }
        }

        // not a directive; expand macros in the token and append to output
        let expanded = preprocess(Rc::clone(&cur)).unwrap_or_else(|| Rc::new(RefCell::new(Token::default())));
        // append expanded tokens up to EOF
        let mut ecur_opt = Some(expanded);
        while let Some(ecur) = ecur_opt.clone() {
            if ecur.borrow().kind == TokenKind::TK_EOF { break; }
            let nc = clone_token(&ecur);
            out_tail.borrow_mut().next = Some(nc.clone());
            out_tail = nc;
            ecur_opt = ecur.borrow().next.clone();
        }

        cur_opt = cur.borrow().next.clone();
    }

    dummy.borrow().next.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::tokenize_from_str;
    use crate::preproc::define_macro;

    #[test]
    fn simple_macro_expansion() {
        init_macros();
        define_macro("X", "1 + 2");
        let src = "X;";
        let tokens = tokenize_from_str(src).unwrap();
        let res = preprocess(tokens);
        // collect tokens into string for assertion
        let mut s = String::new();
        let mut cur = res;
        while let Some(t) = cur {
            let b = t.borrow();
            if let Some(loc) = &b.loc { s.push_str(loc); }
            if matches!(b.kind, TokenKind::TK_EOF) { break; }
            cur = b.next.clone();
        }
        assert!(s.contains("1"));
        assert!(s.contains("2"));
    }

    #[test]
    fn function_macro_simple() {
    init_macros();
    // define inc(x) -> x + 1
    crate::preproc::define_function_macro("inc", vec!["x".to_string()], "x + 1");
        let src = "inc(4);";
        let tokens = tokenize_from_str(src).unwrap();
        let res = preprocess(tokens).unwrap();
        let mut s = String::new();
        let mut cur = Some(res);
        while let Some(t) = cur {
            let b = t.borrow();
            if let Some(loc) = &b.loc { s.push_str(loc); }
            if matches!(b.kind, TokenKind::TK_EOF) { break; }
            cur = b.next.clone();
        }
        // should contain '4' and '1'
        assert!(s.contains("4"));
        assert!(s.contains("1"));
    }

    #[test]
    fn token_paste_and_rescan() {
        init_macros();
        // define X1 -> 100
        crate::preproc::define_macro("X1", "100");
        // define P(a,b) -> a ## b
        crate::preproc::define_function_macro("P", vec!["a".to_string(), "b".to_string()], "a ## b");

        let src = "P(X,1);";
        let tokens = tokenize_from_str(src).unwrap();
        let res = preprocess(tokens).unwrap();
        let mut s = String::new();
        let mut cur = Some(res);
        while let Some(t) = cur {
            let b = t.borrow();
            if let Some(loc) = &b.loc { s.push_str(loc); }
            if matches!(b.kind, TokenKind::TK_EOF) { break; }
            cur = b.next.clone();
        }
        // after paste+rescan, should contain the expansion of X1 -> 100
        assert!(s.contains("100"));
    }
}
