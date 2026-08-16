# agyCLI-multicoding 機能拡張計画書

## Multi-Agent Autonomous Development Platform

**対象リポジトリ**

[https://github.com/sha256san/agyCLI-multicoding](https://github.com/sha256san/agyCLI-multicoding?utm_source=chatgpt.com)

**作成日:** 2026-08-16

---

# 1. 目的

`agyCLI-multicoding` を、単純な「複数AIエージェントをDocker上で動かす実験環境」から、

> **ターミナルを閉じてもAIが自律的に開発を継続し、再接続すると進捗・ログ・結果を確認できるMulti-Agent Development Platform**

へ発展させる。

現在のリポジトリには、Managerと複数の専門エージェントをDockerコンテナで分離する基本構造が存在する。

* Manager
* Developer
* Tester
* Reviewer
* Security
* Researcher

また、`project/src` 以下には `manager`、`worker`、`common`、CLIなどの構造が存在し、エージェント定義もYAMLで分離されている。

この基本構造は維持しつつ、以下の不足機能を追加する。

---

# 2. 現状

## 2.1 現在存在する構成

```text
agyCLI-multicoding/
├── containers/
│   ├── developer/
│   ├── researcher/
│   ├── reviewer/
│   ├── security/
│   └── tester/
│
├── project/
│   ├── agents/
│   │   ├── developer.yaml
│   │   ├── researcher.yaml
│   │   ├── reviewer.yaml
│   │   ├── security.yaml
│   │   └── tester.yaml
│   │
│   ├── src/
│   │   ├── common/
│   │   ├── manager/
│   │   ├── worker/
│   │   └── cli.py
│   │
│   ├── tests/
│   └── project.yaml
│
├── docker-compose.yml
└── mag
```

Docker ComposeではManagerと5種類のAgentが独立サービスとして構成されている。

Managerは8000番、Agentは8001～8005番を使用する構成になっている。

---

# 3. 現状の主な不足点

## 3.1 セッション永続化

現在の実行状態をプロセス終了後も保持する仕組みが不足している。

必要な情報：

```text
Task ID
Session ID
Agent ID
Agent Role
Status
Prompt
Current Step
Progress
Created At
Updated At
Exit Code
Result
Error
Logs
```

---

## 3.2 ターミナル切断への対応

現在のCLIを単純なフォアグラウンドプロセスとして扱うと、

```text
Terminal
   ↓
agyCLI
   ↓
Manager
   ↓
Agents
```

となり、ターミナル終了時に実行環境との関係が切れてしまう。

これを、

```text
Terminal
   │
   │ attach / detach
   ↓
agyCLI Client
   │
   ↓
Persistent Manager
   │
   ├── Developer
   ├── Tester
   ├── Reviewer
   ├── Security
   └── Researcher
```

へ変更する。

---

# 4. 最重要機能：Detached Execution

## 4.1 目標

以下の操作を可能にする。

```bash
agy run "このプロジェクトを完成させて"
```

実行後、

```text
Task started
Task ID: task_01H...
Status: RUNNING
```

と表示。

その後、

```text
Ctrl+C
```

またはターミナルを閉じても、AI側のタスクは停止しない。

---

# 5. Task ID方式

すべてのAI開発処理に一意なTask IDを付与する。

例：

```text
task_01J7X8K2...
```

ユーザーはTask IDによって後から処理へ再接続できる。

---

## 5.1 コマンド

### タスク開始

```bash
agy run "Webアプリを作成してください"
```

### バックグラウンド実行

```bash
agy run --detach "Webアプリを作成してください"
```

### タスク一覧

```bash
agy task list
```

### タスク確認

```bash
agy task status <task-id>
```

### タスクへ再接続

```bash
agy attach <task-id>
```

### ログ確認

```bash
agy logs <task-id>
```

### タスク停止

```bash
agy task stop <task-id>
```

### タスク再開

```bash
agy task resume <task-id>
```

---

# 6. Session Manager

新しい主要コンポーネントとしてSession Managerを追加する。

```text
Manager
   │
   ├── Task Manager
   ├── Session Manager
   ├── Agent Scheduler
   ├── Event Bus
   ├── Log Manager
   └── Recovery Manager
```

---

# 7. Task Manager

Task ManagerはAI開発処理を管理する。

## 状態

```text
QUEUED
  ↓
RUNNING
  ↓
WAITING
  ↓
RUNNING
  ↓
COMPLETED
```

異常時：

```text
RUNNING
   ↓
FAILED
   ↓
RECOVERING
   ↓
RUNNING
```

ユーザーが停止：

```text
RUNNING
   ↓
STOPPING
   ↓
STOPPED
```

---

# 8. 永続データベース

最初の実装ではSQLiteを採用する。

理由：

* 外部DB不要
* ローカルCLIとの相性が良い
* Dockerでも扱いやすい
* 開発初期の導入コストが低い

将来的にはPostgreSQLへ移行可能な構造にする。

---

## 8.1 データモデル

### tasks

```text
id
project_id
prompt
status
created_at
updated_at
started_at
completed_at
current_agent
result
error
```

### sessions

```text
id
task_id
created_at
last_attached_at
last_detached_at
status
```

### agents

```text
id
role
status
container_id
current_task
last_heartbeat
```

### events

```text
id
task_id
agent_id
event_type
payload
created_at
```

---

# 9. Event Log

AIエージェントの処理をイベントとして保存する。

例：

```text
TASK_CREATED
AGENT_ASSIGNED
AGENT_STARTED
TOOL_CALLED
CODE_CHANGED
TEST_STARTED
TEST_FAILED
REVIEW_STARTED
SECURITY_SCAN
AGENT_FINISHED
TASK_COMPLETED
```

これにより、ターミナルを閉じても処理履歴を復元できる。

---

# 10. Attach / Detach

## detach

ユーザーがターミナルから離脱しても、Taskは継続。

```text
agy attach task_xxxx
       │
       ▼
     RUNNING
       │
       ▼
Ctrl+C
       │
       ▼
     DETACHED
       │
       │
       └──── AI処理継続
```

## attach

後から再接続。

```bash
agy attach task_xxxx
```

すると現在の状態を取得。

```text
Task: task_xxxx
Status: RUNNING

Developer     ████████████ 100%
Tester        ████████░░░░  65%
Reviewer      waiting

Current:
Running integration tests...
```

---

# 11. tmuxへの依存を避ける

tmuxは便利だが、コア機能として依存しない。

悪い構成：

```text
agyCLI
  ↓
tmux
  ↓
Docker
```

推奨：

```text
agyCLI
  ↓
Persistent Manager
  ↓
Docker
```

tmuxは補助機能として利用可能にする。

---

# 12. Manager Daemon

Managerを常駐プロセスとして動作させる。

```bash
agy daemon start
```

停止：

```bash
agy daemon stop
```

状態：

```bash
agy daemon status
```

再起動：

```bash
agy daemon restart
```

---

# 13. Dockerの常駐化

現在のDocker Compose構成を発展させ、ManagerとAgentをタスク実行基盤として常駐させる。

重要な設定：

```yaml
restart: unless-stopped
```

を基本方針とする。

これによりManager停止時の自動復旧を可能にする。

---

# 14. Heartbeat

各AgentからManagerへ定期的にHeartbeatを送る。

```text
Agent
  │
  ├── heartbeat
  ├── heartbeat
  ├── heartbeat
  └── heartbeat
```

一定時間Heartbeatがない場合：

```text
Agent timeout
     ↓
Manager detects failure
     ↓
Container health check
     ↓
Restart
     ↓
Restore task
```

---

# 15. Crash Recovery

AIエージェントが停止してもTaskを失わない。

```text
Developer Agent
      ↓
     CRASH
      ↓
Manager detects
      ↓
Restart container
      ↓
Load task state
      ↓
Resume
```

---

# 16. Agent Scheduler

現在の固定Agent構成から、動的なタスク割り当てへ発展させる。

例：

```text
User Task
   ↓
Researcher
   ↓
Developer
   ↓
Tester
   ↓
Reviewer
   ↓
Security
   ↓
Developer
   ↓
Tester
   ↓
Completed
```

必要に応じて並列実行：

```text
             ┌── Tester
Developer ───┤
             └── Security
```

---

# 17. 並列Agent

例えば、

```text
Developer
     │
     ├── Feature A
     ├── Feature B
     └── Feature C
```

を別Agentで並列処理する。

ただし、同一ファイルへの同時編集を避けるため、Git Worktreeを使用する。

---

# 18. Git Worktree統合

Agentごとに独立したWorktreeを作成する。

```text
project/
├── main
├── worktree-developer
├── worktree-tester
├── worktree-reviewer
└── worktree-security
```

これによりAgent同士の変更衝突を減らす。

---

# 19. 自動Commit

Agentの作業単位ごとにGit Commitを作成する。

例：

```text
feat: implement authentication
test: add authentication tests
fix: resolve authentication failure
security: fix unsafe input handling
```

---

# 20. 自動Rollback

Agentによってコードが破壊された場合に備える。

```text
Before task
    ↓
Git checkpoint
    ↓
AI modification
    ↓
Tests
    ↓
FAIL
    ↓
Rollback
```

---

# 21. Human-in-the-Loop

完全自律実行だけでは危険なので、重要操作ではユーザー確認を可能にする。

例：

```text
AI wants to:

[ ] Delete 143 files
[ ] Modify database schema
[ ] Install system package
[ ] Push to remote
[ ] Merge branch

Approve? [y/N]
```

---

# 22. Approval Policy

プロジェクト単位で自動承認ルールを設定する。

```yaml
permissions:
  file_write: allow
  git_commit: allow
  git_push: ask
  package_install: ask
  database_delete: deny
  system_command: ask
```

---

# 23. セキュリティ強化

現在のSecurity Agentをさらに強化する。

チェック対象：

* Secrets
* API Keys
* SSH Keys
* `.env`
* Credentials
* Dangerous shell commands
* Dependency vulnerabilities
* OWASP
* CWE
* CVE

---

# 24. Agent権限制御

Agentごとに権限を設定する。

例：

```text
Researcher
  ├── Read: YES
  ├── Write: NO
  └── Execute: LIMITED

Developer
  ├── Read: YES
  ├── Write: YES
  └── Execute: YES

Reviewer
  ├── Read: YES
  ├── Write: NO
  └── Execute: LIMITED

Security
  ├── Read: YES
  ├── Write: NO
  └── Execute: SCAN ONLY
```

---

# 25. ログ管理

Agentごとのログを保存する。

```text
logs/
├── manager/
├── developer/
├── tester/
├── reviewer/
├── security/
└── researcher/
```

Task単位でも取得可能にする。

```bash
agy logs task_xxxx
```

---

# 26. ログのリアルタイム表示

```bash
agy attach task_xxxx
```

でイベントをリアルタイム表示。

```text
[12:01:03] Developer started
[12:01:10] Reading project
[12:02:14] Editing src/main.py
[12:03:01] Tester started
[12:03:17] Test failed
[12:03:25] Developer notified
```

---

# 27. Web Dashboard

CLIだけでなくWeb UIを追加する。

```text
http://localhost:8000
```

Dashboard：

```text
┌───────────────────────────────────────┐
│ agyCLI Multi-Agent Dashboard          │
├───────────────────────────────────────┤
│ Tasks                                 │
│                                       │
│ task-001  RUNNING                     │
│ task-002  COMPLETED                   │
│ task-003  FAILED                      │
│                                       │
├───────────────────────────────────────┤
│ Agents                                │
│                                       │
│ Developer   RUNNING                   │
│ Tester      RUNNING                   │
│ Reviewer    IDLE                      │
│ Security    RUNNING                   │
│ Researcher  IDLE                      │
└───────────────────────────────────────┘
```

---

# 28. リモート監視

将来的には別PCからTaskを確認できるようにする。

```text
PC
 │
 ├── agyCLI
 │
 └── Manager
       │
       ├── Developer
       ├── Tester
       ├── Reviewer
       ├── Security
       └── Researcher
```

別PC：

```text
agy remote connect <server>
agy task list
agy attach <task-id>
```

---

# 29. 認証

Manager APIには認証を追加する。

最低限：

```text
Local Mode
Remote Mode
```

を分離。

Local：

```text
127.0.0.1
```

Remote：

```text
TLS
API Token
```

を使用する。

---

# 30. Antigravity CLI認証状態

重要な追加機能として、Antigravity CLIの認証状態をTaskとは分離して管理する。

目標：

```text
Login
  ↓
Credential / Session
  ↓
Persistent Manager
  ↓
Agent
```

ターミナルを閉じても、

```text
Login state ≠ Terminal process
```

となる設計を目指す。

ただし、Antigravity CLI自体の認証情報を独自にコピー・抽出するのではなく、公式CLIが提供するログイン状態・設定・認証機構を利用する。

---

# 31. Credential Security

認証情報を以下へ保存しない。

```text
Git repository
Docker image
Task log
Agent prompt
Event payload
```

ログには、

```text
API_KEY=********
TOKEN=********
```

のようにマスクされた状態だけを残す。

---

# 32. 再起動後の復旧

PC再起動後もTask情報を復元できる構造にする。

```text
PC Shutdown
      ↓
Manager stopped
      ↓
PC Restart
      ↓
Docker start
      ↓
Manager start
      ↓
SQLite load
      ↓
Recover unfinished tasks
```

---

# 33. Queue System

Taskをキューに入れる。

```text
QUEUE

task-001
task-002
task-003
task-004
```

Agentの空き状況に応じて処理する。

---

# 34. 優先度

TaskにPriorityを追加。

```text
CRITICAL
HIGH
NORMAL
LOW
```

例：

```bash
agy run --priority high "重大なバグを修正"
```

---

# 35. Retry

Agent失敗時の自動Retry。

```yaml
retry:
  max_attempts: 3
  backoff: exponential
```

---

# 36. 無限ループ対策

AI Agentは自律実行時にループする可能性がある。

そのため、

```text
max_iterations
max_time
max_cost
max_retries
```

を設定可能にする。

例：

```yaml
limits:
  max_iterations: 50
  max_runtime_minutes: 120
  max_retries: 3
```

---

# 37. コスト管理

将来的にAPI型LLMにも対応する。

Taskごとに、

```text
tokens
requests
estimated_cost
runtime
```

を記録する。

---

# 38. Provider Abstraction

Antigravityだけに依存しない構造にする。

```text
Agent Runtime
      │
      ├── Antigravity
      ├── Gemini CLI
      ├── Claude Code
      ├── Codex
      └── Local LLM
```

Agent APIを共通化する。

---

# 39. MCP対応

将来的にMCP Serverとして利用可能にする。

```text
Claude
  │
Codex
  │
Antigravity
  │
  ▼
agyCLI MCP
  │
  ▼
Multi-Agent Manager
```

これにより外部AIからagyCLIを呼び出せる。

---

# 40. Research AgentのWeb調査

Researcher AgentにWeb調査能力を追加する。

処理例：

```text
User Request
    ↓
Researcher
    ↓
Search
    ↓
Documentation
    ↓
GitHub
    ↓
Issue
    ↓
Research Report
    ↓
Developer
```

情報源URLも記録する。

---

# 41. Project Memory

プロジェクトごとの長期記憶を保存する。

```text
.memory/
├── architecture.md
├── decisions.md
├── known-issues.md
├── conventions.md
└── agent-history.json
```

これにより毎回ゼロからプロジェクトを理解する必要を減らす。

---

# 42. Agent間Communication

Agent同士が直接自由に会話するのではなく、Managerを経由する。

```text
Developer
    │
    ▼
 Manager
    │
    ▼
 Tester
```

通信内容をイベントとして保存する。

---

# 43. Agent成果物

Agentの結果を明確に分類する。

```text
artifact/
├── source/
├── test/
├── report/
├── security/
└── research/
```

---

# 44. 最終レビュー

Task完了前に以下を必須化する。

```text
Developer
    ↓
Tester
    ↓
Reviewer
    ↓
Security
    ↓
Manager
    ↓
DONE
```

最低条件：

* Build成功
* Test成功
* Review成功
* Security重大問題なし

---

# 45. Failure Handling

例えばTesterが失敗した場合：

```text
Developer
    ↓
Tester
    ↓
FAIL
    ↓
Manager
    ↓
Developer
    ↓
FIX
    ↓
Tester
```

最大Retry回数を超えた場合：

```text
FAILED
```

としてユーザーへ通知する。

---

# 46. CLI設計

最終的なCLIは以下を目標とする。

```bash
agy run
agy run --detach
agy task list
agy task status
agy task stop
agy task resume
agy attach
agy logs
agy agent list
agy agent status
agy daemon start
agy daemon stop
agy daemon status
agy project init
agy project status
agy config
agy doctor
```

---

# 47. `agy doctor`

環境診断機能を追加する。

チェック対象：

```text
Docker
Docker Compose
Antigravity CLI
Login status
Git
Git Worktree
Python
Network
Disk
Memory
Container health
Manager health
Agent health
```

出力例：

```text
agy doctor

✓ Docker
✓ Docker Compose
✓ Git
✓ Antigravity CLI
✓ Authentication
✓ Manager
✓ Developer Agent
✓ Tester Agent
✓ Reviewer Agent
✓ Security Agent
✓ Researcher Agent

System is ready.
```

---

# 48. Configuration

ユーザー設定を一元管理する。

```text
~/.config/agy/
├── config.yaml
├── credentials/
├── sessions/
├── logs/
└── cache/
```

プロジェクト設定：

```text
.agy/
├── config.yaml
├── agents/
├── tasks/
└── memory/
```

---

# 49. Docker Healthcheck

各コンテナにHealthcheckを追加。

```text
healthy
unhealthy
starting
```

ManagerはAgent状態を監視する。

---

# 50. テスト

## Unit Test

対象：

* Task Manager
* Session Manager
* Scheduler
* Event Store
* Config
* Authentication state
* Recovery

## Integration Test

```text
Manager
  ↓
Developer
  ↓
Tester
  ↓
Reviewer
  ↓
Security
```

## Recovery Test

```text
Task running
↓
Manager kill
↓
Restart
↓
Task recovery
```

## Terminal Test

```text
agy run
↓
Terminal close
↓
Wait
↓
New terminal
↓
agy task list
↓
agy attach
```

このテストを必須化する。

---

# 51. 長時間実行テスト

最低でも以下をテストする。

```text
1 hour
6 hours
12 hours
24 hours
```

Taskが途中で停止せず、ログ・状態が維持されることを確認する。

---

# 52. セキュリティテスト

以下をテストする。

* AgentからHostへの不正アクセス
* Docker privilege escalation
* Secret leakage
* Prompt injection
* Malicious repository
* Shell command injection
* Path traversal
* Git credential leakage
* API token leakage

---

# 53. 実装フェーズ

## Phase 1 — 基盤整理

* [ ] CLI構造整理
* [ ] Manager API整理
* [ ] Agent API整理
* [ ] Configuration設計
* [ ] Logging設計

---

## Phase 2 — Task Persistence

* [ ] SQLite導入
* [ ] Task Model
* [ ] Session Model
* [ ] Event Model
* [ ] Task Manager
* [ ] Event Store

---

## Phase 3 — Detached Execution

* [ ] `agy run`
* [ ] `agy run --detach`
* [ ] `agy attach`
* [ ] `agy task status`
* [ ] `agy task list`
* [ ] `agy logs`
* [ ] terminal disconnect対応

---

## Phase 4 — Manager Daemon

* [ ] Manager常駐化
* [ ] Healthcheck
* [ ] Heartbeat
* [ ] 自動Restart
* [ ] Crash Recovery
* [ ] Task Recovery

---

## Phase 5 — Agent Scheduler

* [ ] Agent Registry
* [ ] Task Queue
* [ ] Priority
* [ ] Retry
* [ ] Parallel execution
* [ ] Dependency graph

---

## Phase 6 — Git統合

* [ ] Git Worktree
* [ ] Auto Commit
* [ ] Checkpoint
* [ ] Rollback
* [ ] Merge
* [ ] Conflict handling

---

## Phase 7 — Security

* [ ] Permission system
* [ ] Secret masking
* [ ] Sandbox
* [ ] Command approval
* [ ] Security Agent強化
* [ ] Audit Log

---

## Phase 8 — Web Dashboard

* [ ] Task dashboard
* [ ] Agent dashboard
* [ ] Real-time logs
* [ ] Task control
* [ ] Agent control
* [ ] System health

---

## Phase 9 — Provider/MCP

* [ ] Provider abstraction
* [ ] Antigravity backend
* [ ] Gemini backend
* [ ] Claude backend
* [ ] Codex backend
* [ ] MCP Server

---

# 54. 推奨ディレクトリ構成

最終的には以下を目標とする。

```text
agyCLI-multicoding/
│
├── containers/
│   ├── developer/
│   ├── tester/
│   ├── reviewer/
│   ├── security/
│   └── researcher/
│
├── project/
│   ├── agents/
│   ├── src/
│   │   ├── cli/
│   │   ├── manager/
│   │   ├── worker/
│   │   ├── scheduler/
│   │   ├── session/
│   │   ├── task/
│   │   ├── event/
│   │   ├── storage/
│   │   ├── recovery/
│   │   ├── security/
│   │   └── provider/
│   │
│   ├── tests/
│   └── project.yaml
│
├── dashboard/
│
├── docs/
│   ├── architecture.md
│   ├── cli.md
│   ├── agents.md
│   ├── security.md
│   └── recovery.md
│
├── .github/
│   └── workflows/
│
├── docker-compose.yml
├── Dockerfile
├── mag
└── README.md
```

---

# 55. 最終アーキテクチャ

```text
                    User
                     │
                     ▼
                ┌─────────┐
                │ agy CLI │
                └────┬────┘
                     │
              attach / detach
                     │
                     ▼
        ┌────────────────────────┐
        │   Persistent Manager   │
        │                        │
        │ Task Manager           │
        │ Session Manager        │
        │ Scheduler              │
        │ Event Bus              │
        │ Recovery Manager       │
        │ Security Manager       │
        └───────────┬────────────┘
                    │
           ┌────────┼────────┐
           │        │        │
           ▼        ▼        ▼
       Developer  Tester  Reviewer
           │        │        │
           └────────┼────────┘
                    │
             ┌──────┴──────┐
             ▼             ▼
         Security      Researcher
             │             │
             └──────┬──────┘
                    ▼
              Git / Project
                    │
                    ▼
              Completed Code
```

---

# 56. 目標とするユーザー体験

最終的には以下の操作だけでAI開発を開始できる状態を目指す。

```bash
agy run "このプロジェクトを完成させてください"
```

すると、

```text
Task created: task_01J...

Researcher  → Researching
Developer   → Implementing
Tester      → Waiting
Reviewer    → Waiting
Security    → Waiting

Detach safely with:
agy attach task_01J...
```

ここでターミナルを閉じる。

数時間後、

```bash
agy task list
```

すると、

```text
ID              STATUS       PROGRESS
task_01J...     COMPLETED    100%
```

さらに、

```bash
agy attach task_01J...
```

で結果を確認できる。

---

# 57. 最終目標

`agyCLI-multicoding` を以下の5段階で発展させる。

```text
現在

Multi-Agent Docker Prototype
        │
        ▼
Phase 1

Persistent Multi-Agent CLI
        │
        ▼
Phase 2

Detached Autonomous Agent
        │
        ▼
Phase 3

Self-Recovering Development System
        │
        ▼
Phase 4

Multi-Agent Development Platform
        │
        ▼
Phase 5

Autonomous Software Engineering Platform
```

最終的なコンセプトは、

> **「AIを増やす」のではなく、「AIを開発チームとして動かす」**

とする。

ユーザーがターミナルを開いてAIを監視し続けるのではなく、

```text
Taskを投入
   ↓
AI Teamが計画
   ↓
Research
   ↓
Implementation
   ↓
Testing
   ↓
Review
   ↓
Security
   ↓
Fix
   ↓
Final Verification
   ↓
完成
```

までをManagerが管理する。

---

# 58. 優先順位

実装優先順位は以下とする。

| 優先度 | 機能                  | 重要度 |
| --- | ------------------- | --: |
| P0  | Task Persistence    |  必須 |
| P0  | Detached Execution  |  必須 |
| P0  | Session Manager     |  必須 |
| P0  | Manager Daemon      |  必須 |
| P0  | Crash Recovery      |  必須 |
| P0  | Heartbeat           |  必須 |
| P1  | Task Queue          |   高 |
| P1  | Agent Scheduler     |   高 |
| P1  | Git Worktree        |   高 |
| P1  | Auto Retry          |   高 |
| P1  | Logging             |   高 |
| P1  | Secret Management   |   高 |
| P2  | Web Dashboard       |   中 |
| P2  | Project Memory      |   中 |
| P2  | MCP                 |   中 |
| P2  | Remote Access       |   中 |
| P3  | Multi-Provider      |  将来 |
| P3  | Cost Optimization   |  将来 |
| P3  | Distributed Manager |  将来 |

---

# 59. MVP

最初から全機能を実装せず、まず以下を完成させる。

```text
MVP

[1] agy run
       ↓
[2] Task ID発行
       ↓
[3] SQLite保存
       ↓
[4] Manager常駐
       ↓
[5] Agent実行
       ↓
[6] Terminal detach
       ↓
[7] Task継続
       ↓
[8] 新しいTerminal
       ↓
[9] agy attach
       ↓
[10] 結果確認
```

この10項目が動けば、今回の最大の目的である

> **「ログイン・実行状態を維持したままターミナルを閉じ、後から再接続する」**

が実現する。

---

# 60. 成功条件

以下をすべて満たした時点でPhase 1完成とする。

* [ ] `agy run` でTaskを作成できる
* [ ] Task IDが発行される
* [ ] Task状態がSQLiteに保存される
* [ ] Managerが常駐する
* [ ] AgentがDockerで実行される
* [ ] ターミナルを閉じてもTaskが継続する
* [ ] 新しいターミナルからTaskを確認できる
* [ ] `agy attach <task-id>` で再接続できる
* [ ] ログを取得できる
* [ ] Manager再起動後もTask状態を復元できる
* [ ] Agent停止時に自動復旧できる
* [ ] 認証情報がログ・Git・Task DBへ漏洩しない

---

# 61. 結論

`agyCLI-multicoding` の現在の構成は、ManagerとDeveloper / Tester / Reviewer / Security / ResearcherをDockerで分離するという、Multi-Agent開発システムの基礎としては適切である。

しかし、現段階では「Agentを動かす基盤」に近く、長時間の自律開発を行うために必要な、

* 永続Task
* Session
* Detached execution
* Daemon
* Heartbeat
* Recovery
* Queue
* Scheduler
* Git Worktree
* Security
* Audit Log
* Dashboard

が不足している。

したがって、最優先で実装するべきなのは**AIモデルの追加ではなく、AIの実行状態をプロセスから分離すること**である。

最終的には、

```text
Terminal
   ↓
CLI
   ↓
Persistent Manager
   ↓
Task Queue
   ↓
Multi-Agent Team
   ↓
Autonomous Development
```

という構造に変更する。

これにより、ユーザーは「AIが終わるまでターミナルの前で待つ」のではなく、

```bash
agy run "プロジェクトを完成させて"
```

だけを実行して離脱し、後から

```bash
agy task list
agy attach <task-id>
```

で開発状況を確認できる。

これを `agyCLI-multicoding` の次期バージョンにおける**最重要機能**とする。
