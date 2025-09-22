use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Member {
    pub next: Option<Rc<RefCell<Member>>>,
    pub ty: Option<Rc<RefCell<super::super::types::Type>>>,
    // Token placeholder
    pub tok: Option<String>,
    pub name: Option<String>,
    pub idx: i32,
    pub align: i32,
    pub offset: i32,
    // Bitfield
    pub is_bitfield: bool,
    pub bit_offset: i32,
    pub bit_width: i32,
}
