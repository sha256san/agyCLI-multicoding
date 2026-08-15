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
            agents,
        }
    }
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
    }
}
