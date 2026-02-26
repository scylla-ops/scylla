#[derive(Debug)]
pub enum Act {
    Create,
    Read,
    Write,
    Delete,
    All,
}

impl Act {
    pub fn as_str(&self) -> &'static str {
        match self {
            Act::Create => "create",
            Act::Read => "read",
            Act::Write => "write",
            Act::Delete => "delete",
            Act::All => "*",
        }
    }
}
