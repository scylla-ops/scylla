use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shell {
    #[default]
    Sh,
    Bash,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Step {
    Exec { command: String, args: Vec<String> },
    Script { script: String, shell: Shell },
}

impl Step {
    pub fn exec(command: String, args: Vec<String>) -> DomainResult<Self> {
        if command.trim().is_empty() {
            return Err(DomainError::validation("Exec command cannot be empty"));
        }
        Ok(Self::Exec { command, args })
    }

    pub fn script(script: String, shell: Shell) -> DomainResult<Self> {
        if script.trim().is_empty() {
            return Err(DomainError::validation("Script cannot be empty"));
        }
        Ok(Self::Script { script, shell })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_rejects_blank_command() {
        assert!(Step::exec("   ".into(), vec![]).is_err());
        assert!(Step::exec("echo".into(), vec!["hi".into()]).is_ok());
    }

    #[test]
    fn script_rejects_blank_script() {
        assert!(Step::script("  \n ".into(), Shell::Sh).is_err());
        assert!(Step::script("echo hi".into(), Shell::Bash).is_ok());
    }

    #[test]
    fn step_json_is_tagged() {
        let exec = Step::exec("ls".into(), vec!["-la".into()]).unwrap();
        let json = serde_json::to_string(&exec).unwrap();
        assert!(json.contains(r#""kind":"exec""#), "{json}");

        let script = Step::script("make".into(), Shell::Sh).unwrap();
        let json = serde_json::to_string(&script).unwrap();
        assert!(json.contains(r#""kind":"script""#), "{json}");
        assert!(json.contains(r#""shell":"sh""#), "{json}");
    }
}
