# Multi-Agent Development Orchestrator (`mag`)

## Rust実装マスター計画書

> **AIを増やすのではなく、AIを組織化する。**

---

# 1. この文書の目的

本書は `mag` の実装担当AIおよび開発者が、設計を独自解釈せずに実装できるようにするための**マスター実装計画書**である。

本プロジェクトは**Rustを主要実装言語として開発する**。

Manager、Worker、Scheduler、CLI、Task Manager、Git Manager、Container Manager、API、ログ管理、データ管理など、本システムのコアロジックは原則としてRustで実装する。

Python等を補助スクリプトとして使用することは可能だが、`mag`本体の主要機能をPythonで実装してはならない。

---

# 2. 最終目標

最終的に以下の操作だけで、複数のAI Agentがソフトウェア開発を実行できる状態を目指す。

```bash
mag init my-project
mag start
mag "FastAPIを使用したJWT認証付きユーザー管理APIを構築してください"
```

その後、システムが自動的に、

```text
ユーザー
  ↓
Manager
  ↓
要件分析
  ↓
タスク分解
  ↓
Agent割り当て
  ↓
Git Worktree作成
  ↓
Developer
  ↓
Tester
  ↓
Reviewer
  ↓
Security
  ↓
問題発見
  ↓
Developerへ修正指示
  ↓
再テスト
  ↓
Managerによる最終評価
  ↓
Merge
  ↓
完成
```

までを実行する。

---

# 3. 実装言語

## 3.1 必須

```text
Rust
```

`mag`のコアシステムはRustで実装する。

---

# 4. 推奨Rustバージョン

プロジェクト開始時点で安定版Rustを使用する。

ただし、特定のRustバージョンに依存する機能を安易に採用せず、可能な限りstable Rustで動作する実装にする。

`rust-toolchain.toml`を用意し、プロジェクトが使用するRustバージョンを固定する。

---

# 5. 開発環境

Managerは以下の環境を想定する。

```text
Ubuntu 26.04
    │
    └── mag
        └── Manager
```

Worker実行基盤は、

```text
Ubuntu 24.04
    │
    ├── Container 1 / Agent A
    ├── Container 2 / Agent B
    ├── Container 3 / Agent C
    ├── Container 4 / Agent D
    └── Container 5 / Agent E
```

とする。

---

# 6. システムアーキテクチャ

```text
                         User
                          │
                          ▼
                    ┌───────────┐
                    │ mag CLI   │
                    └─────┬─────┘
                          │
                          ▼
                    ┌───────────┐
                    │ Manager   │
                    └─────┬─────┘
                          │
             ┌────────────┼────────────┐
             │            │            │
             ▼            ▼            ▼
         Scheduler    Task Manager   Git Manager
             │
             ▼
       Worker Manager
             │
       ┌─────┼─────┬─────┬─────┐
       ▼     ▼     ▼     ▼     ▼
      A      B     C     D     E
   Developer Tester Review Security Research
       │     │     │     │     │
       └─────┴─────┴─────┴─────┘
                    │
                    ▼
                Manager
                    │
              Evaluation
                    │
              ┌─────┴─────┐
              ▼           ▼
            Retry        Merge
```

---

# 7. Rust Workspace構成

Cargo Workspaceを使用する。

推奨構成：

```text
mag/
├── Cargo.toml
├── Cargo.lock
├── rust-toolchain.toml
│
├── crates/
│   ├── mag-cli/
│   ├── mag-manager/
│   ├── mag-worker/
│   ├── mag-scheduler/
│   ├── mag-task/
│   ├── mag-agent/
│   ├── mag-container/
│   ├── mag-git/
│   ├── mag-api/
│   ├── mag-storage/
│   ├── mag-config/
│   ├── mag-logging/
│   └── mag-common/
│
├── agents/
│   ├── developer.toml
│   ├── tester.toml
│   ├── reviewer.toml
│   ├── security.toml
│   └── researcher.toml
│
├── templates/
│   ├── docker/
│   └── project/
│
├── tests/
│
├── docs/
│
└── README.md
```

---

# 8. Crateの責務

## `mag-cli`

CLIインターフェースを担当する。

