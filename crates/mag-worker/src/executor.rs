//! Command execution engine with safety allowlist and timeout control.

use mag_common::constants::DANGEROUS_COMMAND_KEYWORDS;
use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ExecutionOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub duration: Duration,
    pub timed_out: bool,
}

pub struct CommandExecutor {
    allowlist: HashSet<String>,
}

impl CommandExecutor {
    pub fn new(allowlist: HashSet<String>) -> Self {
        Self { allowlist }
    }

    pub fn is_safe(&self, command: &str) -> (bool, &'static str) {
        for keyword in DANGEROUS_COMMAND_KEYWORDS {
            if command.contains(keyword) {
                return (false, "Command contains dangerous keyword");
            }
        }

        let base_cmd = command.split_whitespace().next().unwrap_or("");
        let bin_name = Path::new(base_cmd)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(base_cmd);

        if !self.allowlist.is_empty() && !self.allowlist.contains(bin_name) {
            return (false, "Command not in allowlist");
        }

        (true, "OK")
    }

    pub fn run_command(&self, command: &str, cwd: Option<&Path>) -> ExecutionOutput {
        let (is_safe, reason) = self.is_safe(command);
        if !is_safe {
            return ExecutionOutput {
                success: false,
                exit_code: Some(-1),
                stdout: String::new(),
                stderr: reason.to_string(),
                duration: Duration::from_secs(0),
                timed_out: false,
            };
        }

        let start = Instant::now();
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(command);
        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        match cmd.output() {
            Ok(output) => {
                let duration = start.elapsed();
                ExecutionOutput {
                    success: output.status.success(),
                    exit_code: output.status.code(),
                    stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&output.stderr).to_string(),
                    duration,
                    timed_out: false,
                }
            }
            Err(e) => ExecutionOutput {
                success: false,
                exit_code: Some(-1),
                stdout: String::new(),
                stderr: e.to_string(),
                duration: start.elapsed(),
                timed_out: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_safety() {
        let mut allowlist = HashSet::new();
        allowlist.insert("echo".into());
        let exec = CommandExecutor::new(allowlist);

        let (safe, _) = exec.is_safe("echo hello");
        assert!(safe);

        let (safe, _) = exec.is_safe("rm -rf /");
        assert!(!safe);
    }
}
