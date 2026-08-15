# Changelog (CHANGELOG.md)

All notable changes to the **Multi-Agent Development Orchestrator (`mag`)** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0-beta.1] - 2026-08-15

### Added
- **Worker Agent REST API Service**:
  - Implemented multi-threaded Worker HTTP REST server (`project/src/worker/server.py`) supporting `/task`, `/status`, `/result`, `/cancel`, and `/health`.
  - Implemented `CommandExecutor` (`project/src/worker/executor.py`) with command allowlist enforcement, dangerous pattern filtering, and execution timeout protection.
  - Implemented dedicated agent handlers (`project/src/worker/agent_logic.py`) for Developer (A), Tester (B), Reviewer (C), Security (D), and Researcher (E).
- **Manager Orchestration Engine**:
  - Implemented SQLite database layer (`project/src/manager/db.py`) for persistent task, agent, and logging storage.
  - Implemented HTTP `WorkerClient` (`project/src/manager/worker_client.py`) for inter-agent communication.
  - Implemented `TaskManager` (`project/src/manager/task_manager.py`) with DAG dependency resolution and state machine transitions (`PENDING` -> `ASSIGNED` -> `RUNNING` -> `REVIEW` -> `TESTING` -> `COMPLETED` / `RETRY` / `FAILED_PERMANENTLY`).
  - Implemented `ResultEvaluator` (`project/src/manager/evaluator.py`) providing self-repair feedback and `MAX_RETRY` enforcement.
  - Implemented `GitManager` (`project/src/manager/git_manager.py`) for branch, worktree, and merge handling.
  - Implemented diagnostic tools (`project/src/manager/diagnostics.py`) for `EnvDoctor` and `JpCargoAnalyzer`.
  - Implemented `Orchestrator` (`project/src/manager/orchestrator.py`) executing autonomous multi-agent development loops.
- **Unified CLI Tool (`mag`)**:
  - Created root executable [`mag`](file:///home/guru/agyCLI++/mag) and CLI module (`project/src/cli.py`) with commands: `init`, `status`, `run`, `doctor`, `task list`, `task show`, and `logs`.
- **Docker & Container Environment**:
  - Created Dockerfiles for all 5 worker roles (`containers/*/Dockerfile`) and `docker-compose.yml`.
  - Created role configuration YAMLs (`project/agents/*.yaml`) and `project/project.yaml`.
- **Automated Test Suite**:
  - Created 14 unit and integration tests across `test_common.py`, `test_worker.py`, `test_manager.py`, and `test_e2e.py` (100% PASS).

---

## [0.1.0-alpha.1] - 2026-08-15

### Added
- **Master Architecture & Requirements**: Created [`plan.md`](file:///home/guru/agyCLI++/mddir/plan.md) defining the comprehensive 66-section specification for the Multi-Agent Orchestrator.
- **Specification Document**: Created [`SPEC.md`](file:///home/guru/agyCLI++/mddir/SPEC.md).
- **Task Backlog & Roadmap**: Created [`TODO.md`](file:///home/guru/agyCLI++/mddir/TODO.md).
- **Design Decisions & Principles**: Created [`MEMORY.md`](file:///home/guru/agyCLI++/mddir/MEMORY.md).
- **Agent Behavioral Guidelines**: Created [`AGENTS.md`](file:///home/guru/agyCLI++/mddir/AGENTS.md).
- **Project Structure**: Initialized `project/` directory with user-facing [`readme.md`](file:///home/guru/agyCLI++/project/readme.md).