実装対象：

```text
mag init
mag start
mag stop
mag status
mag agents
mag task
mag logs
mag test
mag review
mag merge
mag doctor
```

---

## `mag-manager`

Manager Agent本体。

担当：

* ユーザー要求の受け取り
* タスク分解
* Workerへの指示
* 結果収集
* 成否判定
* Retry判断
* 最終統合判断

---

## `mag-worker`

Worker側の実行エンジン。

担当：

* タスク受信
* タスク実行
* コマンド実行
* Git操作
* 結果返却

---

## `mag-scheduler`

タスクスケジューラ。

担当：

* タスクキュー
* 優先順位
* 依存関係
* 並列実行
* Retry

---

## `mag-task`

Taskモデル。

担当：

* Task ID
* Task状態
* Task依存関係
* Task結果

---

## `mag-agent`

Agent定義。

担当：

* Agent ID
* Agent Role
* Agent能力
* Agent権限
* Agent状態

---

## `mag-container`

Docker/Container管理。

担当：

* Container作成
* 起動
* 停止
* 再起動
* 削除
* Health Check
* Resource Limit

---

## `mag-git`

Git操作。

担当：

* Repository確認
* Branch作成
* Worktree作成
* Commit
* Diff
* Merge
* Conflict検出

---

## `mag-api`

ManagerとWorker間の通信。

---

## `mag-storage`

永続データ保存。

MVPではSQLiteを使用する。

---

## `mag-config`

設定ファイルの読み込み・検証。

---

## `mag-logging`

構造化ログを担当する。

---

## `mag-common`

複数crateで共有する型。

例：

```text
Task
TaskStatus
Agent
AgentStatus
TaskResult
ReviewResult
TestResult
SecurityResult
```

---

# 9. CLI仕様

## 9.1 init

```bash
mag init my-project
```

処理：

1. Project Directory作成
2. `.mag/`作成
3. 設定ファイル作成
4. Git Repository確認
5. 初期Agent設定生成
6. SQLite Database作成

---

# 10. `.mag`ディレクトリ

プロジェクト内部に、

```text
.mag/
├── config.toml
├── database.sqlite
├── agents/
├── tasks/
├── worktrees/
├── logs/
├── results/
└── runtime/
```

を作成する。

---

# 11. start

```bash
mag start
```

以下を実行する。

```text
1. 設定読み込み
2. Docker接続確認
3. Worker確認
4. Container起動
5. Health Check
6. Manager起動
7. Scheduler起動
```

---

# 12. status

```bash
mag status
```

出力例：

```text
MAG STATUS

Manager     RUNNING

Agents
──────────────────────────────
A Developer    RUNNING
B Tester       IDLE
C Reviewer     IDLE
D Security     IDLE
E Researcher   IDLE

Tasks
──────────────────────────────
TASK-001       RUNNING
TASK-002       PENDING
TASK-003       PENDING
```

---

# 13. stop

```bash
mag stop
```

以下を安全な順番で停止する。

```text
Scheduler
 ↓
Manager
 ↓
Workers
 ↓
Containers
```

実行中Taskが存在する場合は確認する。

---

# 14. 自然言語タスク

以下を可能にする。

```bash
mag "RustでCLIツールを作ってください"
```

CLIは入力をManagerに渡す。

ManagerはLLMへ、

```text
User Requirement
```

として渡す。

LLMの結果をTask Planに変換する。

---

# 15. Task Plan

例えば、

```text
TASK-001
要件分析

TASK-002
プロジェクト設計

TASK-003
CLI実装

TASK-004
テスト作成

TASK-005
コードレビュー

TASK-006
セキュリティ確認

TASK-007
README作成
```

を生成する。

---

# 16. Task Dependency

Taskには依存関係を持たせる。

例：

```text
TASK-001
   ↓
TASK-002
   ↓
TASK-003
   ├──→ TASK-004
   ├──→ TASK-005
   ├──→ TASK-006
   └──→ TASK-007
```

独立Taskは並列実行する。

---

# 17. Task State

以下の状態を実装する。

```text
PENDING
ASSIGNED
RUNNING
WAITING
REVIEW
TESTING
FAILED
RETRYING
COMPLETED
CANCELLED
```

