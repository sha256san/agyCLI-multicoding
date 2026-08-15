# Multi-Agent Development Orchestrator

## 複数AIエージェントによる自律型ソフトウェア開発環境

---

# 1. プロジェクト概要

## 1.1 目的

本プロジェクトでは、複数のAIエージェントを独立したコンテナ環境で動作させ、それらを1つの親エージェントが統括することで、**AI同士が役割分担しながらソフトウェアプロジェクトを開発できる環境**を構築する。

単純にAntigravityを複数起動するだけではなく、

* タスク分解
* エージェントへのタスク割り当て
* 開発
* テスト
* コードレビュー
* セキュリティチェック
* エラー解析
* 修正指示
* Git管理
* 結果の収集
* エージェント間の情報共有
* 最終的な成果物の統合

までを自動化する。

---

# 2. このプロジェクトで解決する問題

現在の複数AI利用には以下の問題がある。

## 2.1 複数AIを起動するだけでは協調できない

例えば、

```text
Agent A
Agent B
Agent C
Agent D
Agent E
```

を同時に起動しても、

```text
A → コードを書く
B → コードを書く
C → コードを書く
D → コードを書く
E → コードを書く
```

だけでは、AI同士の連携がない。

そこで本プロジェクトでは、

```text
                    Manager
                       │
       ┌───────────────┼───────────────┐
       ↓               ↓               ↓
   Developer         Tester          Reviewer
       │               │               │
       └───────────────┼───────────────┘
                       ↓
                    Manager
```

という構造を作る。

---

# 3. Multigravityとの差別化

本プロジェクトは、単純な複数Antigravityプロファイル管理ツールを目指さない。

## Multigravity型

```text
Antigravity
├── Account A
├── Account B
├── Account C
├── Account D
└── Account E
```

主目的：

> 複数の独立したAntigravity環境を利用する。

---

## 本プロジェクト

```text
                    Manager
                       │
             ┌─────────┼─────────┐
             ↓         ↓         ↓
             A         B         C
          Developer   Tester   Security
             │         │         │
             └─────────┼─────────┘
                       ↓
                    Manager
                       │
                       ↓
                  Project
```

主目的：

> 複数のAIを「開発チーム」として協調動作させる。

---

# 4. 基本コンセプト

プロジェクトの基本思想は以下とする。

> **AIを増やすのではなく、AIを組織化する。**

各AIには役割を与える。

例えば、

| Agent   | 役割        |
| ------- | --------- |
| Manager | 全体管理      |
| Agent A | 実装        |
| Agent B | テスト       |
| Agent C | コードレビュー   |
| Agent D | セキュリティ    |
| Agent E | 調査・ドキュメント |

とする。

---

# 5. システム全体構成

## 5.1 推奨構成

```text
Windows / Host
│
└── Ubuntu 26.04
    │
    ├── Project
    │   ├── src/
    │   ├── tests/
    │   ├── docs/
    │   ├── .git/
    │   └── ...
    │
    ├── Manager / Antigravity CLI
    │
    └── Communication Layer
            │
            ▼
        Ubuntu 24.04
        │
        ├── account F
        │
        ├── Container 1
        │   └── account A
        │
        ├── Container 2
        │   └── account B
        │
        ├── Container 3
        │   └── account C
        │
        ├── Container 4
        │   └── account D
        │
        └── Container 5
            └── account E
```

---

# 6. 各環境の役割

## 6.1 Ubuntu 26.04

プロジェクトの中心。

担当：

* プロジェクトファイル管理
* Git管理
* Manager Agent
* タスク管理
* エージェント管理
* 結果収集
* 最終判断
* ログ管理

---

## 6.2 Ubuntu 24.04

Agent実行基盤。

担当：

* Container管理
* Worker Agent実行
* 隔離環境提供
* テスト環境提供

---

## 6.3 account F

Ubuntu 24.04ホスト側の管理ユーザー。

担当：

* Docker/Podman管理
* Container起動
* Container停止
* ログ取得
* Volume管理
* ネットワーク管理

---

# 7. Container構成

## Container 1

```text
Container 1
└── account A
```

役割：

> Developer Agent

担当：

* コード実装
* バグ修正
* リファクタリング
* 新機能追加

