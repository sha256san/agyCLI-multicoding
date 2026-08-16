# Multi-Agent Development Orchestrator (`mag` / `agycli`) v0.2.2

> **AIを増やすのではなく、AIを組織化する。**  
> 複数のAIエージェントが独立したコンテナ環境で協調し、自律的にソフトウェアを開発・検証・統合するRust製次世代オーケストレーションプラットフォーム。

---

## 📖 概要

**Multi-Agent Development Orchestrator (`mag` / `agycli`)** は、単一のAIに依存するのではなく、**Manager（統括）**、**Developer（実装）**、**Tester（テスト）**、**Reviewer（レビュー）**、**Security（セキュリティ）**、**Researcher（調査）** の専門エージェントを組織化し、ソフトウェア開発ライフサイクル（要件分析→タスクDAG分解→実装→テスト→レビュー→セキュリティ診断→自己修復→Gitマージ）を自律実行するシステムです。

Rust製13 Crateワークスペースにより、高い信頼性、メモリ安全性、高速な並列実行基盤、および **`agy` ネイティブの対話型ターミナル REPL（TUI）** を提供します。

```text
                        Manager (Rust / Ubuntu 26.04)
                                      │
          ┌───────────────────────────┼───────────────────────────┐
          │ (Task DAG Decomposition)  │                           │
          ▼                           ▼                           ▼
      Researcher                  Developer                    Tester
      (Agent E)                   (Agent A)                  (Agent B)
          │                           │                           │
          └───────────────────────────┼───────────────────────────┘
                                      │
          ┌───────────────────────────┴───────────────────────────┐
          ▼                                                       ▼
       Reviewer                                                Security
      (Agent C)                                               (Agent D)
          │                                                       │
          └───────────────────────────┬───────────────────────────┘
                                      ▼
                         Manager (Evaluation & Merge)
                                      │
                              ┌───────┴───────┐
                              ▼               ▼
                         Self-Repair        Merge
                          (Retry <=3)      (Main)
```

---

## ✨ 主な特徴

- 🏢 **組織化された役割分担**: 5つの特化ロール（実装/テスト/レビュー/セキュリティ/調査）をDAG（有向非巡回グラフ）で連携。
- 🦀 **Rust-Native 13 Crate 構成**: 高速・安全・モジュール分離されたCargo Workspace設計。
- 🖥️ **`agy` 互換の対話型ターミナル REPL**: 引数なしで起動すると、常駐プロンプト `agycli ❯ ` が起動し、チャット感覚でスラッシュコマンドや開発指示を実行可能。
- 📦 **全コンテナへの `agycli` 自動インストール**: 各Workerコンテナ内に `agycli` がプリインストールされ、コンテナ内からも完全操作可能。
- 🔑 **コンテナ個別ログイン & 認証永続化**: `agycli login "agent-a"` で個別認証。コンテナ停止・再起動・再インストール後も認証情報を自動維持（Zero Auth Loss）。
- 🤝 **動的マルチロール & 協調型タスクキュー**: ワーカー数が少数（例: 2台）でも、複数ロールを柔軟に兼任し、空きワーカーがタスクを自律取得（Work-Stealing）。
- 🔄 **自己修復 (Self-Repair) & 自動 `main` マージ**: テスト・レビュー失敗時の自動修正（最大3回）と、全パス時の自動ブランチマージ。
- 🌳 **Git Worktree 分離**: Agent/Taskごとに専用作業領域を割り当て、並列実行時のファイル競合を物理的に防止。
- 🛡️ **厳格な安全装置**: コマンドホワイトリスト、危険コマンド検知、タイムアウト制限、人間確認ゲートを標準装備。
- 🧩 **診断ツール連携**: `envdoctor` (環境起因エラー診断) や `jpcargo` (Rust日本語エラー解析) との連携。

---

## 👥 エージェント一覧

| エージェント | ロール | 担当内容 |
|---|---|---|
| **Manager** | 全体統括 | 要件分析、タスクDAG分解、ディスパッチ、品質評価、Gitマージ |
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
> ※ `agycli` および `mag` が `~/.cargo/bin/` にインストールされ、任意のディレクトリから実行可能になります。

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
 Multi-Agent Development Orchestrator (`agycli` - Rust Native v0.2.2)
    
 📂 Workspace:  /home/guru/agyCLI++
 👤 User:       developer@google.com (Google Developer) [google]
 🤖 Workers:    5 active collaborative agents
 ⚡ Mode:       Interactive REPL  |  Type /help for commands

