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

---

## 🎯 認証・可変スケーリング・コード生成ロードマップ (`mddir/addplan2.md` 準拠) 【完了】

- [x] **Google OAuth2 認証 & セッション管理**
  - [x] `mag login google`: Device Authorization Flow による安全な認証
  - [x] `mag whoami`: 認証中アカウント情報の確認
  - [x] `mag logout`: クレデンシャルの安全な消去
- [x] **コンテナ・ワーカー数の動的スケーリング**
  - [x] `mag scale --workers <N>`: ワーカー数の動的変更
  - [x] `WorkerPoolManager` によるオンデマンドコンテナ管理
- [x] **自然言語指示からの特定パス実コード自動生成**
  - [x] プロンプトからの対象ディレクトリ自動抽出（例: `/home/guru/agytest`）
  - [x] `Cargo.toml`, `src/main.rs`, `docs/spec.md` の実ファイル生成 & テスト自動実行

---

## 🎯 コンテナ内 agycli 統合 & リリースロードマップ (`mddir/addplan3.md` 準拠) 【完了】

- [x] **コンテナ内への `agycli` 自動インストール**
  - [x] 全 Worker Dockerfile (`containers/*/Dockerfile`) に `agycli` / `mag` バイナリをコピー・実行権限付与
- [x] **コンテナ内 `agycli` の自動ログイン連携**
  - [x] `docker-compose.yml` でホストの `.mag/credentials.json` を共有マウント
  - [x] `find_project_root` にて `/workspace/.mag` および `$HOME/.mag` を自動探索
- [x] **GitHub リリース v0.2.0**
  - [x] 全ソースコードのコミット
  - [x] Git タグ `v0.2.0` 作成
  - [x] `git@github.com:sha256san/agyCLI-multicoding.git` へのプッシュ

---

## 🎯 コンテナ個別ログイン・協調キュー・v0.2.1 ロードマップ (`mddir/addplan4.md` 準拠) 【完了】

- [x] **コンテナ個別ログイン & 永続化 (`agycli login <container>`)**
  - [x] `.mag/containers/<name>/credentials.json` による認証永続化
  - [x] コンテナ停止・再起動・再インストール後も認証維持（Zero Auth Loss）
- [x] **動的マルチロール協調キュー (Work-Stealing)**
  - [x] 少数ワーカー（例: 2台）でのマルチロール兼任 & 自律タスク取得
  - [x] Git Worktree による並列競合防止
- [x] **GitHub リリース v0.2.1**
  - [x] 自動 `main` マージ
  - [x] Git タグ `v0.2.1` 作成・プッシュ

---

## 🎯 対話型 REPL / TUI ターミナル & スラッシュコマンド (`mddir/addplan5.md` 準拠) 【完了】

- [x] **`agy` 互換の対話型ターミナル REPL / TUI モード**
  - [x] 引数なし実行時の対話型プロンプト（`agycli ❯ `）起動
  - [x] リッチなバナーと認証・ワーカー稼働状態ヘッダー
- [x] **AGY 標準スラッシュコマンド**
  - [x] `/help`, `/status`, `/doctor`, `/login`, `/whoami`, `/workers`, `/tasks`, `/clear`, `/exit`
- [x] **対話型自律マルチエージェント開発プロンプト実行**
  - [x] REPL内からのリアルタイムプロンプトディスパッチと `agycli ❯ ` への安全復帰

---

## 🎯 事前認証ログ (`agent.md`) & タスク詳細ログ (`task.md`) ロードマップ (`mddir/addplan6.md` 準拠) 【完了】

- [x] **事前ログイン & `agent.md` 自動同期**
  - [x] `agycli login <agent-name>` で `agy` 認証を行い `agent.md` を自動更新
  - [x] 待機エージェント数（STANDBY）を Manager に報告
- [x] **ログイン中エージェント限定の動的タスクディスパッチ**
  - [x] Manager Agent が `agent.md` の認証アカウントを自動検知してタスク割り当て
- [x] **`task.md` による詳細実行ログ & 自己修復レポート**
  - [x] 実行計画・各エージェントのログ・ファイル変更・テスト結果を `task.md` に自動出力
  - [x] 完了後の最終レポートを `task.md` に集約出力
