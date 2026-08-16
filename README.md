# Multi-Agent Development Platform (`mag` / `agycli`) v0.3.0

> **AIを増やすのではなく、AIを組織化する。**  
> ターミナルを閉じてもAIが自律的に開発を継続し、再接続すると進捗・ログ・結果を確認できるRust製次世代Multi-Agent Development Platform。

---

## 📖 概要

**Multi-Agent Development Platform (`mag` / `agycli`)** は、単一のAIに依存するのではなく、**Manager（統括）**、**Developer（実装）**、**Tester（テスト）**、**Reviewer（レビュー）**、**Security（セキュリティ）**、**Researcher（調査）** の専門エージェントを組織化し、ソフトウェア開発ライフサイクル（要件分析→タスクDAG分解→実装→テスト→レビュー→セキュリティ診断→自己修復→Gitマージ）を自律実行するシステムです。

Rust製13 Crateワークスペースにより、高い信頼性、メモリ安全性、高速な並列実行基盤、**`agy` ネイティブの対話型ターミナル REPL（TUI）**、**バックグラウンド常駐デーモン & Detached 実行**、および **`attach` によるリアルタイム進捗復元** を提供します。

```text
                    User Terminal (agycli)
                              │
              ┌───────────────┴───────────────┐
              │ agycli run --detach "<prompt>"│
              │ agycli attach <task-id>       │
              │ agycli task list / status     │
              │ agycli logs <task-id>         │
              │ agycli daemon [start|status]  │
              └───────────────┬───────────────┘
                              │ (Attach / Detach / Query)
                              ▼
         ┌────────────────────────────────────────┐
         │       Persistent Manager Daemon        │
         │                                        │
         │  • Task Manager (State machine)        │
         │  • Session Manager (Attach/Detach)     │
         │  • Agent Scheduler (Dynamic DAG/Queue) │
         │  • Event Store (Structured Event Bus)  │
         │  • Crash Recovery & Heartbeat Engine   │
         └───────────────────┬────────────────────┘
                             │
             ┌───────────────┼───────────────┐
             │ (SQLite DB: .mag/mag.db)      │
             ▼                               ▼
   ┌───────────────────┐           ┌───────────────────┐
   │ Persistent SQLite │           │ Authenticated     │
   │  • tasks          │           │ Multi-Agent Pool  │
   │  • sessions       │           │  • Researcher     │
   │  • agents         │           │  • Developer      │
   │  • events         │           │  • Tester         │
   │  • logs           │           │  • Reviewer       │
   └───────────────────┘           │  • Security       │
                                   └───────────────────┘
```

---

## ✨ 主な特徴

- 🚀 **ターミナル切断永続化 (Detached Execution)**: `agycli run --detach` でタスクを開始すると、ターミナルを閉じたり PC を切断してもAIがバックグラウンドで開発を継続。
- 🔌 **ライブセッション再接続 (`agycli attach`)**: いつでもタスクへ再接続し、各ロール（Researcher, Developer, Tester, Reviewer, Security）のASCII進捗バーとリアルタイムログを確認可能。Ctrl+C でいつでも安全に detach 可能。
- 📜 **構造化イベントストア (Event Log & Timeline)**: `events` テーブルに全ライフサイクルイベント（生成、割当、実装差分、テスト合否、レビュー、セキュリティ診断）を完全記録。`agycli logs <task-id>` で時系列タイムラインを表示。
- ⚙️ **常駐 Manager Daemon (`agycli daemon`)**: PID監視、稼働時間計測、アクティブタスク追跡、およびクラッシュ時の一括自動復旧（Crash Recovery）を標準装備。
- 🏢 **組織化された役割分担**: 5つの特化ロールをDAG（有向非巡回グラフ）で連携。
- 🦀 **Rust-Native 13 Crate 構成**: 高速・安全・モジュール分離されたCargo Workspace設計。
- 🖥️ **`agy` 互換の対話型ターミナル REPL**: 引数なしで起動すると、常駐プロンプト `agycli ❯ ` が起動し、チャット感覚でスラッシュコマンドや開発指示を実行可能。
- 📋 **`agent.md` による事前認証 & アカウント管理**: `agycli login <agent-name>` でログインしたアカウントをエージェントに紐付け、待機状態を `agent.md` に常時記録。Manager はログイン中エージェントへタスクを自動割り振り。
- 📝 **`task.md` による詳細実行記録 & 自己修復レポート**: タスクDAG計画、各エージェントのログ、変更ファイル、テスト結果、自己修復履歴をリアルタイムで `task.md` に出力。
- 🔑 **コンテナ個別ログイン & 認証永続化**: コンテナ停止・再起動・再インストール後も認証情報を自動維持（Zero Auth Loss）。
- 🤝 **動的マルチロール & 協調型タスクキュー**: ワーカー数が少数（例: 2台）でも、複数ロールを柔軟に兼任し、空きワーカーがタスクを自律取得（Work-Stealing）。
- 🔄 **自己修復 (Self-Repair) & 自動 `main` マージ**: テスト・レビュー失敗時の自動修正（最大3回）と、全パス時の自動ブランチマージ。

