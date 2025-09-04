use protocol::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Shell {
    Sh,
    Bash,
    Pwsh,
    Powershell,
    Cmd,
}

impl From<Shell> for Command {
    fn from(value: Shell) -> Self {
        match value {
            Shell::Sh => {
                let mut cmd = Command::new("sh");
                cmd.arg("-c");
                cmd
            }
            Shell::Bash => {
                let mut cmd = Command::new("bash");
                cmd.arg("-c");
                cmd
            }
            Shell::Pwsh => {
                let mut cmd = Command::new("pwsh");
                cmd.arg("-Command");
                cmd
            }
            Shell::Powershell => {
                let mut cmd = Command::new("powershell");
                cmd.arg("-Command");
                cmd
            }
            Shell::Cmd => {
                let mut cmd = Command::new("cmd");
                cmd.arg("/C");
                cmd
            }
        }
    }
}
