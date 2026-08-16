//! Configuration models and loader for `mag`.

use mag_common::AgentRole;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("File I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML decode error: {0}")]
    TomlDecode(#[from] toml::de::Error),
    #[error("TOML encode error: {0}")]
    TomlEncode(#[from] toml::ser::Error),
}

/// Project root configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub manager: ManagerConfig,
    #[serde(default)]
    pub pool: mag_common::WorkerPoolConfig,
    #[serde(default)]
    pub agents: HashMap<String, AgentEndpointConfig>,
}

fn default_version() -> String {
    "0.1.0".into()
}

fn default_language() -> String {
    "rust".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_manager_port")]
    pub port: u16,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_task_timeout")]
    pub task_timeout: u64,
}

impl Default for ManagerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_manager_port(),
            max_retries: default_max_retries(),
            task_timeout: default_task_timeout(),
        }
    }
}

fn default_host() -> String {
    "127.0.0.1".into()
}
fn default_manager_port() -> u16 {
    8000
}
fn default_max_retries() -> u32 {
    3
}
fn default_task_timeout() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentEndpointConfig {
    pub id: String,
    pub role: AgentRole,
    #[serde(default = "default_host")]
    pub host: String,
    pub port: u16,
}

/// Agent profile definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub allowed_commands: Vec<String>,
}

impl ProjectConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)?;
        let config: ProjectConfig = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    pub fn default_project(name: &str) -> Self {
        let mut agents = HashMap::new();
        agents.insert(
            "developer".into(),
            AgentEndpointConfig {
                id: "agent-a".into(),
                role: AgentRole::Developer,
                host: "127.0.0.1".into(),
                port: 8001,
            },
        );
        agents.insert(
            "tester".into(),
            AgentEndpointConfig {
                id: "agent-b".into(),
                role: AgentRole::Tester,
                host: "127.0.0.1".into(),
                port: 8002,
            },
        );
        agents.insert(
            "reviewer".into(),
            AgentEndpointConfig {
                id: "agent-c".into(),
                role: AgentRole::Reviewer,
                host: "127.0.0.1".into(),
                port: 8003,
            },
        );
        agents.insert(
            "security".into(),
            AgentEndpointConfig {
                id: "agent-d".into(),
                role: AgentRole::Security,
                host: "127.0.0.1".into(),
                port: 8004,
            },
        );
        agents.insert(
            "researcher".into(),
            AgentEndpointConfig {
                id: "agent-e".into(),
                role: AgentRole::Researcher,
                host: "127.0.0.1".into(),
                port: 8005,
            },
        );

        Self {
            name: name.into(),
            version: "0.1.0".into(),
            language: "rust".into(),
            manager: ManagerConfig::default(),
            pool: mag_common::WorkerPoolConfig::default(),
            agents,
        }
    }
}

pub fn load_auth_config<P: AsRef<Path>>(path: P) -> Option<mag_common::AuthConfig> {
    if let Ok(content) = fs::read_to_string(path) {
        serde_json::from_str(&content).ok()
    } else {
        None
    }
}

pub fn save_auth_config<P: AsRef<Path>>(path: P, auth: &mag_common::AuthConfig) -> Result<(), std::io::Error> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(auth)?;
    fs::write(path, json)
}

fn fetch_email_from_token(access_token: &str) -> Option<String> {
    if !access_token.starts_with("ya29.") {
        return None;
    }
    let url = format!("https://oauth2.googleapis.com/tokeninfo?access_token={}", access_token);
    let output = std::process::Command::new("curl")
        .args(["-s", "--max-time", "3", &url])
        .output()
        .ok()?;
    if output.status.success() {
        let content = String::from_utf8_lossy(&output.stdout);
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(email) = val.get("email").and_then(|v| v.as_str()) {
                return Some(email.to_string());
            }
        }
    }
    None
}