状態遷移をManagerで管理する。

---

# 18. Agent定義

Agentは設定ファイルから読み込む。

例：

```toml
id = "agent-a"
name = "Developer"
role = "developer"
container = "container-1"
```

---

# 19. Agent能力

AgentにはCapabilitiesを設定する。

例：

```toml
capabilities = [
    "code.write",
    "code.modify",
    "git.commit",
    "test.run"
]
```

ManagerはTaskに必要なCapabilityを確認し、適切なAgentへ割り当てる。

---

# 20. Agent権限

Agentには権限を与える。

例：

```text
Developer
├── source read
├── source write
├── test execute
└── git commit

Reviewer
├── source read
└── review write
```

権限外の操作は拒否する。

---

# 21. Container管理

`mag-container`はDocker CLIまたはDocker Engine APIを利用する。

最初の実装ではDocker CLIを呼び出す方式でもよい。

ただし、コマンド文字列を直接連結せず、RustのCommand APIを使用する。

---

# 22. Container命名

以下の命名規則を使用する。

```text
mag-{project}-agent-a
mag-{project}-agent-b
mag-{project}-agent-c
mag-{project}-agent-d
mag-{project}-agent-e
```

---

# 23. Container基本設定

各Containerには、

```text
/workspace
/workspace/project
/workspace/result
/workspace/logs
```

を用意する。

---

# 24. Container Resource Limit

Agentごとに、

```text
CPU
Memory
PIDs
Disk
Network
```

を制限できるようにする。

設定例：

```toml
cpu = 2
memory_mb = 4096
pids_limit = 256
```

---

# 25. Container再起動

Workerが異常終了した場合、

```text
Health Check
 ↓
Failure
 ↓
Manager
 ↓
Container Restart
```

を実行する。

Taskは可能な限り状態を復元する。

---

# 26. Git Worktree

AgentごとにWorktreeを作る。

例：

```text
.mag/worktrees/
├── agent-a/
├── agent-b/
├── agent-c/
├── agent-d/
└── agent-e/
```

Branch：

```text
mag/agent-a/TASK-001
mag/agent-b/TASK-002
mag/agent-c/TASK-003
```

---

# 27. Git操作

ManagerまたはWorkerが以下を実行できるようにする。

```text
status
diff
branch
worktree
add
commit
merge
```

危険なGit操作はManagerのみ許可する。

---

# 28. Merge

Merge条件を設定する。

最低条件：

```text
Build PASS
Test PASS
Review APPROVED
Security PASS
```

いずれかがFAILの場合はMergeしない。

---

# 29. Conflict処理

Merge Conflictが発生した場合、

```text
Merge
 ↓
Conflict
 ↓
Manager
 ↓
Developer Agent
 ↓
Conflict resolution
 ↓
Test
 ↓
Review
```

とする。

自動解決に失敗した場合はユーザー確認に切り替える。

---

# 30. Worker API

最低限以下を実装する。

```http
GET  /health
GET  /status
POST /task
GET  /task/{id}
POST /task/{id}/cancel
```

---

# 31. API通信形式

JSONを使用する。

Task Request：

```json
{
  "task_id": "TASK-001",
  "type": "implementation",
  "description": "CLI parserを実装する",
  "workspace": "/workspace/project"
}
```

---

# 32. Task Result

```json
{
  "task_id": "TASK-001",
  "status": "completed",
  "summary": "CLI parser implemented",
  "files_changed": [
    "src/cli.rs"
  ],
  "commit": "a82f91c",
  "tests_passed": true
}
```

---

# 33. LLM Adapter

LLM依存部分をManager本体から分離する。

```text
mag-manager
    │
    ▼
LLM Adapter
    │
    ├── Provider A
    ├── Provider B
    └── Provider C
```

特定のLLMサービスにManager全体が依存しない設計にする。

---

# 34. LLMインターフェース

Rust Traitとして抽象化する。

概念：

```text
trait LlmProvider
```

必要な機能：

```text
generate()
stream()
```

将来的に複数Providerを追加できる設計にする。

---

# 35. Antigravity連携

