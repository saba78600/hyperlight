#[derive(Debug, Default)]
pub struct StringArray {
    pub data: Vec<String>,
    pub capacity: usize,
}

impl StringArray {
    pub fn push(&mut self, s: String) {
        self.data.push(s);
        self.capacity = self.data.capacity();
    }
}
