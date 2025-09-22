use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Node {
    pub kind: super::node_kind::NodeKind,
    pub next: Option<Rc<RefCell<Node>>>,
    pub ty: Option<Rc<RefCell<super::super::types::Type>>>,
    pub tok: Option<String>,

    pub lhs: Option<Rc<RefCell<Node>>>,
    pub rhs: Option<Rc<RefCell<Node>>>,

    // if/for
    pub cond: Option<Rc<RefCell<Node>>>,
    pub then_branch: Option<Rc<RefCell<Node>>>,
    pub els: Option<Rc<RefCell<Node>>>,
    pub init: Option<Rc<RefCell<Node>>>,
    pub inc: Option<Rc<RefCell<Node>>>,

    pub brk_label: Option<String>,
    pub cont_label: Option<String>,

    pub body: Option<Rc<RefCell<Node>>>,
    pub member: Option<Rc<RefCell<super::member::Member>>>,

    pub func_ty: Option<Rc<RefCell<super::super::types::Type>>>,
    pub args: Option<Rc<RefCell<Node>>>,
    pub pass_by_stack: bool,
    pub ret_buffer: Option<Rc<RefCell<super::obj::Obj>>>,

    pub label: Option<String>,
    pub unique_label: Option<String>,
    pub goto_next: Option<Rc<RefCell<Node>>>,

    pub case_next: Option<Rc<RefCell<Node>>>,
    pub default_case: Option<Rc<RefCell<Node>>>,

    pub begin: i64,
    pub end: i64,

    pub asm_str: Option<String>,

    pub cas_addr: Option<Rc<RefCell<Node>>>,
    pub cas_old: Option<Rc<RefCell<Node>>>,
    pub cas_new: Option<Rc<RefCell<Node>>>,

    pub atomic_addr: Option<Rc<RefCell<super::obj::Obj>>>,
    pub atomic_expr: Option<Rc<RefCell<Node>>>,

    pub var: Option<Rc<RefCell<super::obj::Obj>>>,

    pub val: i64,
    pub fval: f64,
}

impl Default for Node {
    fn default() -> Self {
        Node {
            kind: super::node_kind::NodeKind::ND_NULL_EXPR,
            next: None,
            ty: None,
            tok: None,
            lhs: None,
            rhs: None,
            cond: None,
            then_branch: None,
            els: None,
            init: None,
            inc: None,
            brk_label: None,
            cont_label: None,
            body: None,
            member: None,
            func_ty: None,
            args: None,
            pass_by_stack: false,
            ret_buffer: None,
            label: None,
            unique_label: None,
            goto_next: None,
            case_next: None,
            default_case: None,
            begin: 0,
            end: 0,
            asm_str: None,
            cas_addr: None,
            cas_old: None,
            cas_new: None,
            atomic_addr: None,
            atomic_expr: None,
            var: None,
            val: 0,
            fval: 0.0,
        }
    }
}