Antigravity CLIをWorkerとして利用する場合は、Managerから直接内部実装に依存しない。

以下のようなAdapter層を作る。

```text
Agent Runtime
│
├── AntigravityAdapter
├── LocalCommandAdapter
└── FutureAdapter
```

これによりAntigravity以外のAgent Runtimeにも対応可能にする。

---

# 36. 重要な設計方針

`mag`はAntigravity専用システムにしない。

最初はAntigravityを主要Runtimeとして利用するが、将来的には、

```text
Antigravity
Claude
Codex
Gemini
Local LLM
Custom Agent
```

などをWorker Runtimeとして利用できる設計にする。

---

# 37. Self-Repair

BuildまたはTestが失敗した場合、

```text
Failure
 ↓
Error Collection
 ↓
Manager Analysis
 ↓
Developer Task
 ↓
Fix
 ↓
Build
 ↓
Test
```

を実行する。

---

# 38. Retry制限

デフォルト：

```text
MAX_RETRY = 3
```

設定ファイルで変更可能にする。

3回失敗した場合、

```text
FAILED_PERMANENTLY
```

として処理を停止する。

---

# 39. エラー情報収集

Failure時には、

```text
exit code
stdout
stderr
command
working directory
changed files
git diff
environment information
```

を保存する。

---

# 40. envdoctor連携

コード原因が特定できない場合、

```text
Build Failure
 ↓
Manager
 ↓
Environment Diagnosis
 ↓
envdoctor
 ↓
Diagnosis Result
 ↓
Manager
```

とする。

外部ツールとの連携はAdapter方式にする。

---

# 41. jpcargo連携

Rustプロジェクトでは、

```text
cargo build
 ↓
Rust Error
 ↓
jpcargo
 ↓
Japanese Diagnosis
 ↓
Manager
```

という流れを実装可能にする。

`mag`本体にjpcargoのロジックをコピーせず、外部Command/Adapterとして扱う。

---

# 42. Tester Agent

Testerは以下を実行する。

```text
Build
Unit Test
Integration Test
Regression Test
Lint
```

Rustの場合：

```bash
cargo check
cargo build
cargo test
cargo clippy
```

を基本セットとする。

---

# 43. Reviewer Agent

Reviewerはコードを変更しない。

基本的に、

```text
READ
ANALYZE
REPORT
```

のみ許可する。

レビュー結果：

```json
{
  "approved": false,
  "severity": "medium",
  "issues": []
}
```

---

# 44. Security Agent

Security Agentは、

```text
Dependency
CVE
Secret
Unsafe code
Command injection
Path traversal
Authentication
Authorization
```

などを確認する。

ツールはAdapter化する。

---

# 45. Researcher Agent

Researcherは、

```text
仕様調査
ライブラリ調査
公式ドキュメント調査
既存コード調査
README
CHANGELOG
設計資料
```

を担当する。

---

# 46. Managerの意思決定

Managerは単純にLLMの回答を信用してはならない。

例えば、

```text
Agent A: 完成しました
```

だけでは完了としない。

必ず、

```text
Git diff
Build
Test
Review
Security
```

を確認する。

---

# 47. 完了条件

TaskをCompletedにするには、TaskごとにAcceptance Criteriaを設定する。

例：

```text
TASK-001

Acceptance Criteria:
- cargo buildが成功する
- cargo testが成功する
- ReviewerがApprovedする
```

---

# 48. Human Approval

以下の操作には人間確認を要求する。

```text
危険なshell command
Git force push
大量ファイル削除
外部ネットワークへの機密情報送信
秘密情報へのアクセス
本番環境操作
```

---

# 49. コマンド実行制御

Workerが任意のShellを実行できる設計にしない。

Command Executorを作る。

```text
CommandRequest
    ↓
Policy Check
    ↓
Allowed?
 ┌──┴──┐
YES    NO
 │      │
 ↓      ↓
Execute Reject
```

---

# 50. Allowlist

例えば、

```toml
allowed_commands = [
    "cargo",
    "rustc",
    "git",
    "python",
    "pytest"
]
```

のように設定できるようにする。

---

# 51. ログ

JSON構造化ログを基本とする。

