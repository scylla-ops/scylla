use crate::domain::text::{Rule, Text};

pub enum RoleNameRule {}

impl Rule for RoleNameRule {
    const LABEL: &'static str = "Role name";
    const MAX: usize = 255;
}

pub type RoleName = Text<RoleNameRule>;