---

## Container 2

```text
Container 2
└── account B
```

役割：

> Tester Agent

担当：

* Unit Test
* Integration Test
* Build
* 実行確認
* 回帰テスト

---

## Container 3

```text
Container 3
└── account C
```

役割：

> Reviewer Agent

担当：

* コードレビュー
* 設計レビュー
* 可読性確認
* 保守性確認
* API設計確認

---

## Container 4

```text
Container 4
└── account D
```

役割：

> Security Agent

担当：

* 脆弱性確認
* 依存関係確認
* 危険なコードの検出
* 秘密情報混入確認
* CVEチェック

---

## Container 5

```text
Container 5
└── account E
```

役割：

> Research / Documentation Agent

担当：

* 技術調査
* ドキュメント作成
* README更新
* API仕様調査
* 既存実装調査

---

# 8. Manager Agent

最重要コンポーネント。

Managerは直接大量のコードを書くことを主目的としない。

主な仕事は、

```text
ユーザー
  ↓
Manager
  ↓
タスク分解
  ↓
Worker割り当て
  ↓
結果収集
  ↓
結果評価
  ↓
追加指示
  ↓
統合
```

とする。

---

# 9. タスク分解システム

ユーザーが、

```text
RustでCLIツールを作ってください。
```

と入力した場合、Managerが自動的に、

```text
TASK-001
プロジェクト設計

TASK-002
CLI実装

TASK-003
テスト実装

TASK-004
セキュリティチェック

TASK-005
README作成
```

に分解する。

---

# 10. タスク情報

各タスクには最低限以下を持たせる。

```text
Task ID
Title
Description
Priority
Assigned Agent
Status
Dependencies
Input
Output
Created At
Updated At
```

例：

```json
{
  "task_id": "TASK-002",
  "title": "CLI implementation",
  "assigned_agent": "agent-a",
  "priority": "high",
  "status": "running",
  "dependencies": ["TASK-001"]
}
```

---

# 11. タスク状態

以下の状態を実装する。

```text
PENDING
    ↓
ASSIGNED
    ↓
RUNNING
    ↓
REVIEW
    ↓
TESTING
    ↓
COMPLETED
```

失敗した場合：

```text
RUNNING
   ↓
FAILED
   ↓
RETRY
   ↓
RUNNING
```

---

# 12. Agent間通信

最初のバージョンでは、複雑な分散システムを作らず、

```text
Manager
   │
   ├── command
   ↓
Worker
   │
   └── result
   ↓
Manager
```

という単純な通信方式から始める。

通信方法の候補：

1. HTTP API
2. Unix Socket
3. JSONファイル
4. Redis
5. メッセージキュー

MVPでは、

> **HTTP + JSON**

を推奨する。

---

# 13. Worker API

各Workerは最低限以下のAPIを持つ。

```text
POST /task
GET  /status
GET  /result
POST /cancel
GET  /health
```

---

# 14. `/task`

ManagerがWorkerにタスクを送る。

例：

```json
{
  "task_id": "TASK-002",
  "type": "implementation",
  "description": "CLI parserを実装してください",
  "repository": "/workspace/project",
  "branch": "agent-a/task-002"
}
```

---

# 15. `/status`

ManagerがWorkerの状態を取得する。

```json
{
  "agent": "agent-a",
  "status": "running",
  "task_id": "TASK-002"
}
```

---

# 16. `/result`

Workerの結果を取得する。

```json
{
  "task_id": "TASK-002",
  "status": "completed",
  "commit": "a82f91c",
  "tests_passed": true,
  "summary": "CLI parser implemented."
}
```

---

# 17. Git管理

複数Agentが同じファイルを同時に編集すると競合が発生する。

そのため、各AgentにGit worktreeまたはbranchを割り当てる。

```text
Project
│
├── main
│
├── agent-a/task-001
├── agent-b/task-002
├── agent-c/task-003
├── agent-d/task-004
└── agent-e/task-005
```

---

# 18. 推奨開発フロー

