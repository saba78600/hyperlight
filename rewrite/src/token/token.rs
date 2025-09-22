use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    TK_IDENT,
    TK_PUNCT,
    TK_KEYWORD,
    TK_STR,
    TK_NUM,
    TK_PP_NUM,
    TK_EOF,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub next: Option<Rc<RefCell<Token>>>,
    pub val: i64,
    pub fval: f64,
    pub loc: Option<String>,
    pub len: i32,
    pub ty: Option<Rc<RefCell<super::super::types::Type>>>,
    pub str_lit: Option<String>,

    pub file: Option<Rc<RefCell<super::file::File>>>,
    pub filename: Option<String>,
    pub line_no: i32,
    pub line_delta: i32,
    pub at_bol: bool,
    pub has_space: bool,
    pub origin: Option<Rc<RefCell<Token>>>,
    // For the preprocessor: a linked list of macro names which hid this token
    pub hideset: Option<Rc<RefCell<Hideset>>>,
}

#[derive(Debug, Clone)]
pub struct Hideset {
    pub next: Option<Rc<RefCell<Hideset>>>,
    pub name: String,
}

impl Default for Token {
    fn default() -> Self {
        Token {
            kind: TokenKind::TK_EOF,
            next: None,
            val: 0,
            fval: 0.0,
            loc: None,
            len: 0,
            ty: None,
            str_lit: None,
            file: None,
            filename: None,
            line_no: 0,
            line_delta: 0,
            at_bol: false,
            has_space: false,
            origin: None,
            hideset: None,
        }
    }
}
