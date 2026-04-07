#[derive(Debug)]
pub enum Act {
    Create,
    Read,
    Write,
    Delete,
    Execute,
    All,
}

impl Act {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Act::Create => "create",
            Act::Read => "read",
            Act::Write => "write",
            Act::Delete => "delete",
            Act::Execute => "execute",
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
            "execute" => Ok(Act::Execute),
            _ => Ok(Act::All),
        }
    }
}