```text
ユーザー
   ↓
Manager
   ↓
要件分析
   ↓
タスク分解
   ↓
Git branch作成
   ↓
Agent Aへ実装依頼
   ↓
Agent A実装
   ↓
Agent Bテスト
   ↓
Agent Cレビュー
   ↓
Agent Dセキュリティチェック
   ↓
Agent Eドキュメント
   ↓
Manager評価
   ↓
問題あり？
 ┌─┴─┐
YES  NO
 │    │
 ↓    ↓
修正  Merge
 │
 ↓
再テスト
```

---

# 19. Agent間の依存関係

すべてのAgentを同時に実行する必要はない。

例えば、

```text
設計
 ↓
実装
 ↓
テスト
 ↓
レビュー
 ↓
セキュリティ
 ↓
統合
```

という依存関係を設定する。

一方、独立したタスクは並列実行する。

```text
              Manager
                 │
        ┌────────┼────────┐
        ↓        ↓        ↓
       A         B        C
    実装1      実装2     調査
        │        │        │
        └────────┼────────┘
                 ↓
              Manager
```

---

# 20. コンテナ分離

各Agentをコンテナに入れる。

理由：

* 依存関係の衝突防止
* Python環境の分離
* Rust環境の分離
* Node.js環境の分離
* ファイルシステム分離
* プロセス分離
* 破損時の再作成
* セキュリティ向上

---

# 21. Workspace

全Agentが同じホストディレクトリを直接書き換えないようにする。

推奨：

```text
Ubuntu 26.04
└── project/
    └── git repository
```

Agent側：

```text
Container A
└── /workspace

Container B
└── /workspace

Container C
└── /workspace
```

ただし、実際にはGit worktreeまたは一時workspaceを使用する。

---

# 22. Agentの権限

Agentごとに権限を制限する。

例えば、

```text
Developer
├── read project
├── write source
├── run tests
└── git commit

Tester
├── read source
├── run tests
└── write test result

Reviewer
├── read source
└── write review

Security
├── read source
├── run scanner
└── write security report
```

特にSecurity Agentなどに不要な書き込み権限を与えない。

---

# 23. Managerの権限

Managerは最も強い権限を持つが、直接すべてを操作させるのではなく、可能な限りAPI経由でWorkerを管理する。

```text
Manager
├── Task Manager
├── Git Manager
├── Worker Manager
├── Logger
└── Project Manager
```

---

# 24. ログシステム

すべてのAgentの活動を記録する。

```text
logs/
├── manager.log
├── agent-a.log
├── agent-b.log
├── agent-c.log
├── agent-d.log
└── agent-e.log
```

さらに、

```text
logs/tasks/
├── TASK-001.json
├── TASK-002.json
├── TASK-003.json
└── ...
```

とする。

---

# 25. Agentの出力形式

AIの自由形式の文章だけに依存しない。

Workerは、

```json
{
  "status": "success",
  "task_id": "TASK-001",
  "summary": "...",
  "files_changed": [],
  "tests": [],
  "errors": [],
  "commit": ""
}
```

のような構造化データを返す。

これによりManagerが機械的に判断できる。

---

# 26. エラー処理

Workerが失敗した場合、

```text
Agent A
 ↓
ERROR
 ↓
Manager
 ↓
原因分析
 ↓
Retry
```

とする。

同じ処理を無限に繰り返さない。

例えば、

```text
max_retry = 3
```

とする。

3回失敗したら、

```text
FAILED_PERMANENTLY
```

に変更し、ユーザーに報告する。

---

# 27. Build / Test自動化

ManagerはWorkerから、

```text
build
test
lint
security scan
```

の結果を取得する。

例：

```text
cargo build
cargo test
cargo clippy
```

など。

プロジェクトによって実行コマンドを変更できるようにする。

---

# 28. 自動修正ループ

本システムの重要機能。

```text
Build
 ↓
Error
 ↓
Manager
 ↓
Error Analysis
 ↓
Developer Agent
 ↓
Fix
 ↓
Build
```

これを成功するまで繰り返す。

ただし、

```text
MAX_RETRY=3
```

などの上限を設ける。

---

# 29. コードレビュー

Developer Agentが作ったコードをReviewer Agentに渡す。

Reviewerは、

```text
Correctness
Readability
Maintainability
Performance
Security
Architecture
```

を確認する。

結果：

