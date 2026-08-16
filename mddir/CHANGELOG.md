# Changelog (CHANGELOG.md)

All notable changes to the **Multi-Agent Development Orchestrator (`mag` / `agycli`)** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.2] - 2026-08-16

### Added
- **Native AGY Interactive Terminal REPL & Slash Commands (`mddir/addplan5.md`)**:
  - Running `agycli` without arguments now launches an interactive terminal session (`agycli ❯ `) matching the original `agy` experience.
  - Implemented AGY Slash Commands:
    - `/help`: Display available slash commands and descriptions.
    - `/status`: Show orchestrator, agent, container auth, and task statuses.
    - `/doctor`: Run `EnvDoctor` diagnostics.
    - `/login [target]`: Authenticate globally or per-container.
    - `/whoami [cnt]`: Display authenticated user and container identity.
    - `/workers [N]`: Scale worker agent pool dynamically.
    - `/tasks`: List recent tasks and execution results.
    - `/clear`: Clear the screen and refresh header.
    - `/exit` / `/quit`: Terminate interactive session.
  - Real-time streaming autonomous multi-agent task execution from within the REPL.

---

## [0.2.1] - 2026-08-16

### Added
- **Per-Container Authentication & Persistence (`mddir/addplan4.md`)**:
  - `agycli login <container_name>` (e.g. `agycli login agent-a`): triggers browser login and saves persistent credentials to `.mag/containers/<name>/credentials.json`.
  - Credentials persist across container restarts, updates, and reinstalls without authentication loss.
  - Initial container startup is completely clean.
- **Dynamic Multi-Role Collaborative Task Queue**:
  - Work-stealing collaborative task scheduler (`TaskScheduler::assign_collaborative_workers` & `claim_next_task_for_worker`).
  - Allows 1..N workers (e.g. 2 workers) to cooperatively execute all 5 roles without idle blocking.
  - Conflict-free Git Worktrees per task/container.
- **Auto Git Merge on Workflow Completion**:
  - Automatically merges worktree branches into `main` branch upon successful multi-agent verification.

---

## [0.2.0] - 2026-08-16

### Added
- **Container `agycli` Integration (`mddir/addplan3.md`)**:
  - Automatically install `agycli` and `mag` binaries into `/usr/local/bin/` within all Docker container images (`containers/*/Dockerfile`).
  - Standardized authentication sharing via `.mag/credentials.json` volume mounting, enabling automatic login detection inside containers.
  - Enhanced `find_project_root` to support container environments (`/workspace` and `$HOME/.mag`).
- **Google Account Authentication (`mddir/addplan2.md`)**:
  - Implemented Google OAuth2 Device Authorization Flow (`GoogleAuthClient`).
  - Added secure local credential store in `.mag/credentials.json`.
  - Added user identity display in `mag status` and `mag whoami`.
  - Added `mag login [google|token]` and `mag logout` commands.
- **Dynamic Container & Worker Pool Scaling (`mddir/addplan2.md`)**:
  - Implemented `WorkerPoolManager` in `mag-container` to scale active agent containers dynamically up to N.
  - Added `mag scale --workers <N>` command and `--workers` CLI option.
- **Target Path & Real Source Code Generation (`mddir/addplan2.md`)**:
  - Added automatic target directory path extraction from natural language prompts (e.g. `/home/guru/agytest に...`).
  - Generated full Rust project structure (`Cargo.toml`, `src/main.rs`, `docs/spec.md`) with automated test execution and verification.
- **Rust-Native 13-Crate Cargo Workspace Architecture (`mddir/addplan.md`)**:
  - Implemented 13-crate workspace: `mag-common`, `mag-config`, `mag-task`, `mag-agent`, `mag-logging`, `mag-storage`, `mag-git`, `mag-container`, `mag-api`, `mag-worker`, `mag-scheduler`, `mag-manager`, and `mag-cli`.
  - Provided dual CLI commands `mag` and `agycli`.
  - 100% automated test suite passing via `cargo test --workspace`.

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
