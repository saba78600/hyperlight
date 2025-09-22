use crate::token::{File, Token, TokenKind};
use crate::types;
use std::cell::RefCell;
use std::rc::Rc;
use crate::strings::intern;

use super::ident;
use super::number;
use super::punct;
use super::literal;
use crate::token::keywords;

use ident::read_ident;
use number::read_number;
use punct::read_punct;
use literal::{decode_string, decode_char};

fn new_token(file: &Rc<RefCell<File>>, kind: TokenKind, text: &str, start: usize, end: usize, line_no: i32) -> Rc<RefCell<Token>> {
    let mut tok = Token::default();
    tok.kind = kind;
    tok.loc = Some(text[start..end].to_string());
    tok.len = (end - start) as i32;
    tok.file = Some(file.clone());
    // prefer display_name over name
    let filename = file.borrow().display_name.clone().or_else(|| file.borrow().name.clone());
    tok.filename = filename;
    tok.line_no = line_no;
    Rc::new(RefCell::new(tok))
}

fn is_keyword(s: &str) -> bool {
    // Inline keyword list to match behavior in tokenize.c
    matches!(s,
        "return"|"if"|"else"|"for"|"while"|"int"|"sizeof"|"char"|
        "struct"|"union"|"short"|"long"|"void"|"typedef"|"_Bool"|
        "enum"|"static"|"goto"|"break"|"continue"|"switch"|"case"|
        "default"|"extern"|"_Alignof"|"_Alignas"|"do"|"signed"|
        "unsigned"|"const"|"volatile"|"auto"|"register"|"restrict"|
        "__restrict"|"__restrict__"|"_Noreturn"|"float"|"double"|
        "typeof"|"asm"|"_Thread_local"|"__thread"|"_Atomic"|
        "__attribute__"
    )
}