```json
{
  "approved": false,
  "severity": "high",
  "issues": [
    {
      "file": "src/main.rs",
      "line": 42,
      "message": "..."
    }
  ]
}
```

---

# 30. セキュリティ機能

Security Agentでは将来的に、

* CVE
* OSV
* GitHub Advisory Database
* cargo-audit
* npm audit
* pip-audit
* Semgrep
* Trivy

などを利用できるようにする。

さらに、以前検討していたCVE関連プロジェクトとの連携も可能にする。

---

# 31. envdoctorとの連携

環境依存問題を検出するため、

```text
Build Error
    ↓
通常のコード原因を調査
    ↓
原因不明
    ↓
envdoctor
    ↓
環境診断
    ↓
PATH
Compiler
Python
Rust
Library
GPU
Docker
Kernel
    ↓
Manager
```

という流れを実装する。

これにより、

> コードが悪いのか、環境が悪いのか

を自動的に切り分けられる。

---

# 32. jpcargoとの連携

Rustプロジェクトの場合、

```text
cargo build
```

でエラーが発生したら、

```text
jpcargo
```

による日本語エラー解析をWorkerに利用させる。

例えば、

```text
Rust Compiler
    ↓
jpcargo
    ↓
日本語診断
    ↓
Manager
    ↓
Developer Agent
```

という連携を可能にする。

---

# 33. Agent定義

Agentごとに設定ファイルを作る。

例：

```text
agents/
├── developer.yaml
├── tester.yaml
├── reviewer.yaml
├── security.yaml
└── researcher.yaml
```

例：

```yaml
name: developer
role: developer

permissions:
  read:
    - project

  write:
    - source
    - tests

commands:
  - cargo
  - git
```

---

# 34. Project設定

```text
project.yaml
```

を作る。

例：

```yaml
name: example-project

language:
  - rust

target:
  os: ubuntu
  version: "24.04"

agents:
  developer: agent-a
  tester: agent-b
  reviewer: agent-c
  security: agent-d
  researcher: agent-e
```

---

# 35. CLI設計

最初のCLIは以下を目標とする。

```bash
mag init
mag start
mag stop
mag status
mag agents
mag task
mag logs
mag project
mag test
mag review
mag merge
```

`mag` は仮称。

正式名称は後で決定する。

---

# 36. CLI例

プロジェクト作成：

```bash
mag init my-project
```

起動：

```bash
mag start
```

状態確認：

```bash
mag status
```

出力例：

```text
Manager      RUNNING

Agent A      RUNNING    TASK-003
Agent B      TESTING    TASK-004
Agent C      REVIEW     TASK-003
Agent D      IDLE
Agent E      RESEARCH   TASK-006
```

---

# 37. ユーザーからの指示

最終的には、

```bash
mag "RustでWeb APIサーバーを作ってください"
```

のようなインターフェースを目標とする。

Managerが自動的に、

```text
1. 要件分析
2. 設計
3. タスク分解
4. Agent割り当て
5. 実装
6. テスト
7. レビュー
8. セキュリティ確認
9. 修正
10. 統合
```

を実行する。

---

# 38. MVP

最初からすべてを実装しない。

最初の完成目標を以下に限定する。

```text
Ubuntu 26.04
       │
       ▼
Manager
       │
       ├── Container A
       ├── Container B
       └── Container C
```

3 Agentだけで実験する。

役割：

```text
A = Developer
B = Tester
C = Reviewer
```

---

# 39. MVPで実装する機能

必須：

* [ ] Container起動
* [ ] Container停止
* [ ] Agent登録
* [ ] Agentへのタスク送信
* [ ] Agent状態取得
* [ ] 結果取得
* [ ] Git branch作成
* [ ] Git commit取得
* [ ] Test実行
* [ ] Managerによる結果判定
* [ ] ログ保存

---

# 40. Phase 1

## 開発環境構築

対象：

```text
Ubuntu 26.04
Ubuntu 24.04
Docker
Git
Python
Rust
Antigravity CLI
```

作業：

1. Ubuntu 26.04を準備する。
2. Ubuntu 24.04を準備する。
3. Dockerをインストールする。
4. Gitをインストールする。
5. SSHまたはHTTP通信を構築する。
6. Container 1〜5を作成する。
7. 各Containerに専用ユーザーを作成する。
8. Container間通信を確認する。

