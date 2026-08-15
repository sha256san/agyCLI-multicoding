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

/// Dynamic Worker Pool scaling manager.
pub struct WorkerPoolManager {
    container_mgr: ContainerManager,
}

impl Default for WorkerPoolManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkerPoolManager {
    pub fn new() -> Self {
        Self {
            container_mgr: ContainerManager::new(),
        }
    }

    pub fn scale_workers(&self, target_count: usize) -> Result<usize, ContainerError> {
        let roles = ["developer", "tester", "reviewer", "security", "researcher"];
        let mut active = 0;

        for i in 0..target_count {
            let role = roles[i % roles.len()];
            let container_name = format!("mag-{}-{}", role, i + 1);

            if self.container_mgr.is_docker_available() {
                let _ = self.container_mgr.start_container(&container_name);
            }
            active += 1;
        }

        Ok(active)
    }

    pub fn get_pool_status(&self, total_configured: usize) -> Vec<(String, String, bool)> {
        let roles = ["developer", "tester", "reviewer", "security", "researcher"];
        let mut status = Vec::new();

        for i in 0..total_configured {
            let role = roles[i % roles.len()];
            let container_name = format!("mag-{}-{}", role, i + 1);
            let is_running = if self.container_mgr.is_docker_available() {
                self.container_mgr.is_container_running(&container_name)
            } else {
                false
            };
            status.push((container_name, role.to_string(), is_running));
        }

        status
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
