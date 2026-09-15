use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// Shape only (five fields): parsing lives behind the `CronSchedule` port, keeping cron out of the kernel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CronSpec {
    expression: String,
}

impl CronSpec {
    pub fn new(expression: impl Into<String>) -> DomainResult<Self> {
        let raw = expression.into();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::validation("Cron expression cannot be empty"));
        }
        let field_count = trimmed.split_whitespace().count();
        if field_count != 5 {
            return Err(DomainError::validation(format!(
                "Cron expression must have exactly 5 fields (min hour dom mon dow), got {field_count}"
            )));
        }
        Ok(Self {
            expression: trimmed.to_string(),
        })
    }

    #[must_use]
    pub fn expression(&self) -> &str {
        &self.expression
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_five_field_expression() {
        let spec = CronSpec::new("0 9 * * 1-5").unwrap();
        assert_eq!(spec.expression(), "0 9 * * 1-5");
    }

    #[test]
    fn trims_and_normalizes() {
        assert_eq!(
            CronSpec::new("  */5 * * * *  ").unwrap().expression(),
            "*/5 * * * *"
        );
    }

    #[test]
    fn rejects_wrong_field_count() {
        assert!(CronSpec::new("* * * *").is_err());
        assert!(CronSpec::new("* * * * * *").is_err());
        assert!(CronSpec::new("   ").is_err());
    }
}
