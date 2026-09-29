use crate::domain::text::{Rule, Text};

pub enum OrganizationDescriptionRule {}

impl Rule for OrganizationDescriptionRule {
    const LABEL: &'static str = "Description";
    const MAX: usize = 1024;
    const REQUIRED: bool = false;
}

pub type OrganizationDescription = Text<OrganizationDescriptionRule>;