pub fn tokenize(file: Rc<RefCell<File>>) -> Option<Rc<RefCell<Token>>> {
    let s = match &file.borrow().contents {
        Some(c) => c.clone(),
        None => return None,
    };
    let mut head: Option<Rc<RefCell<Token>>> = None;
    let mut tail: Option<Rc<RefCell<Token>>> = None;

    let mut i = 0usize;
    let bs = s.as_bytes();
    let mut line_no = 1i32;

    while i < bs.len() {
        let c = bs[i];
        // whitespace
        if c == b'\n' {
            line_no += 1;
            i += 1;
            continue;
        }
        if c == b' ' || c == b'\t' || c == b'\r' || c == b'\x0c' || c == b'\x0b' {
            i += 1;
            continue;
        }

        // comments: // or /* */
        if c == b'/' {
            if i + 1 < bs.len() {
                if bs[i + 1] == b'/' {
                    // line comment
                    i += 2;
                    while i < bs.len() && bs[i] != b'\n' { i += 1; }
                    continue;
                } else if bs[i + 1] == b'*' {
                    i += 2;
                    while i + 1 < bs.len() && !(bs[i] == b'*' && bs[i + 1] == b'/') { i += 1; }
                    i += 2;
                    continue;
                }
            }
        }

        // identifier or keyword
        let id_len = read_ident(&s, i);
        if id_len > 0 {
            let tok = new_token(&file, TokenKind::TK_IDENT, &s, i, i + id_len, line_no);
            // keyword check
            let text = &s[i..i+id_len];
            if keywords::is_keyword(text) {
                tok.borrow_mut().kind = TokenKind::TK_KEYWORD;
            }
            // intern identifier text (store in loc and keep str_lit for strings)
            let ident_intern = intern(text);
            tok.borrow_mut().loc = Some(ident_intern.as_ref().clone());
            if head.is_none() { head = Some(tok.clone()); } else { tail.as_ref().unwrap().borrow_mut().next = Some(tok.clone()); }
            tail = Some(tok);
            i += id_len;
            continue;
        }

        // number
        let num_len = read_number(&s, i);
        if num_len > 0 {
            // basic integer parsing with base detection
            let mut j = i + num_len;
            let mut base = 10u32;
            if s.as_bytes().get(i) == Some(&b'0') && j < bs.len() {
                match bs.get(i + 1) {
                    Some(b'x') | Some(b'X') => { base = 16; j = i + 2; while j < bs.len() && (bs[j] as char).is_ascii_hexdigit() { j += 1; } }
                    Some(b'b') | Some(b'B') => { base = 2; j = i + 2; while j < bs.len() && (bs[j] == b'0' || bs[j] == b'1') { j += 1; } }
                    Some(b'0'..=b'9') => { base = 8; j = i + 1; while j < bs.len() && (b'0' <= bs[j] && bs[j] <= b'7') { j += 1; } }
                    _ => { base = 10; }
                }
            }
            // read suffix letters (U, L, etc.) but we won't use them for now
            let mut k = j;
            while k < bs.len() && ((bs[k] as char).is_ascii_alphabetic()) { k += 1; }
            let lit = &s[i..k];
            // parse numeric value best-effort
            let mut value: i64 = 0;
            if base == 10 {
                if let Ok(v) = lit.parse::<i64>() { value = v; }
            } else if base == 16 {
                if let Ok(v) = i64::from_str_radix(&lit[2..], 16) { value = v; }
            } else if base == 2 {
                if let Ok(v) = i64::from_str_radix(&lit[2..], 2) { value = v; }
            } else if base == 8 {
                if let Ok(v) = i64::from_str_radix(&lit[1..], 8) { value = v; }
            }
            let mut tok = Token::default();
            tok.kind = TokenKind::TK_NUM;
            tok.loc = Some(lit.to_string());
            tok.len = (k - i) as i32;
            tok.val = value;
            tok.filename = file.borrow().display_name.clone().or_else(|| file.borrow().name.clone());
            tok.file = Some(file.clone());
            tok.line_no = line_no;
            let rc = Rc::new(RefCell::new(tok));
            if head.is_none() { head = Some(rc.clone()); } else { tail.as_ref().unwrap().borrow_mut().next = Some(rc.clone()); }
            tail = Some(rc);
            i = k;
            continue;
        }

        // string literal
        if c == b'"' {
            let start = i;
            i += 1;
            let (buf, new_i) = decode_string(bs, i);
            i = new_i;
            if i < bs.len() && bs[i] == b'"' { i += 1; }
            let tok = new_token(&file, TokenKind::TK_STR, &s, start, i, line_no);
            // intern the string literal contents
            let str_intern = intern(&buf);
            tok.borrow_mut().str_lit = Some(str_intern.as_ref().clone());
            // set type to char array
            let arr = types::r#type::array_of(types::r#type::ty_char(), (buf.len() + 1) as i32);
            tok.borrow_mut().ty = Some(arr);
            if head.is_none() { head = Some(tok.clone()); } else { tail.as_ref().unwrap().borrow_mut().next = Some(tok.clone()); }
            tail = Some(tok);
            continue;
        }

        // char literal
        if c == b'\'' {
            let start = i;
            i += 1;
            let (val, new_i) = decode_char(bs, i);
            i = new_i;
            if i < bs.len() && bs[i] == b'\'' { i += 1; }
            let mut tok = Token::default();
            tok.kind = TokenKind::TK_NUM;
            tok.val = val;
            tok.loc = Some(s[start..i].to_string());
            tok.len = (i - start) as i32;
            tok.filename = file.borrow().display_name.clone().or_else(|| file.borrow().name.clone());
            tok.file = Some(file.clone());
            tok.line_no = line_no;
            tok.ty = Some(types::r#type::ty_int());
            let rc = Rc::new(RefCell::new(tok));
            if head.is_none() { head = Some(rc.clone()); } else { tail.as_ref().unwrap().borrow_mut().next = Some(rc.clone()); }
            tail = Some(rc);
            continue;
        }
        // punctuator
        let p_len = read_punct(&s, i);
        if p_len > 0 {
            let tok = new_token(&file, TokenKind::TK_PUNCT, &s, i, i + p_len, line_no);
            if head.is_none() { head = Some(tok.clone()); } else { tail.as_ref().unwrap().borrow_mut().next = Some(tok.clone()); }
            tail = Some(tok);
            i += p_len;
            continue;
        }

        // unknown char: treat as single-char punct
        let tok = new_token(&file, TokenKind::TK_PUNCT, &s, i, i + 1, line_no);
        if head.is_none() { head = Some(tok.clone()); } else { tail.as_ref().unwrap().borrow_mut().next = Some(tok.clone()); }
        tail = Some(tok);
        i += 1;
    }

    // append EOF
    let eof = new_token(&file, TokenKind::TK_EOF, &s, s.len(), s.len(), line_no);
    if head.is_none() { head = Some(eof.clone()); } else { tail.as_ref().unwrap().borrow_mut().next = Some(eof.clone()); }
    Some(head.unwrap())
}
