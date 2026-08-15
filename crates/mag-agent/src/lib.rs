//! Agent capabilities, definitions, and security policies for `mag`.

use mag_common::AgentRole;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Definition of an Agent with its capabilities and allowed actions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDefinition {
    pub id: String,
    pub name: String,
    pub role: AgentRole,
    pub description: String,
    pub capabilities: HashSet<String>,
    pub allowed_commands: HashSet<String>,
}

impl AgentDefinition {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        role: AgentRole,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            role,
            description: description.into(),
            capabilities: HashSet::new(),
            allowed_commands: HashSet::new(),
        }
    }

    pub fn has_capability(&self, cap: &str) -> bool {
        self.capabilities.contains(cap)
    }

    pub fn is_command_allowed(&self, cmd: &str) -> bool {
        let base_cmd = cmd.split_whitespace().next().unwrap_or("");
        let bin_name = std::path::Path::new(base_cmd)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(base_cmd);

        self.allowed_commands.contains(bin_name)
    }

    pub fn default_for_role(role: AgentRole) -> Self {
        match role {
            AgentRole::Developer => {
                let mut a = Self::new("agent-a", "Developer", role, "Implementation and bug fixing");
                a.capabilities.extend(["code.write".into(), "code.modify".into(), "git.commit".into(), "test.run".into()]);
                a.allowed_commands.extend(["cargo".into(), "rustc".into(), "git".into(), "python3".into(), "pytest".into(), "cat".into(), "ls".into(), "mkdir".into(), "cp".into(), "mv".into()]);
                a
            }
            AgentRole::Tester => {
                let mut a = Self::new("agent-b", "Tester", role, "Unit/integration testing and build verification");
                a.capabilities.extend(["test.run".into(), "build.run".into(), "lint.run".into()]);
                a.allowed_commands.extend(["cargo".into(), "git".into(), "python3".into(), "pytest".into(), "cat".into(), "ls".into(), "grep".into()]);
                a
            }
            AgentRole::Reviewer => {
                let mut a = Self::new("agent-c", "Reviewer", role, "Static code review, readability, and design check");
                a.capabilities.extend(["code.read".into(), "code.review".into()]);
                a.allowed_commands.extend(["git".into(), "cat".into(), "ls".into(), "grep".into(), "find".into(), "clippy".into()]);
                a
            }
            AgentRole::Security => {
                let mut a = Self::new("agent-d", "Security", role, "Vulnerability audit and secret leak scanning");
                a.capabilities.extend(["security.scan".into(), "secret.check".into()]);
                a.allowed_commands.extend(["git".into(), "cargo-audit".into(), "cat".into(), "ls".into(), "grep".into(), "find".into()]);
                a
            }
            AgentRole::Researcher => {
                let mut a = Self::new("agent-e", "Researcher", role, "Technical investigation and documentation");
                a.capabilities.extend(["research.doc".into(), "doc.write".into()]);
                a.allowed_commands.extend(["git".into(), "cat".into(), "ls".into(), "grep".into(), "find".into(), "python3".into()]);
                a
            }
            AgentRole::Manager => {
                let mut a = Self::new("manager", "Manager", role, "Overall management and orchestration");
                a.capabilities.extend(["task.dispatch".into(), "task.evaluate".into(), "git.merge".into()]);
                a
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_allowlist_check() {
        let dev = AgentDefinition::default_for_role(AgentRole::Developer);
        assert!(dev.is_command_allowed("cargo build"));
        assert!(dev.is_command_allowed("/usr/bin/git status"));
        assert!(!dev.is_command_allowed("rm -rf /"));
    }
}
