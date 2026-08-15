//! Docker and container lifecycle management for `mag`.

use std::process::Command;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContainerError {
    #[error("Docker command failed: {0}")]
    CommandFailed(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct ContainerManager {
    docker_cmd: String,
}

impl Default for ContainerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerManager {
    pub fn new() -> Self {
        Self {
            docker_cmd: "docker".into(),
        }
    }

    pub fn is_docker_available(&self) -> bool {
        Command::new(&self.docker_cmd)
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn start_container(&self, container_name: &str) -> Result<(), ContainerError> {
        let output = Command::new(&self.docker_cmd)
            .args(["start", container_name])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(ContainerError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    pub fn stop_container(&self, container_name: &str) -> Result<(), ContainerError> {
        let output = Command::new(&self.docker_cmd)
            .args(["stop", container_name])
            .output()?;

        if output.status.success() {
            Ok(())
        } else {
            Err(ContainerError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    pub fn is_container_running(&self, container_name: &str) -> bool {
        let output = Command::new(&self.docker_cmd)
            .args(["inspect", "-f", "{{.State.Running}}", container_name])
            .output();

        match output {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim() == "true",
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_available_check() {
        let cm = ContainerManager::new();
        let _ = cm.is_docker_available(); // won't panic
    }
}
