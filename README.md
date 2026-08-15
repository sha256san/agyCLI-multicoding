# Multi-Agent Development Orchestrator (`mag`)

> **AIを増やすのではなく、AIを組織化する。**  
> 複数のAIエージェントが独立したコンテナ環境で協調し、自律的にソフトウェアを開発・検証・統合するRust製次世代オーケストレーションプラットフォーム。

---

## 📖 概要

**Multi-Agent Development Orchestrator (`mag`)** は、単一のAIに頼るのではなく、**Manager（統括）**、**Developer（実装）**、**Tester（テスト）**、**Reviewer（レビュー）**、**Security（セキュリティ）**、**Researcher（調査）** の専門エージェントを組織化し、ソフトウェア開発ライフサイクル（要件分析→タスクDAG分解→実装→テスト→レビュー→セキュリティ診断→自己修復→Gitマージ）を自律実行するシステムです。

Rust製13 Crateワークスペースにより、高い信頼性、メモリ安全性、高速な実行基盤を提供します。

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
- 📦 **完全なコンテナ & 実行隔離**: Workerはコンテナ内で独立実行され、依存関係の衝突や環境汚染を防止。
- 🔄 **自己修復 (Self-Repair) ループ**: ビルド・テスト・レビュー失敗時に自動で原因を解析し、修正指示を再試行（最大3回）。
- 🌳 **Git Worktree 分離**: Agentごとに専用ブランチ・作業領域を割り当て、競合を物理的に回避。
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

## 🚀 クイックスタート & CLI

### ビルド
```bash
cargo build --workspace --release
```

### インストール
```bash
cargo install --path crates/mag-cli
```
> ※ `mag` および別名 `agycli` が `~/.cargo/bin/` にインストールされ、任意のディレクトリから実行可能になります。

### コマンド一覧
```bash
# 1. システム環境診断 (EnvDoctor)
mag doctor

# 2. プロジェクトの初期化
mag init my-project

# 3. 全Agentおよびタスクの稼働状態確認
mag status

# 4. 自然言語による開発タスク自律実行
mag "RustでCLIパーサーとJWT認証モジュールを実装してください"

# 5. タスク管理
mag task list           # タスク一覧表示
mag task show TASK-001  # 特定タスクの詳細と実行結果確認
```
> ※ `mag` の代わりに `agycli` コマンド（例: `agycli status`）も同等に使用可能です。

---

## 📂 プロジェクト構成

```text
agyCLI++/
├── Cargo.toml                  # Root Cargo Workspace
├── rust-toolchain.toml         # ツールチェーン固定
├── mag                         # 統合CLIランチャー
├── docker-compose.yml          # コンテナ構成
│
├── crates/                     # Rust 13-Crate コア実装
│   ├── mag-common/             # 共通型・Enum (AgentRole, TaskStatus, TaskResult)
│   ├── mag-config/             # TOML設定ローダー & バリデータ
│   ├── mag-task/               # タスクモデル・状態マシン・DAG依存関係
│   ├── mag-agent/              # エージェント定義・Capabilities・コマンド許可ポリシー
│   ├── mag-logging/            # 構造化JSONロギング
│   ├── mag-storage/            # SQLite 永続化 (rusqlite)
│   ├── mag-git/                # Git Worktree / ブランチ / コミット / マージ管理
│   ├── mag-container/          # Docker CLI 連携 & コンテナライフサイクル
│   ├── mag-api/                # HTTP REST クライアント (reqwest / rustls)
│   ├── mag-worker/             # 実行エンジン (CommandExecutor) & ロール別ハンドラー
│   ├── mag-scheduler/          # タスクDAGスケジューラ & 並列実行制御
│   ├── mag-manager/            # Manager オーケストレーター & 自己修復ループ
│   └── mag-cli/                # 統合CLIバイナリ (clap)
│
├── agents/                     # TOMLエージェント定義 (developer, tester, reviewer, security, researcher)
├── containers/                 # Dockerfile定義 (各ロール専用コンテナ)
│
├── mddir/                      # プロジェクトドキュメント体系
│   ├── addplan.md              # Rust実装マスター計画書
│   ├── SPEC.md                 # 要件・仕様定義書
│   ├── TODO.md                 # 実装進捗ロードマップ
│   ├── MEMORY.md               # 知識ベース・ADR・5大開発原則
│   ├── AGENTS.md               # エージェント行動規範 & JSON出力スキーマ
│   ├── CHANGELOG.md            # 変更履歴
│   └── plan.md                 # 全体構想書
│
└── project/                    # 実装コードベース & プロトタイプ
    ├── project.yaml
    └── src/
```

---

## 📚 関連ドキュメント

- 📘 [仕様書 (SPEC.md)](file:///home/guru/agyCLI++/mddir/SPEC.md)
- 🦀 [Rust実装マスター計画書 (addplan.md)](file:///home/guru/agyCLI++/mddir/addplan.md)
- 📝 [実装計画 & ロードマップ (TODO.md)](file:///home/guru/agyCLI++/mddir/TODO.md)
- 🧠 [知識ベース & 設計原則 (MEMORY.md)](file:///home/guru/agyCLI++/mddir/MEMORY.md)
- 🤖 [エージェント行動規範 (AGENTS.md)](file:///home/guru/agyCLI++/mddir/AGENTS.md)
- 📜 [変更履歴 (CHANGELOG.md)](file:///home/guru/agyCLI++/mddir/CHANGELOG.md)
- 🗺️ [全体構想書 (plan.md)](file:///home/guru/agyCLI++/mddir/plan.md)
