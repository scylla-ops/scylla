use crate::domain::text::{Rule, Text};

pub enum ProjectDescriptionRule {}

impl Rule for ProjectDescriptionRule {
    const LABEL: &'static str = "Description";
    const MAX: usize = 1024;
    const REQUIRED: bool = false;
}

pub type ProjectDescription = Text<ProjectDescriptionRule>;
