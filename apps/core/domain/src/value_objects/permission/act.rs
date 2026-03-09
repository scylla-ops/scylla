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

impl std::str::FromStr for Act {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "create" => Ok(Act::Create),
            "read" => Ok(Act::Read),
            "write" => Ok(Act::Write),
            "delete" => Ok(Act::Delete),
            _ => Ok(Act::All),
        }
    }
}
