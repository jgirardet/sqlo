#[derive(Debug)]
pub enum Context {
    None,
    Call,
}

impl Default for Context {
    fn default() -> Self {
        Self::None
    }
}
