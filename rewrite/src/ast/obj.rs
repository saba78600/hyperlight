use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Obj {
    pub next: Option<Rc<RefCell<Obj>>>,
    pub name: Option<String>,
    pub ty: Option<Rc<RefCell<super::super::types::Type>>>,
    pub tok: Option<String>,
    pub is_local: bool,
    pub align: i32,
    // Local variable
    pub offset: i32,
    // Global variable or function
    pub is_function: bool,
    pub is_definition: bool,
    pub is_static: bool,
    // Global variable
    pub is_tentative: bool,
    pub is_tls: bool,
    pub init_data: Option<String>,
    pub rel: Option<Rc<RefCell<super::relocation::Relocation>>>,
    // Function
    pub is_inline: bool,
    pub params: Option<Rc<RefCell<Obj>>>,
    pub body: Option<Rc<RefCell<super::node::Node>>>,
    pub locals: Option<Rc<RefCell<Obj>>>,
    pub va_area: Option<Rc<RefCell<Obj>>>,
    pub alloca_bottom: Option<Rc<RefCell<Obj>>>,
    pub stack_size: i32,
    // Static inline function
    pub is_live: bool,
    pub is_root: bool,
    pub refs: super::super::strings::StringArray,
}