pub fn load_container_auth<P: AsRef<Path>>(root: P, container_name: &str) -> Option<mag_common::AuthConfig> {
    let clean_name = container_name.trim_start_matches("mag-");
    let container_dir = root.as_ref().join(".mag/containers").join(clean_name);
    let path = container_dir.join("credentials.json");
    let mut auth = load_auth_config(&path)?;

    // Check if we need to resolve the real email from oauth token
    let needs_email_resolve = auth.user.as_ref().map(|u| {
        u.email.as_ref().map(|e| e.starts_with("user-agent") || e.starts_with("user-") || e == "N/A").unwrap_or(true)
    }).unwrap_or(true);

    if needs_email_resolve {
        let token_path = container_dir.join("home/.gemini/antigravity-cli/antigravity-oauth-token");
        if token_path.exists() {
            if let Ok(token_content) = fs::read_to_string(&token_path) {
                if let Ok(token_json) = serde_json::from_str::<serde_json::Value>(&token_content) {
                    if let Some(access_token) = token_json.get("token").and_then(|t| t.get("access_token")).and_then(|v| v.as_str()) {
                        if let Some(real_email) = fetch_email_from_token(access_token) {
                            if let Some(ref mut u) = auth.user {
                                u.email = Some(real_email);
                            }
                            if let Some(ref mut t) = auth.token {
                                t.access_token = access_token.to_string();
                            }
                            let _ = save_auth_config(&path, &auth);
                        }
                    }
                }
            }
        }
    }

    Some(auth)
}

pub fn save_container_auth<P: AsRef<Path>>(root: P, container_name: &str, auth: &mag_common::AuthConfig) -> Result<(), std::io::Error> {
    let clean_name = container_name.trim_start_matches("mag-");
    let path = root.as_ref().join(".mag/containers").join(clean_name).join("credentials.json");
    save_auth_config(path, auth)
}

pub fn get_logged_in_agents<P: AsRef<Path>>(root: P) -> Vec<(String, mag_common::AuthConfig)> {
    let mut list = Vec::new();
    let containers_dir = root.as_ref().join(".mag/containers");
    if let Ok(entries) = fs::read_dir(containers_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(auth) = load_container_auth(root.as_ref(), &name) {
                    list.push((name, auth));
                }
            }
        }
    }
    list
}

pub fn update_agent_md<P: AsRef<Path>>(root: P) -> Result<usize, std::io::Error> {
    let agents = get_logged_in_agents(root.as_ref());
    let mut md = String::new();
    md.push_str("# Active Logged-In Agents (`agent.md`)\n\n");
    md.push_str("This file is automatically maintained by `agycli` and records all authenticated agents and their assigned accounts.\n\n");
    md.push_str(&format!("**Total Active Authenticated Agents:** {}\n\n", agents.len()));
    md.push_str("| Agent Name | Account (Email) | Provider | Status | Last Updated |\n");
    md.push_str("|---|---|---|---|---|\n");

    if agents.is_empty() {
        md.push_str("| *(None)* | *(No accounts logged in yet)* | - | STANDBY | - |\n");
    } else {
        for (name, auth) in &agents {
            let email = auth.user.as_ref().and_then(|u| u.email.clone()).unwrap_or_else(|| "N/A".into());
            let provider = auth.user.as_ref().map(|u| u.provider.as_str()).unwrap_or("google");
            let updated = auth.updated_at.to_rfc3339();
            md.push_str(&format!("| **{}** | `{}` | {} | `READY / STANDBY` | {} |\n", name, email, provider, updated));
        }
    }

    md.push_str("\n---\n*Manager Agent dynamically queries this list to dispatch development tasks exclusively to authenticated agents.*\n");

    let agent_md_path = root.as_ref().join("agent.md");
    fs::write(agent_md_path, md)?;
    Ok(agents.len())
}

pub fn sync_agent_md<P: AsRef<Path>>(root: P) -> Result<usize, std::io::Error> {
    update_agent_md(root)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_config_roundtrip() {
        let cfg = ProjectConfig::default_project("demo");
        let toml_str = toml::to_string(&cfg).unwrap();
        let loaded: ProjectConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(loaded.name, "demo");
        assert_eq!(loaded.agents.len(), 5);
        assert_eq!(loaded.pool.min_workers, 1);
    }
}