```json
{
  "timestamp": "...",
  "level": "INFO",
  "component": "manager",
  "task_id": "TASK-001",
  "agent": "agent-a",
  "message": "task started"
}
```

---

# 52. Database

MVPではSQLiteを使用する。

テーブル：

```text
projects
agents
tasks
task_dependencies
jobs
events
results
reviews
test_results
security_results
```

---

# 53. Event Log

重要操作をEventとして保存する。

例：

```text
TASK_CREATED
TASK_ASSIGNED
AGENT_STARTED
AGENT_COMPLETED
BUILD_FAILED
TEST_FAILED
REVIEW_FAILED
SECURITY_FAILED
RETRY_STARTED
MERGE_STARTED
MERGE_COMPLETED
```

---

# 54. 非同期処理

Managerは複数Agentを同時に動かせる必要がある。

Rustのasync/awaitを使用する。

候補：

```text
tokio
```

を利用する。

---

# 55. HTTP

API実装には、

```text
axum
```

を第一候補とする。

---

# 56. CLI

CLIには、

```text
clap
```

を使用する。

---

# 57. Serialization

設定・API・Task情報には、

```text
serde
serde_json
toml
```

を使用する。

---

# 58. Git

Git操作は最初は、

```text
git command
```

をRustから安全に呼び出す方式でもよい。

将来的にGitライブラリへ移行する。

---

# 59. Container

最初はDocker CLIをRustから呼び出す。

将来的にDocker Engine APIなどへ移行できる構造にする。

---

# 60. Error Handling

Rustのエラー処理は、

```text
Result<T, E>
```

を基本とする。

アプリケーション全体では、

```text
thiserror
anyhow
```

を適切に使い分ける。

Library crateでは可能な限り構造化されたエラーを返す。

---

# 61. テスト方針

テストを最初から実装する。

必要なテスト：

```text
Unit Test
Integration Test
API Test
Container Test
Git Test
Scheduler Test
Task State Test
Security Test
```

---

# 62. Manager Unit Test

最低限、

```text
Task creation
Task assignment
Task dependency
Retry
Failure
Completion
```

をテストする。

---

# 63. Scheduler Test

例えば、

```text
A → B
A → C
```

の場合、

```text
A completed
 ↓
B and C can run concurrently
```

になることを確認する。

---

# 64. Integration Test

以下を自動テストする。

```text
Manager
 ↓
API
 ↓
Worker
 ↓
Container
 ↓
Command
 ↓
Result
 ↓
Manager
```

---

# 65. Security Test

以下をテストする。

```text
禁止Command
権限外アクセス
Container escape対策
Path traversal
不正Task
巨大Task
Timeout
```

---

# 66. MVP Phase 1

## 目標

```text
Rust CLI
+
Docker
+
3 Worker
```

を動作させる。

Agent：

```text
Developer
Tester
Reviewer
```

SecurityとResearcherは後から追加する。

---

# 67. MVP Phase 1 完了条件

以下が成功すること。

```bash
mag init test-project
mag start
mag status
mag "Hello World CLIを作ってください"
```

そして、

```text
Developer
 ↓
コード生成
 ↓
Tester
 ↓
テスト
 ↓
Reviewer
 ↓
レビュー
 ↓
Manager
 ↓
完成
```

まで動作すること。

---

# 68. MVP Phase 2

追加：

```text
Security Agent
Researcher Agent
```

5 Agent構成にする。

---

# 69. MVP Phase 3

Self-Repairを追加する。

```text
Build
 ↓
Failure
 ↓
Analysis
 ↓
Fix
 ↓
Build
 ↓
Test
```

---

# 70. MVP Phase 4

Git Worktreeを完全統合する。

---

# 71. MVP Phase 5

Environment Diagnosisを追加する。

```text
envdoctor
jpcargo
```

との連携を実装する。

---

# 72. Version 1.0

以下を実現する。

```text
5 Agent
Container Isolation
Task Scheduler
Git Worktree
Build
Test
Review
Security
Research
Self Repair
Logs
Database
Human Approval
```

---

# 73. Version 1.1

追加予定：

