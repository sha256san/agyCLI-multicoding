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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContainerInfo {
    pub name: String,
    pub role: String,
    pub status: String,
    pub image: String,
    pub is_running: bool,
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

    pub fn list_containers(&self, root_path: &std::path::Path) -> Vec<ContainerInfo> {
        let mut list = Vec::new();

        // 1. Query docker CLI if accessible
        if let Ok(output) = Command::new(&self.docker_cmd)
            .args(["ps", "--format", "{{.Names}}\t{{.Image}}\t{{.Status}}"])
            .output()
        {
            if output.status.success() {
                let stdout_str = String::from_utf8_lossy(&output.stdout);
                for line in stdout_str.lines() {
                    let parts: Vec<&str> = line.split('\t').collect();
                    if !parts.is_empty() && !parts[0].trim().is_empty() {
                        let name = parts[0].trim().to_string();
                        let image = parts.get(1).unwrap_or(&"").trim().to_string();
                        let status = parts.get(2).unwrap_or(&"Up").trim().to_string();
                        let role = if name.contains("developer") || name.contains("agent-a") {
                            "developer"
                        } else if name.contains("tester") || name.contains("agent-b") {
                            "tester"
                        } else if name.contains("reviewer") || name.contains("agent-c") {
                            "reviewer"
                        } else if name.contains("security") || name.contains("agent-d") {
                            "security"
                        } else if name.contains("researcher") || name.contains("agent-e") {
                            "researcher"
                        } else {
                            "worker"
                        };

                        list.push(ContainerInfo {
                            name,
                            role: role.into(),
                            status,
                            image,
                            is_running: true,
                        });
                    }
                }
            }
        }

        // 2. Complement with active authenticated and configured agent containers
        let logged_in = mag_config::get_logged_in_agents(root_path);
        let configured_roles = [
            ("agent-a", "developer", "agycli-developer:latest"),
            ("agent-b", "tester", "agycli-tester:latest"),
            ("agent-c", "reviewer", "agycli-reviewer:latest"),
            ("agent-d", "security", "agycli-security:latest"),
            ("agent-e", "researcher", "agycli-researcher:latest"),
            ("cnt-a", "collaborative", "agycli-worker:latest"),
        ];

        for (name, role, img) in &configured_roles {
            if !list.iter().any(|c| c.name == *name || c.name == format!("mag-{}", name)) {
                let is_auth = logged_in.iter().any(|(n, _)| n == *name);
                let status_str = if is_auth { "READY / STANDBY" } else { "STOPPED" };
                list.push(ContainerInfo {
                    name: name.to_string(),
                    role: role.to_string(),
                    status: status_str.to_string(),
                    image: img.to_string(),
                    is_running: is_auth,
                });
            }
        }

        list
    }
}

/// Dynamic Worker Pool scaling manager.
pub struct WorkerPoolManager {
    pub container_mgr: ContainerManager,
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

    pub fn list_containers(&self, root_path: &std::path::Path) -> Vec<ContainerInfo> {
        self.container_mgr.list_containers(root_path)
    }

    pub fn exec_container_command(&self, container_name: &str, cmd: &[&str]) -> Result<String, ContainerError> {
        let mut args = vec!["exec", container_name];
        args.extend_from_slice(cmd);

        let output = Command::new(&self.container_mgr.docker_cmd)
            .args(&args)
            .output()?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(ContainerError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }

    pub fn ensure_container_auth_directory(&self, root_path: &std::path::Path, container_name: &str) {
        let clean_name = container_name.trim_start_matches("mag-");
        let dir = root_path.join(".mag/containers").join(clean_name);
        let _ = std::fs::create_dir_all(&dir);
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