完了条件：

```text
26.04 → 24.04
26.04 → Container 1〜5
```

の通信が成功すること。

---

# 41. Phase 2

## Worker Agentの実装

最初はAI機能を複雑にしない。

Worker APIを作る。

```text
POST /task
GET /status
GET /result
POST /cancel
GET /health
```

テスト用に、

```text
sleep 10
```

などのダミータスクを実行させる。

完了条件：

```text
Manager
 ↓
Worker
 ↓
Task
 ↓
Result
 ↓
Manager
```

が正常に動くこと。

---

# 42. Phase 3

## Manager実装

Managerに以下を実装する。

```text
Task Manager
Worker Manager
Git Manager
Result Manager
Log Manager
```

最初はルールベースでもよい。

例：

```text
implementation → Developer
testing        → Tester
review         → Reviewer
security       → Security
research       → Researcher
```

---

# 43. Phase 4

## Git連携

以下を自動化する。

```text
Task
 ↓
branch作成
 ↓
Worker
 ↓
commit
 ↓
Manager
```

Agentごとにbranchを作る。

---

# 44. Phase 5

## AIによるタスク分解

ここからManagerにLLMを導入する。

入力：

```text
「RustでCLIアプリを作ってください」
```

出力：

```text
TASK-001 設計
TASK-002 CLI実装
TASK-003 テスト
TASK-004 セキュリティ
TASK-005 README
```

---

# 45. Phase 6

## Agent間レビュー

以下を実装する。

```text
Developer
    ↓
Reviewer
    ↓
Review Result
    ↓
Manager
```

レビューで問題があった場合、

```text
Reviewer
 ↓
Manager
 ↓
Developer
```

と再指示する。

---

# 46. Phase 7

## 自動テストループ

```text
Developer
 ↓
Build
 ↓
Test
 ↓
Failure?
 ├── YES → Developer
 └── NO
       ↓
    Reviewer
```

---

# 47. Phase 8

## Security Agent

Security Agentを追加する。

```text
Code
 ↓
Security Agent
 ↓
CVE / Dependency / Static Analysis
 ↓
Report
```

---

# 48. Phase 9

## 5 Agent構成

最終的に、

```text
Manager
│
├── A Developer
├── B Tester
├── C Reviewer
├── D Security
└── E Researcher
```

を実現する。

---

# 49. Phase 10

## 自律開発

最終目標：

```text
ユーザー
 ↓
「○○を作って」
 ↓
Manager
 ↓
タスク分解
 ↓
Agent A〜E
 ↓
実装
 ↓
テスト
 ↓
レビュー
 ↓
セキュリティ
 ↓
修正
 ↓
再テスト
 ↓
Git merge
 ↓
完成
```

ユーザーは途中で細かい指示を出さなくてもよい状態を目指す。

---

# 50. 安全装置

AIに完全な権限を与えない。

必須機能：

```text
MAX_RETRY
MAX_EXECUTION_TIME
MAX_TASK_COUNT
COMMAND_ALLOWLIST
FILE_ACCESS_LIMIT
NETWORK_ACCESS_LIMIT
```

危険な操作についてはManagerからユーザーに確認を求める。

例：

```text
WARNING

Agent D requested:

rm -rf /workspace/build

Allow? [y/N]
```

---

# 51. Containerセキュリティ

Worker Containerでは、

* root実行を避ける
* capabilitiesを削減
* read-only filesystemの検討
* workspaceのみ書き込み可能にする
* network制限
* CPU制限
* RAM制限
* timeout設定

を行う。

---

# 52. リソース管理

5 Agentを同時に動かすため、

```text
Container
├── CPU limit
├── Memory limit
├── Process limit
└── Network limit
```

を設定する。

特にLLM Agentが大量のプロセスを起動しないようにする。

---

# 53. 障害発生時

Containerが落ちた場合：

```text
Container
 ↓
Health Check failure
 ↓
Manager
 ↓
Container restart
 ↓
Task recovery
```

を実装する。

---

# 54. Agentが暴走した場合

例：

