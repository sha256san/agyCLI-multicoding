# Changelog (CHANGELOG.md)

All notable changes to the **Multi-Agent Development Orchestrator (`mag`)** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0-rust.1] - 2026-08-15

### Added
- **Rust-Native Multi-Crate Architecture (`mddir/addplan.md`)**:
  - Implemented 13-crate Cargo Workspace with pure Rust toolchain:
    - [`crates/mag-common`](file:///home/guru/agyCLI++/crates/mag-common): Core shared enums, types, and constants.
    - [`crates/mag-config`](file:///home/guru/agyCLI++/crates/mag-config): TOML configuration loader and validator.
    - [`crates/mag-task`](file:///home/guru/agyCLI++/crates/mag-task): Task data models, state transitions, and dependency validation.
    - [`crates/mag-agent`](file:///home/guru/agyCLI++/crates/mag-agent): Agent capability definitions and command allowlist verification.
    - [`crates/mag-logging`](file:///home/guru/agyCLI++/crates/mag-logging): Structured JSON logging.
    - [`crates/mag-storage`](file:///home/guru/agyCLI++/crates/mag-storage): SQLite database persistence with `rusqlite`.
    - [`crates/mag-git`](file:///home/guru/agyCLI++/crates/mag-git): Git repo initialization, branch, worktree, and merge manager.
    - [`crates/mag-container`](file:///home/guru/agyCLI++/crates/mag-container): Docker container lifecycle and resource management.
    - [`crates/mag-api`](file:///home/guru/agyCLI++/crates/mag-api): HTTP REST client using `reqwest` with `rustls-tls`.
    - [`crates/mag-worker`](file:///home/guru/agyCLI++/crates/mag-worker): Task execution engine and dedicated handlers for Developer, Tester, Reviewer, Security, and Researcher.
    - [`crates/mag-scheduler`](file:///home/guru/agyCLI++/crates/mag-scheduler): Task DAG dependency resolution and scheduling.
    - [`crates/mag-manager`](file:///home/guru/agyCLI++/crates/mag-manager): Autonomous orchestration engine, `ResultEvaluator` self-repair loop, `EnvDoctor`, and `JpCargoAnalyzer`.
    - [`crates/mag-cli`](file:///home/guru/agyCLI++/crates/mag-cli): High-performance Rust binary executable (`mag`).
  - Added TOML agent profiles in [`agents/`](file:///home/guru/agyCLI++/agents/).
  - Added automated workspace test suite across all 13 crates (`cargo test --workspace`).

---

## [0.1.0-beta.1] - 2026-08-15

### Added
- Worker Agent REST API Service (FastAPI prototype).
- Manager Orchestration Engine in Python.
- Unified CLI Tool (`mag`) and Docker Compose environment.

---

## [0.1.0-alpha.1] - 2026-08-15

### Added
- Master Architecture & Requirements specification in [`plan.md`](file:///home/guru/agyCLI++/mddir/plan.md).
- Initial project documentation体系 (`SPEC.md`, `TODO.md`, `MEMORY.md`, `AGENTS.md`).