---

## 👥 エージェント一覧

| エージェント | ロール | 担当内容 |
|---|---|---|
| **Manager** | 全体統括 & Daemon | 要件分析、`agent.md` 確認、タスクDAG分解・割り振り、品質評価、Gitマージ、Daemon管理 |
| **Agent A (Developer)** | 実装 | ソースコード作成、バグ修正、リファクタリング、コミット |
| **Agent B (Tester)** | テスト | 単体・結合テスト、ビルド検証、回帰テスト実行 |
| **Agent C (Reviewer)** | レビュー | 静的解析、可読性、保守性、設計妥当性、パフォーマンス検証 |
| **Agent D (Security)** | セキュリティ | 脆弱性診断 (CVE/OSV)、依存関係スキャン、シークレット検出 |
| **Agent E (Researcher)** | 調査 & 文書 | 技術仕様調査、設計書作成、README/CHANGELOG保守 |

---

## 🚀 クイックスタート & 使い方

### ビルド & インストール
```bash
cargo build --workspace --release
cargo install --path crates/mag-cli
```

---

### 💻 1. 対話型 REPL ターミナルモード（推奨）

引数を付けずに `agycli` を実行すると、本来の `agy` と同じリッチな対話型ターミナルが起動します。

```bash
$ agycli
```

