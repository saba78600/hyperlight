#[derive(Debug, Default)]
pub struct File {
    pub name: Option<String>,
    pub file_no: i32,
    pub contents: Option<String>,

    // For #line directive
    pub display_name: Option<String>,
    pub line_delta: i32,
}