```text
Web UI
Live Agent Monitor
Task Graph
Log Viewer
Diff Viewer
Agent Metrics
```

---

# 74. Version 1.2

追加予定：

```text
複数LLM Provider
複数Agent Runtime
Remote Worker
複数Ubuntuホスト
```

---

# 75. 将来構想

最終的には、

```text
Ubuntu 26.04
       │
       ▼
Manager
       │
       ├── Local Worker
       ├── Docker Worker
       ├── WSL Worker
       ├── Remote Linux Worker
       └── Cloud Worker
```

という分散Worker構成を実現する。

---

# 76. マルチホスト構成

将来的には、

```text
Host A
Ubuntu 26.04
Manager
   │
   ├─────────────┐
   │             │
   ▼             ▼
Host B         Host C
Ubuntu 24.04   Ubuntu 22.04
Workers        Workers
```

にも対応する。

---

# 77. Remote Worker

Workerには、

```text
Worker ID
Host ID
Agent ID
Capabilities
Status
```

を持たせる。

ManagerはCapabilityを見てWorkerを選択する。

---

# 78. Capability Routing

例えば、

```text
Task:
ROCm環境でPyTorchをテスト
```

の場合、

```text
GPU capability
```

を持つWorkerへ送る。

---

# 79. AI Agent Marketplaceへの発展

将来的にはAgent Roleを追加可能にする。

```text
agents/
├── developer
├── tester
├── security
├── reviewer
├── researcher
├── devops
├── database
├── frontend
└── performance
```

---

# 80. Plugin System

AgentをPluginとして追加できる設計を検討する。

例：

```text
mag plugin install security-agent
mag plugin install rust-agent
mag plugin install rocm-agent
```

---

# 81. 重要な差別化

`mag`は、

```text
複数AIを起動する
```

こと自体を目的としない。

以下を目的とする。

```text
Task Decomposition
+
Role Assignment
+
Isolation
+
Execution
+
Verification
+
Review
+
Security
+
Self Repair
+
Integration
```

---

# 82. 開発順序

実装順序を変更しない。

```text
01 Rust Workspace
02 CLI
03 Config
04 Common Types
05 Task Model
06 Worker
07 Worker API
08 Container Manager
09 Manager
10 Scheduler
11 Git Manager
12 Git Worktree
13 Tester
14 Reviewer
15 Security
16 Researcher
17 LLM Adapter
18 Task Decomposition
19 Self Repair
20 Human Approval
21 envdoctor
22 jpcargo
23 Full Integration
```

---

# 83. 実装上の禁止事項

以下を禁止する。

### 1

Managerにすべての処理を詰め込まない。

### 2

Agent間で直接ファイルを書き換えない。

### 3

Git管理を無視して共有Workspaceを直接変更しない。

### 4

LLM出力を無条件に信用しない。

### 5

Workerに無制限のroot権限を与えない。

### 6

無限Retryを実装しない。

### 7

ログを保存しない設計にしない。

### 8

特定のAIサービスにManager全体を依存させない。

---

# 84. 実装時のAIへの指示

実装担当AIは、以下を厳守する。

```text
1. Rustで実装する。
2. Cargo Workspaceを使用する。
3. 各機能をcrate単位で分離する。
4. TODOを残したまま次のPhaseへ進まない。
5. コンパイルできないコードをコミットしない。
6. Unit Testを追加する。
7. Integration Testを必要に応じて追加する。
8. エラー処理を適切に実装する。
9. unsafeを原則使用しない。
10. public APIにはドキュメントを付ける。
11. clippyを実行する。
12. rustfmtを実行する。
13. Git変更内容を確認してからcommitする。
14. 既存機能を壊さない。
15. 仕様変更が必要な場合は設計文書を更新する。
```

---

# 85. 各Phaseの終了条件

各Phaseには必ず、

```text
Implementation
 ↓
Unit Test
 ↓
Integration Test
 ↓
Documentation
 ↓
Review
 ↓
Commit
```

の工程を設ける。

「コードを書いた」だけではPhase完了としない。

---

# 86. Definition of Done

Taskは以下を満たした場合のみ完了とする。

