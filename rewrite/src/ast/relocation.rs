use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Relocation {
    pub next: Option<Rc<RefCell<Relocation>>>,
    pub offset: i32,
    pub label: Option<Vec<String>>,
    pub addend: i64,
}