```text
      _          ___ _     ___ _ 
     /_\  __ _ _/ __| |   |_ _| |
    / _ \/ _` | | (__| |__ | || |
   /_/ \_\__, |_|\___|____|___|_|
         |___/                   
 Multi-Agent Development Platform (`agycli` - Rust Native v0.3.0)
    
 📂 Workspace:  /home/guru/agyCLI++
 👤 User:       developer@google.com (Google Developer) [google]
 🤖 Workers:    4 active authenticated agents (in agent.md)
 ⚡ Mode:       Interactive REPL & Detachable Daemon  |  Type /help for commands

agycli ❯ 
```

#### ⚡ スラッシュコマンド一覧
| コマンド | 動作内容 |
|---|---|
| `/help` | 利用可能なスラッシュコマンド一覧と使い方の表示 |
| `/status` | オーケストレーター、エージェント、デーモン、タスク状況一覧 |
| `/doctor` | `EnvDoctor` システム環境・ツール診断の実行 |
| `/login [target]` | Google 認証および特定エージェント（`agent-a` など）のブラウザ認証ログイン（`agent.md` 自動同期） |
| `/whoami [cnt]` | ログイン中のユーザー情報・コンテナ認証情報の確認 |
| `/workers [N]` | ワーカーコンテナ数の動的スケーリング（例: `/workers 4`） |
| `/tasks` | 最近のタスク履歴・ステータス一覧の表示 |
| `/attach [id]` | バックグラウンドタスクへの再接続・進捗確認 |
| `/logs [id]` | タスクのイベントタイムライン表示 |
| `/daemon` | Manager Daemon の稼働ステータス確認 |
| `/clear` | ターミナル画面クリア & ヘッダー再描画 |
| `/exit` / `/quit` | 対話型セッションの終了 |
| `<自然言語指示>` | プロンプトを入力すると、5-Agent DAG 自律開発ループを実行し `task.md` に全ログを出力 |

---

### ⌨️ 2. バックグラウンド自律開発 & Attach / Detach ワークフロー

#### ① バックグラウンド（Detached）で開発タスクを開始
```bash
$ agycli run --detach "/home/guru/agytest に高速な数値計算モジュールを実装して"

======================================================================
 [✓] Task started in DETACHED background mode!
 Task ID: TASK-001
 Status:  RUNNING

 Detach safely. Reconnect anytime with:
   agycli attach TASK-001
   agycli logs TASK-001
======================================================================
```
> ※ ここでターミナルを閉じても、AIエージェント達は自律的に開発を継続します。

#### ② タスク一覧の確認
```bash
$ agycli task list

TASK ID      STATUS         ROLE         AGENT        RETRY  TITLE                         
------------------------------------------------------------------------------------------
TASK-001     COMPLETED      researcher   agent-a      0      Spec: /home/guru/agytest に高速な数値計算モジュールを実装して
TASK-002     COMPLETED      developer    agent-c      0      Implementation: /home/guru/agytest に高速な数値計算モジュールを実装して
TASK-003     COMPLETED      tester       cnt-a        0      Testing: /home/guru/agytest に高速な数値計算モジュールを実装して
TASK-004     COMPLETED      reviewer     agent-b      0      Review: /home/guru/agytest に高速な数値計算モジュールを実装して
TASK-005     COMPLETED      security     agent-a      0      Security: /home/guru/agytest に高速な数値計算モジュールを実装して
```

#### ③ タスクへの再接続（Attach）
```bash
$ agycli attach TASK-001

======================================================================
 Attached Session: Task [TASK-001]
 Status:           COMPLETED
 Progress:         100% [████████████]
 Current Step:     All tasks completed
======================================================================

Agent Role Stages Breakdown:
  researcher   [████████████] 100% (agent-a) -> COMPLETED
  developer    [████████████] 100% (agent-c) -> COMPLETED
  tester       [████████████] 100% (cnt-a)   -> COMPLETED
  reviewer     [████████████] 100% (agent-b) -> COMPLETED
  security     [████████████] 100% (agent-a) -> COMPLETED

Recent Event Logs:
  • [12:12:47] AGENT_ASSIGNED     | "Assigned to agent-a for role researcher"
  • [12:12:54] AGENT_STARTED      | "Agent agent-a started role researcher"
  • [12:12:54] CODE_CHANGED       | ["docs/spec.md"]
  • [12:12:54] AGENT_FINISHED     | "Researcher 'agent-e' completed specification and research for task 'TASK-001'."
  • [12:12:56] TASK_COMPLETED     | "Completed and verified"

(Detach safely with Ctrl+C. AI will continue running in background.)
(Type `agycli attach TASK-001` to reconnect anytime)
```

#### ④ イベントログ・タイムラインの確認
```bash
$ agycli logs TASK-001

Event Log Timeline for Task [TASK-001]:
----------------------------------------------------------------------
[12:12:47] TASK_CREATED       | Agent: agent-a    | "Spec: /home/guru/agytest に高速な数値計算モジュールを実装して"
[12:12:47] AGENT_ASSIGNED     | Agent: agent-a    | "Assigned to agent-a for role researcher"
[12:12:54] AGENT_STARTED      | Agent: agent-a    | "Agent agent-a started role researcher"
[12:12:54] CODE_CHANGED       | Agent: agent-a    | ["docs/spec.md"]
[12:12:54] AGENT_FINISHED     | Agent: agent-a    | "Researcher 'agent-e' completed specification and research for task 'TASK-001'."
[12:12:56] TASK_COMPLETED     | Agent: manager    | "Completed and verified"
----------------------------------------------------------------------
```

#### ⑤ コンテナ稼働状態 & 認証ユーザー確認
```bash
$ agycli containers

Active & Configured Agent Containers:
CONTAINER      ROLE           STATUS                 ACCOUNT (EMAIL)                  IMAGE                    
-----------------------------------------------------------------------------------------------------------
agent-a        developer      [●] READY / STANDBY    user-agent-a@google.com          agycli-developer:latest  
agent-b        tester         [●] READY / STANDBY    user-agent-b@google.com          agycli-tester:latest     
agent-c        reviewer       [●] READY / STANDBY    user-agent-c@google.com          agycli-reviewer:latest   
agent-d        security       [○] STOPPED            - (Not Logged In)                agycli-security:latest   
agent-e        researcher     [○] STOPPED            - (Not Logged In)                agycli-researcher:latest 
cnt-a          collaborative  [●] READY / STANDBY    user-cnt-a@google.com            agycli-worker:latest     
```

#### ⑥ 常駐 Daemon 管理
```bash
agycli daemon status          # デーモン稼働状況・PID・アクティブタスク数
agycli daemon start           # デーモン起動 & クラッシュリカバリ
agycli daemon stop            # デーモン停止
agycli daemon restart         # デーモン再起動
```

---

## 📂 プロジェクト構成

```text
agyCLI++/
├── Cargo.toml                  # Root Cargo Workspace (v0.3.0)
├── rust-toolchain.toml         # ツールチェーン固定
├── mag                         # 統合CLIランチャー
├── docker-compose.yml          # コンテナ構成 & 認証ボリューム共有
├── agent.md                    # ログイン中エージェント & アカウント一覧 (自動同期)
├── task.md                     # 実行タスクDAG・リアルタイムログ・検証レポート (自動生成)
│
├── crates/                     # Rust 13-Crate コア実装
│   ├── mag-common/             # 共通型・Enum (AgentRole, TaskStatus, TaskResult, AuthConfig)
│   ├── mag-config/             # TOML設定ローダー & agent.md 自動同期管理
│   ├── mag-task/               # タスクモデル・状態マシン・DAG依存関係
│   ├── mag-agent/              # エージェント定義・Capabilities・コマンド許可ポリシー
│   ├── mag-logging/            # 構造化JSONロギング
│   ├── mag-storage/            # SQLite 永続化 (tasks, events, sessions, agents)
│   ├── mag-git/                # Git Worktree / ブランチ / コミット / マージ管理
│   ├── mag-container/          # Docker CLI 連携 & コンテナ内コマンド実行 & プールスケーリング
│   ├── mag-api/                # HTTP REST & Google OAuth2 Device Flow クライアント
│   ├── mag-worker/             # 実行エンジン (CommandExecutor) & ロール別ハンドラー
│   ├── mag-scheduler/          # タスクDAGスケジューラ & Work-Stealing 協調キュー
│   ├── mag-manager/            # Manager オーケストレーター, Daemon, Session (Attach/Detach), Crash Recovery
│   └── mag-cli/                # 統合CLIバイナリ (`mag` & `agycli`, Detached, Attach, Logs, REPL)
│
├── agents/                     # TOMLエージェント定義 (developer, tester, reviewer, security, researcher)
├── containers/                 # Dockerfile定義 (agycli プリインストール済みコンテナ)
│
├── mddir/                      # プロジェクトドキュメント体系
│   ├── addbigplan.md           # ターミナル切断永続化・Attach/Detach・Daemon・Event Log 全体仕様書
│   ├── addplan6.md             # agent.md事前認証・ログイン中割当・task.md詳細ログ仕様書
│   ├── addplan5.md             # 対話型 REPL / TUI & スラッシュコマンド仕様書
│   ├── addplan4.md             # コンテナログイン・再認証永続化・協調キュー・自動マージ仕様書
│   ├── addplan3.md             # コンテナ内 agycli 統合 & v0.2.0 仕様書
│   ├── addplan2.md             # Google認証・可変スケーリング・コード生成仕様書
│   ├── addplan.md              # Rust 13-Crate 実装マスター計画書
│   ├── SPEC.md                 # 要件・仕様定義書
│   ├── TODO.md                 # 実装進捗ロードマップ
│   ├── MEMORY.md               # 知識ベース・ADR・5大開発原則
│   ├── AGENTS.md               # エージェント行動規範 & JSON出力スキーマ
│   ├── CHANGELOG.md            # 変更履歴 (v0.3.0)
│   └── plan.md                 # 全体構想書
│
└── project/                    # 実装コードベース & プロトタイプ
    ├── project.yaml
    └── src/
```

---

## 📚 関連ドキュメント

- 🚀 [全体拡張仕様書 (addbigplan.md)](file:///home/guru/agyCLI++/mddir/addbigplan.md)
- 📋 [第6次拡張仕様書 (addplan6.md)](file:///home/guru/agyCLI++/mddir/addplan6.md)
- 🖥️ [第5次拡張仕様書 (addplan5.md)](file:///home/guru/agyCLI++/mddir/addplan5.md)
- 🚀 [第4次拡張仕様書 (addplan4.md)](file:///home/guru/agyCLI++/mddir/addplan4.md)
- 📦 [第3次拡張仕様書 (addplan3.md)](file:///home/guru/agyCLI++/mddir/addplan3.md)
- 🔑 [第2次拡張仕様書 (addplan2.md)](file:///home/guru/agyCLI++/mddir/addplan2.md)
- 🦀 [Rust実装マスター計画書 (addplan.md)](file:///home/guru/agyCLI++/mddir/addplan.md)
- 📘 [仕様書 (SPEC.md)](file:///home/guru/agyCLI++/mddir/SPEC.md)
- 📝 [実装計画 & ロードマップ (TODO.md)](file:///home/guru/agyCLI++/mddir/TODO.md)
- 🧠 [知識ベース & 設計原則 (MEMORY.md)](file:///home/guru/agyCLI++/mddir/MEMORY.md)
- 🤖 [エージェント行動規範 (AGENTS.md)](file:///home/guru/agyCLI++/mddir/AGENTS.md)
- 📜 [変更履歴 (CHANGELOG.md)](file:///home/guru/agyCLI++/mddir/CHANGELOG.md)
- 🗺️ [全体構想書 (plan.md)](file:///home/guru/agyCLI++/mddir/plan.md)
