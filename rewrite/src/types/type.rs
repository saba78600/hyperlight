use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Type {
    pub kind: super::type_kind::TypeKind,
    pub size: i32,
    pub align: i32,
    pub is_unsigned: bool,
    pub is_atomic: bool,
    pub origin: Option<Rc<RefCell<Type>>>,
    pub base: Option<Rc<RefCell<Type>>>,
    // Declaration
    pub name: Option<String>,
    pub name_pos: Option<String>,
    // Array
    pub array_len: i32,
    // VLA
    pub vla_len: Option<Rc<RefCell<super::super::ast::Node>>>,
    pub vla_size: Option<Rc<RefCell<super::super::ast::Obj>>>,
    // Struct
    pub members: Option<Rc<RefCell<super::super::ast::Member>>>,
    pub is_flexible: bool,
    pub is_packed: bool,
    // Function type
    pub return_ty: Option<Rc<RefCell<Type>>>,
    pub params: Option<Rc<RefCell<Type>>>,
    pub is_variadic: bool,
    pub next: Option<Rc<RefCell<Type>>>,
}

impl Default for Type {
    fn default() -> Self {
        Type {
            kind: super::type_kind::TypeKind::TY_VOID,
            size: 0,
            align: 0,
            is_unsigned: false,
            is_atomic: false,
            origin: None,
            base: None,
            name: None,
            name_pos: None,
            array_len: 0,
            vla_len: None,
            vla_size: None,
            members: None,
            is_flexible: false,
            is_packed: false,
            return_ty: None,
            params: None,
            is_variadic: false,
            next: None,
        }
    }
}

// Constructors for common primitive types (mirrors C externs)
pub fn ty_void() -> std::rc::Rc<std::cell::RefCell<Type>> {
    std::rc::Rc::new(std::cell::RefCell::new(Type { kind: super::type_kind::TypeKind::TY_VOID, ..Default::default() }))
}

pub fn ty_int() -> std::rc::Rc<std::cell::RefCell<Type>> {
    std::rc::Rc::new(std::cell::RefCell::new(Type { kind: super::type_kind::TypeKind::TY_INT, size: 4, align: 4, ..Default::default() }))
}

pub fn ty_char() -> std::rc::Rc<std::cell::RefCell<Type>> {
    std::rc::Rc::new(std::cell::RefCell::new(Type { kind: super::type_kind::TypeKind::TY_CHAR, size: 1, align: 1, ..Default::default() }))
}

pub fn ty_float() -> std::rc::Rc<std::cell::RefCell<Type>> {
    std::rc::Rc::new(std::cell::RefCell::new(Type { kind: super::type_kind::TypeKind::TY_FLOAT, size: 4, align: 4, ..Default::default() }))
}

pub fn ty_double() -> std::rc::Rc<std::cell::RefCell<Type>> {
    std::rc::Rc::new(std::cell::RefCell::new(Type { kind: super::type_kind::TypeKind::TY_DOUBLE, size: 8, align: 8, ..Default::default() }))
}

// Type helper implementations
pub fn pointer_to(base: std::rc::Rc<std::cell::RefCell<Type>>) -> std::rc::Rc<std::cell::RefCell<Type>> {
    let mut t = Type::default();
    t.kind = super::type_kind::TypeKind::TY_PTR;
    t.base = Some(base.clone());
    t.size = std::mem::size_of::<*const ()>() as i32;
    t.align = std::mem::align_of::<*const ()>() as i32;
    std::rc::Rc::new(std::cell::RefCell::new(t))
}

pub fn func_type(return_ty: std::rc::Rc<std::cell::RefCell<Type>>) -> std::rc::Rc<std::cell::RefCell<Type>> {
    let mut t = Type::default();
    t.kind = super::type_kind::TypeKind::TY_FUNC;
    t.return_ty = Some(return_ty.clone());
    std::rc::Rc::new(std::cell::RefCell::new(t))
}