```text
Agent A
 ↓
大量コマンド実行
 ↓
Manager検出
 ↓
Agent停止
 ↓
Container停止
 ↓
ログ保存
```

その後、

```text
原因分析
 ↓
再起動
```

またはユーザーへ報告する。

---

# 55. データ構造

最低限以下を管理する。

```text
projects
agents
tasks
jobs
logs
commits
reviews
test_results
security_results
```

将来的にはSQLite/PostgreSQLを使用する。

MVPではSQLiteで十分。

---

# 56. 推奨技術スタック

## Manager

候補：

```text
Rust
```

または、

```text
Python
```

最初のプロトタイプではPythonを推奨。

理由：

* API実装が容易
* Docker操作が容易
* LLM API連携が容易
* JSON処理が容易
* 非同期処理が容易

安定版ではRustへの移行を検討する。

---

## Worker

```text
Python
```

を推奨。

---

## API

```text
FastAPI
```

---

## Container

```text
Docker
```

---

## Database

MVP：

```text
SQLite
```

将来：

```text
PostgreSQL
```

---

## Communication

MVP：

```text
HTTP + JSON
```

将来：

```text
Redis
Message Queue
WebSocket
```

---

# 57. 推奨ディレクトリ構成

```text
multi-agent/
│
├── manager/
│   ├── main.py
│   ├── api/
│   ├── agents/
│   ├── tasks/
│   ├── git/
│   ├── docker/
│   ├── database/
│   └── logs/
│
├── worker/
│   ├── main.py
│   ├── api/
│   ├── executor/
│   └── git/
│
├── agents/
│   ├── developer.yaml
│   ├── tester.yaml
│   ├── reviewer.yaml
│   ├── security.yaml
│   └── researcher.yaml
│
├── containers/
│   ├── developer/
│   │   └── Dockerfile
│   ├── tester/
│   │   └── Dockerfile
│   ├── reviewer/
│   │   └── Dockerfile
│   ├── security/
│   │   └── Dockerfile
│   └── researcher/
│       └── Dockerfile
│
├── project/
│
├── tests/
│
├── docs/
│
├── docker-compose.yml
│
├── project.yaml
│
└── README.md
```

---

# 58. Docker Compose

最終的には、

```text
services:

  manager:
    ...

  agent-a:
    ...

  agent-b:
    ...

  agent-c:
    ...

  agent-d:
    ...

  agent-e:
    ...
```

という構成にする。

---

# 59. 開発優先順位

優先順位は以下とする。

```text
1. Container
2. Worker API
3. Manager
4. Task system
5. Git
6. AI task decomposition
7. Test
8. Review
9. Security
10. Autonomous loop
```

AI機能を最初から作り込まない。

まず、

```text
Manager → Worker → Result
```

を確実に動かす。

---

# 60. 最初の完成目標

最初の目標は以下とする。

```text
$ mag start

Manager: RUNNING
Agent A: RUNNING
Agent B: RUNNING
Agent C: RUNNING
```

その後、

```text
$ mag "PythonでHello World CLIを作ってください"
```

と入力。

Manager：

```text
TASK-001 → Developer
TASK-002 → Tester
TASK-003 → Reviewer
```

Worker：

```text
Developer
 ↓
コード作成
 ↓
commit
```

Tester：

```text
test
 ↓
PASS
```

Reviewer：

```text
review
 ↓
APPROVED
```

Manager：

```text
TASK COMPLETED
```

ここまで動けばMVP完成とする。

---

# 61. 最終完成形

最終的には、

```text
                         USER
                           │
                           ▼
                  ┌────────────────┐
                  │    Manager     │
                  │   Antigravity  │
                  └───────┬────────┘
                          │
                  Task Decomposition
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
     Developer         Tester          Reviewer
        /A                /B               /C
          │               │               │
          └───────────────┼───────────────┘
                          │
              ┌───────────┴───────────┐
              ▼                       ▼
          Security                  Research
             /D                       /E
              │                       │
              └───────────┬───────────┘
                          ▼
                       Manager
                          │
                   Evaluation
                          │
                   ┌──────┴──────┐
                   │             │
                 FAIL          PASS
                   │             │
                   ▼             ▼
                 Retry          Merge
                   │             │
                   └──────┐      │
                          ▼      ▼
                       Project Complete
```