```text
[ ] 要件を満たしている
[ ] Rustコードがコンパイルできる
[ ] rustfmt済み
[ ] clippyで重大な警告がない
[ ] Unit Test成功
[ ] 必要なIntegration Test成功
[ ] Git diff確認済み
[ ] ドキュメント更新済み
[ ] Reviewer承認済み
```

---

# 87. 最初に実装するもの

実装開始時は以下だけを作る。

```text
mag
│
├── CLI
│
├── Config
│
├── Task
│
├── Worker
│
└── Manager
```

最初から、

```text
Security
CVE
Self Repair
LLM
Web UI
```

まで実装しない。

---

# 88. 最初の動作目標

まず以下を成功させる。

```bash
mag init demo
```

↓

```bash
mag start
```

↓

```bash
mag status
```

↓

```bash
mag task create "create hello world"
```

↓

```text
Manager
 ↓
Worker
 ↓
Task
 ↓
Result
```

この最小ループを完成させる。

---

# 89. 最重要目標

最初に完成させるべきものは、

```text
Manager
   ↓
Task
   ↓
Worker
   ↓
Result
```

である。

これが安定してから、

```text
Container
Git
Test
Review
Security
LLM
Self Repair
```

を順番に追加する。

---

# 90. 最終評価基準

`mag`が本当に完成したかを判断する基準は、

> **ユーザーが1つの自然言語指示を出すだけで、複数の隔離されたAI Agentが役割分担し、実装・テスト・レビュー・セキュリティ確認・修正・統合まで実行できるか**

とする。

単に5つのAntigravityを起動できるだけでは完成とはしない。

---

# 91. 最終システム

```text
                              USER
                                │
                                ▼
                    ┌────────────────────┐
                    │     mag CLI        │
                    └─────────┬──────────┘
                              │
                              ▼
                    ┌────────────────────┐
                    │ Manager / Rust     │
                    │                    │
                    │ Planner             │
                    │ Scheduler           │
                    │ Evaluator           │
                    │ Git Manager         │
                    └─────────┬──────────┘
                              │
              ┌───────────────┼───────────────┐
              │               │               │
              ▼               ▼               ▼
       ┌────────────┐  ┌────────────┐  ┌────────────┐
       │ Developer  │  │ Tester     │  │ Reviewer   │
       │ Container  │  │ Container  │  │ Container  │
       │ Agent A    │  │ Agent B    │  │ Agent C    │
       └────────────┘  └────────────┘  └────────────┘
              │               │               │
              └───────────────┼───────────────┘
                              │
              ┌───────────────┴───────────────┐
              │                               │
              ▼                               ▼
       ┌────────────┐                  ┌────────────┐
       │ Security   │                  │ Researcher │
       │ Agent D    │                  │ Agent E    │
       └────────────┘                  └────────────┘
              │                               │
              └───────────────┬───────────────┘
                              ▼
                         Evaluation
                              │
                     ┌────────┴────────┐
                     │                 │
                   FAIL              PASS
                     │                 │
                     ▼                 ▼
                 Self Repair          Merge
                     │                 │
                     └───────┐         │
                             ▼         ▼
                            Test    Project
                             │      Complete
                             └─────────┘
```

---

# 92. 開発者への最終指示

このプロジェクトを実装する際は、**Rustを第一言語として扱うこと**。

実装は必ずPhase順に進める。

特に、

```text
Phase 1
Rust Workspace

↓

Phase 2
CLI

↓

Phase 3
Task

↓

Phase 4
Worker

↓

Phase 5
Manager

↓

Phase 6
Container

↓

Phase 7
Git

↓

Phase 8
Test

↓

Phase 9
Review

↓

Phase 10
Security

↓

Phase 11
LLM

↓

Phase 12
Self Repair

↓

Phase 13
Full Integration
```

の順序を守る。

各Phaseでは、

```text
実装
↓
テスト
↓
修正
↓
ドキュメント
↓
レビュー
↓
コミット
```

まで完了してから次のPhaseへ進む。

最終的には、`mag`を**単なるAntigravity複数起動ツールではなく、複数AIエージェントをソフトウェア開発チームとして組織化するRust製オーケストレーションプラットフォーム**として完成させる。
