use crate::domain::text::{Rule, Text};

pub enum AppSecretLabelRule {}

impl Rule for AppSecretLabelRule {
    const LABEL: &'static str = "App secret label";
    const MAX: usize = 64;
}

pub type AppSecretLabel = Text<AppSecretLabelRule>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creation_and_trimming() {
        assert!(AppSecretLabel::new("ci-runner").is_ok());
        assert_eq!(
            AppSecretLabel::new("  default  ").unwrap().as_str(),
            "default"
        );
        assert!(AppSecretLabel::new("").is_err());
        assert!(AppSecretLabel::new("   ").is_err());
    }

    #[test]
    fn enforces_length_bound() {
        assert!(AppSecretLabel::new("a".repeat(AppSecretLabelRule::MAX)).is_ok());
        assert!(AppSecretLabel::new("a".repeat(AppSecretLabelRule::MAX + 1)).is_err());
    }
}