agycli ❯ 
```

#### ⚡ スラッシュコマンド一覧
| コマンド | 動作内容 |
|---|---|
| `/help` | 利用可能なスラッシュコマンド一覧と使い方の表示 |
| `/status` | エージェント・コンテナ認証・タスク稼働状況一覧 |
| `/doctor` | `EnvDoctor` システム環境・ツール診断の実行 |
| `/login [target]` | Google 認証および特定コンテナ（`agent-a` など）のブラウザ認証ログイン |
| `/whoami [cnt]` | ログイン中のユーザー情報・コンテナ認証情報の確認 |
| `/workers [N]` | ワーカーコンテナ数の動的スケーリング（例: `/workers 4`） |
| `/tasks` | 最近のタスク履歴・ステータス一覧の表示 |
| `/clear` | ターミナル画面クリア & ヘッダー再描画 |
| `/exit` / `/quit` | 対話型セッションの終了 |
| `<自然言語指示>` | プロンプトを入力すると、5-Agent DAG 自律開発ループをリアルタイムに実行 |

---

### ⌨️ 2. コマンドライン（CLI）直接実行モード

非対話型スクリプトやパイプラインからも、サブコマンド形式で直接実行できます。

```bash
# 1. 認証 (グローバル / コンテナ個別指定ログイン)
agycli login google           # グローバル Google OAuth2 ログイン
agycli login "agent-a"        # 特定コンテナ指定のブラウザ認証ログイン
agycli whoami                 # グローバル認証ユーザー情報の確認
agycli whoami agent-a         # 特定コンテナの認証状態確認
agycli logout                 # ログアウト

# 2. ワーカーコンテナ数の動的スケーリング (可変プール)
agycli scale --workers 8      # ワーカー数を動的に8台へスケール

# 3. システム環境診断 (EnvDoctor)
agycli doctor

# 4. プロジェクトの初期化 & ステータス確認
agycli init my-project
agycli status

# 5. 自然言語による開発タスク自律実行 (特定ディレクトリ指定 & 協調ワーカー数指定)
agycli run "/home/guru/agytest にrustのプログラムを書いて" --workers 2

# 6. タスク管理
agycli task list              # タスク一覧表示
agycli task show TASK-001     # 特定タスクの詳細と実行結果確認
```
> ※ `agycli` と `mag` コマンド（例: `mag status`）は完全に同等に使用可能です。

---

## 📂 プロジェクト構成

```text
agyCLI++/
├── Cargo.toml                  # Root Cargo Workspace (v0.2.2)
├── rust-toolchain.toml         # ツールチェーン固定
├── mag                         # 統合CLIランチャー
├── docker-compose.yml          # コンテナ構成 & 認証ボリューム共有
│
├── crates/                     # Rust 13-Crate コア実装
│   ├── mag-common/             # 共通型・Enum (AgentRole, TaskStatus, TaskResult, AuthConfig)
│   ├── mag-config/             # TOML設定ローダー & コンテナ認証永続化管理
│   ├── mag-task/               # タスクモデル・状態マシン・DAG依存関係
│   ├── mag-agent/              # エージェント定義・Capabilities・コマンド許可ポリシー
│   ├── mag-logging/            # 構造化JSONロギング
│   ├── mag-storage/            # SQLite 永続化 (rusqlite)
│   ├── mag-git/                # Git Worktree / ブランチ / コミット / マージ管理
│   ├── mag-container/          # Docker CLI 連携 & コンテナ内コマンド実行 & プールスケーリング
│   ├── mag-api/                # HTTP REST & Google OAuth2 Device Flow クライアント
│   ├── mag-worker/             # 実行エンジン (CommandExecutor) & ロール別ハンドラー
│   ├── mag-scheduler/          # タスクDAGスケジューラ & Work-Stealing 協調キュー
│   ├── mag-manager/            # Manager オーケストレーター & 自動 main マージ
│   └── mag-cli/                # 統合CLIバイナリ (`mag` & `agycli`, 対話型 REPL)
│
├── agents/                     # TOMLエージェント定義 (developer, tester, reviewer, security, researcher)
├── containers/                 # Dockerfile定義 (agycli プリインストール済みコンテナ)
│
├── mddir/                      # プロジェクトドキュメント体系
│   ├── addplan5.md             # 対話型 REPL / TUI & スラッシュコマンド仕様書
│   ├── addplan4.md             # コンテナログイン・再認証永続化・協調キュー・自動マージ仕様書
│   ├── addplan3.md             # コンテナ内 agycli 統合 & v0.2.0 仕様書
│   ├── addplan2.md             # Google認証・可変スケーリング・コード生成仕様書
│   ├── addplan.md              # Rust 13-Crate 実装マスター計画書
│   ├── SPEC.md                 # 要件・仕様定義書
│   ├── TODO.md                 # 実装進捗ロードマップ
│   ├── MEMORY.md               # 知識ベース・ADR・5大開発原則
│   ├── AGENTS.md               # エージェント行動規範 & JSON出力スキーマ
│   ├── CHANGELOG.md            # 変更履歴 (v0.2.2)
│   └── plan.md                 # 全体構想書
│
└── project/                    # 実装コードベース & プロトタイプ
    ├── project.yaml
    └── src/
```

---

## 📚 関連ドキュメント

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