---

# 62. プロジェクトの最終的な差別化

本プロジェクトは、単なる「Antigravity複数起動ツール」にはしない。

以下をコア機能とする。

### ① Multi-Agent

複数AIを同時利用する。

### ② Isolation

AIごとにContainerを分離する。

### ③ Orchestration

ManagerがAIを管理する。

### ④ Specialization

AIごとに専門分野を与える。

### ⑤ Collaboration

AI同士が結果を共有する。

### ⑥ Verification

別AIがコードを検証する。

### ⑦ Self-Repair

失敗したコードをAIが修正する。

### ⑧ Environment Diagnosis

envdoctorなどと連携して環境問題を診断する。

### ⑨ Security

CVE・依存関係・静的解析を行う。

### ⑩ Autonomous Development

最終的に、

> 「このソフトウェアを作ってください」

という1つの指示から、

```text
設計
→ 実装
→ テスト
→ レビュー
→ セキュリティ
→ 修正
→ 再テスト
→ 統合
```

まで自動的に進められるようにする。

---

# 63. 開発時の重要原則

## 原則1

**最初から完全自動化しない。**

まず、

```text
Manager → Worker → Result
```

を完成させる。

---

## 原則2

**AIよりも実行基盤を先に安定させる。**

Container、API、Git、ログが安定してからAIを組み込む。

---

## 原則3

**Agentに無制限の権限を与えない。**

Containerと権限で被害範囲を限定する。

---

## 原則4

**Agentの出力を必ず構造化する。**

自然言語だけでAgent間通信を行わない。

---

## 原則5

**すべての作業をログに残す。**

後から、

```text
誰が
いつ
何を
なぜ
実行したか
```

を追跡できるようにする。

---

# 64. 開発ロードマップ

```text
Phase 1
環境構築
        ↓
Phase 2
Worker
        ↓
Phase 3
Manager
        ↓
Phase 4
Git
        ↓
Phase 5
Task Manager
        ↓
Phase 6
AI Task Decomposition
        ↓
Phase 7
Testing
        ↓
Phase 8
Review
        ↓
Phase 9
Security
        ↓
Phase 10
Self-Repair
        ↓
Phase 11
Autonomous Development
        ↓
Version 1.0
```

---

# 65. Version 1.0の完成条件

以下を満たした時点でVersion 1.0とする。

* [ ] Ubuntu 26.04上でManagerが動作する
* [ ] Ubuntu 24.04上でWorker環境が動作する
* [ ] 5つのContainerを起動できる
* [ ] A〜EのAgentを識別できる
* [ ] Managerからタスクを送信できる
* [ ] Workerから結果を取得できる
* [ ] Git branchを自動作成できる
* [ ] Agentごとに作業領域を分離できる
* [ ] Buildを自動実行できる
* [ ] Testを自動実行できる
* [ ] Reviewを自動実行できる
* [ ] Security Checkを実行できる
* [ ] エラー時に再試行できる
* [ ] 最大試行回数を設定できる
* [ ] Agentのログを保存できる
* [ ] Managerがタスク結果を評価できる
* [ ] 成功した変更を統合できる
* [ ] 失敗時にユーザーへ報告できる
* [ ] `mag status` などで全Agentの状態を確認できる

---

# 66. 最終目標

このプロジェクトの最終目標は、

```text
人間
 │
 │ 「○○というソフトウェアを作って」
 ▼
Manager Agent
 │
 ├── 設計
 ├── タスク分解
 ├── Agent割り当て
 │
 ├──────── Developer
 │
 ├──────── Tester
 │
 ├──────── Reviewer
 │
 ├──────── Security
 │
 └──────── Researcher
              │
              ▼
          結果収集
              │
              ▼
          問題分析
              │
              ▼
            修正
              │
              ▼
            再テスト
              │
              ▼
          コード統合
              │
              ▼
        完成プロジェクト
```

という、**複数のAIエージェントが隔離された実行環境の中で協調してソフトウェアを開発するプラットフォーム**を構築することである。

単なる「複数Antigravity起動ツール」ではなく、

> **Multi-Agent Software Development Orchestrator**

として設計する。
