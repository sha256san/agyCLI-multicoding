"""Constants and default configurations for Multi-Agent Development Orchestrator."""

# Default network ports for Worker agents
DEFAULT_WORKER_PORTS = {
    "agent-a": 8001,  # Developer
    "agent-b": 8002,  # Tester
    "agent-c": 8003,  # Reviewer
    "agent-d": 8004,  # Security
    "agent-e": 8005,  # Researcher
}

# Manager default port
DEFAULT_MANAGER_PORT = 8000

# Safety Limits
DEFAULT_MAX_RETRY = 3
DEFAULT_TASK_TIMEOUT_SECONDS = 300
DEFAULT_MAX_TASK_COUNT = 50

# Command Allowlists per role
ROLE_COMMAND_ALLOWLIST = {
    "developer": [
        "git", "python", "python3", "pytest", "cargo", "rustc",
        "npm", "node", "make", "gcc", "g++", "cat", "ls", "find",
        "mkdir", "cp", "mv", "touch"
    ],
    "tester": [
        "git", "python", "python3", "pytest", "cargo", "npm",
        "node", "make", "cat", "ls", "grep"
    ],
    "reviewer": [
        "git", "cat", "ls", "grep", "find", "clippy", "flake8", "eslint"
    ],
    "security": [
        "git", "cargo-audit", "npm", "pip-audit", "trivy", "semgrep",
        "cat", "ls", "grep", "find"
    ],
    "researcher": [
        "git", "cat", "ls", "grep", "find", "curl", "python", "python3"
    ],
}

# Dangerous command keywords requiring human confirmation
DANGEROUS_COMMAND_KEYWORDS = [
    "rm -rf", "mkfs", "dd if=", ":(){ :|:& };:", "chmod -R 777",
    "sudo", "shutdown", "reboot", "iptables -F"
]
