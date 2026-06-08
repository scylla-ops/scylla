use crate::domain::errors::{DomainError, DomainResult};
use serde::{Deserialize, Serialize};

/// Which shell interprets a [`Step::Script`]. Runs with fail-fast semantics so a
/// failing line aborts the script with a non-zero exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Shell {
    /// POSIX `/bin/sh` — always present in the agent image.
    #[default]
    Sh,
    /// `/bin/bash` — requires bash in the agent base image.
    Bash,
}

/// What a pipeline node actually runs. Either a direct process exec (no shell,
/// deterministic, injection-proof) or a shell script (the ergonomic default).
///
/// Serialized with an internal `kind` tag so the persisted JSONB blob is
/// self-describing: `{"kind":"exec",...}` / `{"kind":"script",...}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Step {
    /// `command` resolved via PATH, `args` a literal argv vector.
    Exec { command: String, args: Vec<String> },
    /// A (possibly multi-line) shell script run via `shell`.
    Script { script: String, shell: Shell },
}

impl Step {
    /// Build a direct-exec step. The command must be non-empty.
    pub fn exec(command: String, args: Vec<String>) -> DomainResult<Self> {
        if command.trim().is_empty() {
            return Err(DomainError::validation("Exec command cannot be empty"));
        }
        Ok(Self::Exec { command, args })
    }

    /// Build a shell-script step. The script must be non-empty.
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