pub fn copy_type(ty: std::rc::Rc<std::cell::RefCell<Type>>) -> std::rc::Rc<std::cell::RefCell<Type>> {
    let orig = ty.borrow();
    let mut t = Type::default();
    t.kind = orig.kind;
    t.size = orig.size;
    t.align = orig.align;
    t.is_unsigned = orig.is_unsigned;
    t.is_atomic = orig.is_atomic;
    t.origin = orig.origin.clone();
    t.base = orig.base.clone();
    t.name = orig.name.clone();
    t.name_pos = orig.name_pos.clone();
    t.array_len = orig.array_len;
    t.vla_len = orig.vla_len.clone();
    t.vla_size = orig.vla_size.clone();
    t.members = orig.members.clone();
    t.is_flexible = orig.is_flexible;
    t.is_packed = orig.is_packed;
    t.return_ty = orig.return_ty.clone();
    t.params = orig.params.clone();
    t.is_variadic = orig.is_variadic;
    t.next = orig.next.clone();
    std::rc::Rc::new(std::cell::RefCell::new(t))
}

pub fn is_integer(ty: std::rc::Rc<std::cell::RefCell<Type>>) -> bool {
    match ty.borrow().kind {
        super::type_kind::TypeKind::TY_CHAR
        | super::type_kind::TypeKind::TY_SHORT
        | super::type_kind::TypeKind::TY_INT
        | super::type_kind::TypeKind::TY_LONG => true,
        _ => false,
    }
}

pub fn is_flonum(ty: std::rc::Rc<std::cell::RefCell<Type>>) -> bool {
    match ty.borrow().kind {
        super::type_kind::TypeKind::TY_FLOAT
        | super::type_kind::TypeKind::TY_DOUBLE
        | super::type_kind::TypeKind::TY_LDOUBLE => true,
        _ => false,
    }
}

pub fn is_numeric(ty: std::rc::Rc<std::cell::RefCell<Type>>) -> bool {
    is_integer(ty.clone()) || is_flonum(ty)
}

pub fn array_of(base: std::rc::Rc<std::cell::RefCell<Type>>, len: i32) -> std::rc::Rc<std::cell::RefCell<Type>> {
    let mut t = Type::default();
    t.kind = super::type_kind::TypeKind::TY_ARRAY;
    t.base = Some(base.clone());
    t.array_len = len;
    // naive size calculation
    t.size = base.borrow().size * len;
    t.align = base.borrow().align;
    std::rc::Rc::new(std::cell::RefCell::new(t))
}

pub fn vla_of(base: std::rc::Rc<std::cell::RefCell<Type>>, expr: std::rc::Rc<std::cell::RefCell<super::super::ast::Node>>) -> std::rc::Rc<std::cell::RefCell<Type>> {
    let mut t = Type::default();
    t.kind = super::type_kind::TypeKind::TY_VLA;
    t.base = Some(base.clone());
    t.vla_len = Some(expr.clone());
    // size unknown until runtime
    t.size = 0;
    t.align = base.borrow().align;
    std::rc::Rc::new(std::cell::RefCell::new(t))
}

pub fn enum_type() -> std::rc::Rc<std::cell::RefCell<Type>> {
    let mut t = Type::default();
    t.kind = super::type_kind::TypeKind::TY_ENUM;
    t.size = 4;
    t.align = 4;
    std::rc::Rc::new(std::cell::RefCell::new(t))
}

pub fn struct_type() -> std::rc::Rc<std::cell::RefCell<Type>> {
    let mut t = Type::default();
    t.kind = super::type_kind::TypeKind::TY_STRUCT;
    t.size = 0;
    t.align = 1;
    std::rc::Rc::new(std::cell::RefCell::new(t))
}

pub fn add_type(_node: std::rc::Rc<std::cell::RefCell<super::super::ast::Node>>) {
    // No-op placeholder: in C this populates node->ty for AST; we will
    // implement this when porting the parser.
}
