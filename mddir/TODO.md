# Multi-Agent Development Orchestrator 実装タスク一覧 (TODO.md)

本ドキュメントは、プロジェクト「`mag` (Multi-Agent Development Orchestrator)」の未実装機能および開発ロードマップの進捗を管理します。

---

## 🎯 Rustネイティブ実装マスターロードマップ (`mddir/addplan.md` 準拠) 【完了】

- [x] **01. Cargo Workspace 構成**
  - [x] ルート `Cargo.toml`, `rust-toolchain.toml` 定義
  - [x] 13 Crate の依存関係・ワークスペース設定
- [x] **02. コア基盤 Crates**
  - [x] `mag-common`: 基本型・状態マシン (`TaskStatus`, `AgentRole`, `TaskRequest`, `TaskResult`)
  - [x] `mag-config`: TOML設定ローダー・バリデーション (`ProjectConfig`, `AgentProfile`)
  - [x] `mag-task`: タスクデータモデル・ライフサイクル遷移・依存関係管理
  - [x] `mag-agent`: エージェント権限・Capabilities・コマンドホワイトリスト検証
  - [x] `mag-logging`: 構造化JSONログ・イベント追跡
  - [x] `mag-storage`: SQLite 永続化 (`rusqlite`, tasks, logs テーブル)
- [x] **03. 実行・通信・連携 Crates**
  - [x] `mag-git`: Git 初期化、Worktree 分離、コミット、差分抽出、マージ
  - [x] `mag-container`: Docker CLI ラッパー、コンテナ起動/停止、健全性監視
  - [x] `mag-api`: HTTP REST クライアント (`reqwest` with `rustls-tls`)
  - [x] `mag-worker`: コマンド実行エンジン (`CommandExecutor`) & 専門ロール別ハンドラー (`Developer`, `Tester`, `Reviewer`, `Security`, `Researcher`)
- [x] **04. スケジューラ & Manager Crates**
  - [x] `mag-scheduler`: DAG依存関係解決、並列実行可能タスク抽出
  - [x] `mag-manager`: オーケストレーションエンジン (`Orchestrator`)、結果評価 & 自己修復ループ (`ResultEvaluator`, MAX_RETRY=3)、診断ツール (`EnvDoctor`, `JpCargoAnalyzer`)
- [x] **05. 統合 CLI (`mag-cli`)**
  - [x] `mag init <project>`: プロジェクト初期化
  - [x] `mag status`: エージェント & タスク状態表示
  - [x] `mag doctor`: EnvDoctor システム診断
  - [x] `mag task list` / `mag task show <id>`: タスク詳細表示
  - [x] `mag "<prompt>"`: 自然言語プロンプトからの 5-Agent 自律開発ループ実行
- [x] **06. エージェント設定ファイル群 (TOML)**
  - [x] `agents/developer.toml`, `agents/tester.toml`, `agents/reviewer.toml`, `agents/security.toml`, `agents/researcher.toml`
- [x] **07. テスト自動化**
  - [x] `cargo test --workspace` による全13 crateの単体・結合テスト（100% PASS）
