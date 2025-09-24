use inkwell::context::Context;

pub struct ContextOwner(pub(crate) *mut Context);

impl Drop for ContextOwner {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.0));
        }
    }
}
