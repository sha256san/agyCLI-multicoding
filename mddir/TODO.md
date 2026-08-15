# Multi-Agent Development Orchestrator 実装タスク一覧 (TODO.md)

本ドキュメントは、プロジェクト「`mag` (Multi-Agent Development Orchestrator)」の未実装機能および開発ロードマップの進捗を管理します。

---

## 🎯 優先目標: MVP (Minimum Viable Product) 【完了】

> **MVP目標**: 複数Agent構成（Manager + Developer A + Tester B + Reviewer C + Security D + Researcher E）で、タスク送信から実装、テスト、レビュー、セキュリティ検証、マージまでの自動サイクルを実証する。

- [x] **1. MVP基盤環境**
  - [x] Manager - Worker 間の通信疎通（HTTP + JSON REST API）
  - [x] 5コンテナ/プロセスの起動・停止スクリプト整備（Agent A〜E）
  - [x] 基本設定ファイル（`project/project.yaml`, 各Agent定義YAML: `developer.yaml`, `tester.yaml`, `reviewer.yaml`, `security.yaml`, `researcher.yaml`）
- [x] **2. Worker API 実装**
  - [x] `POST /task` (タスク受付 & プロセス非同期起動)
  - [x] `GET /status` (実行中ステータス返却)
  - [x] `GET /result` (構造化JSON結果の返却)
  - [x] `POST /cancel` (タスク強制終了)
  - [x] `GET /health` (ヘルスチェック)
- [x] **3. Manager コア機能**
  - [x] タスクディスパッチャ（各専門Workerへのタスク送信）
  - [x] ポーリング/コールバックによる結果収集
  - [x] テスト結果・レビュー結果・セキュリティ結果に基づく合否判定 & 自己修復ループ（ResultEvaluator / MAX_RETRY=3）
  - [x] 永続化・ログ保存機構（SQLite: `DatabaseManager`, `logs/database.sqlite`）
- [x] **4. Git自動連携**
  - [x] タスクごとのブランチ作成 (`agent-a/task-xxx`)
  - [x] コミットハッシュの回収と判定後のマージ支援 (`GitManager`)
- [x] **5. CLI基本コマンド (`mag`)**
  - [x] `mag init` (プロジェクト初期化)
  - [x] `mag status` (各Agentおよびタスクの状態表示)
  - [x] `mag doctor` (EnvDoctor環境診断)
  - [x] `mag run "<指示>"` (自然言語プロンプトからの自動タスクDAG分解・自律実行ループ)
  - [x] `mag task list` / `mag task show <id>` (タスク一覧・詳細確認)
  - [x] `mag logs` (ログ確認)

---

## 📋 フェーズ別詳細ロードマップ

### Phase 1: 開発環境 & インフラ基盤構築 【完了】
- [x] Ubuntu 26.04 (Manager) と Ubuntu 24.04 (Workerホスト) 間の通信設計 (HTTP + JSON)
- [x] Worker用ベースDockerfileの作成 (`containers/*/Dockerfile`)
- [x] 各Agent用専用ユーザー定義 (`account A〜E`)
- [x] Docker Compose による5コンテナ定義 (`docker-compose.yml`)
- [x] エージェント設定ファイル群 (`project/agents/*.yaml`)

### Phase 2: Worker Agent サービス実装 【完了】
- [x] 軽量Worker REST APIデーモン実装 (`project/src/worker/server.py`)
- [x] サブプロセス実行エンジン (`project/src/worker/executor.py`)
- [x] コマンドホワイトリスト制御機構 (`ROLE_COMMAND_ALLOWLIST` & 危険コマンド検知)
- [x] 構造化出力パーサー (`TaskResult`, JSONスキーマ正規化)
- [x] 各ロール（Developer, Tester, Reviewer, Security, Researcher）ハンドラー (`project/src/worker/agent_logic.py`)

### Phase 3: Manager コアアーキテクチャ実装 【完了】
- [x] **Task Manager**: タスクキュー、依存関係グラフ（DAG）管理 (`project/src/manager/task_manager.py`)
- [x] **Worker Manager / Client**: Workerの生存監視、ヘルスチェック、死活復旧 (`project/src/manager/worker_client.py`)
- [x] **Result Evaluator**: WorkerからのJSON出力を評価し次アクションを決定 (`project/src/manager/evaluator.py`)
- [x] **Log Manager / SQLite データストア**: tasks, agents, logs テーブルの設計・マイグレーション (`project/src/manager/db.py`)

### Phase 4: Git ワークスペース & Worktree 連携 【完了】
- [x] Git Worktree による各Agent用作業ディレクトリの生成
- [x] Agent用ブランチの自動作成・チェックアウト
- [x] コミットメッセージの自動生成および差分（diff）抽出
- [x] 承認済みブランチの `main` への自動マージ & 競合検知 (`GitManager`)

### Phase 5: タスク状態マシン & 安全装置 【完了】
- [x] 状態遷移マシン実装 (`PENDING` → `ASSIGNED` → `RUNNING` → `REVIEW` → `TESTING` → `COMPLETED`)
- [x] 最大リトライ上限制御 (`MAX_RETRY = 3`)
- [x] タスクタイムアウト制御 (`MAX_EXECUTION_TIME = 300`)
- [x] 危険コマンド（ファイル一括削除等）の自動拒否ガードレール

### Phase 6: AI タスク分解エンジン 【完了】
- [x] 自然言語要件からタスクDAGへの自動分解 (`Orchestrator.decompose_requirement`)
- [x] タスク間の入力/出力依存関係の自動解決
- [x] 連続タスクID採番（`TASK-001`〜）

### Phase 7: 自動テスト & フィードバックループ 【完了】
- [x] Tester Agent (Agent B) のテスト実行エンジン
- [x] `unittest`, `pytest`, `cargo test` 等の言語別テストランナー対応
- [x] テスト失敗時のスタックトレース抽出 & Developer Agentへの修正指示自動生成
- [x] 最大3回までの修正・再テスト自動サイクル

### Phase 8: コードレビュー自動化 【完了】
- [x] Reviewer Agent (Agent C) による静的コード解析 & 構文レビュー
- [x] レビュー観点（正確性、保守性、設計、パフォーマンス）の検証
- [x] レビュー指摘事項のJSON形式出力とDeveloperへの修正要求フロー

### Phase 9: Security Agent & 5-Agent フル構成 【完了】
- [x] Security Agent (Agent D) の実装
- [x] ハードコードされた秘密情報（APIキー・秘密鍵等）の検出スキャン
- [x] Research / Documentation Agent (Agent E) の実装（仕様調査、ドキュメント自動生成）

### Phase 10: 自己修復 & 環境エラー自動診断 【完了】
- [x] **envdoctor** 連携: 環境起因エラー（PATH, コンパイラ, Docker等）の自動切り分け (`EnvDoctor`)
- [x] **jpcargo** 連携: Rustエラーメッセージの日本語解析・修正アドバイス (`JpCargoAnalyzer`)

### Phase 11: 自律開発 & Version 1.0 リリース 【進行中】
- [x] 統合CLIツール (`mag`) の完成
- [x] ユーザーの1指示からの完全自動開発（設計→実装→テスト→レビュー→セキュリティ→統合）の実証
- [x] ユニット & 統合テストスイート（14件のテスト全PASS）
- [ ] 物理コンテナ実運用デプロイの検証

---

## 🏁 Version 1.0 リリース完了条件 (Acceptance Criteria)

- [x] Ubuntu 26.04 上で Manager が安定動作する
- [x] 5つのWorkerコンテナ/プロセス（Agent A〜E）が安定動作する
- [x] Manager から Worker へタスクを送信し、構造化結果を取得できる
- [x] Gitブランチが自動作成され、Agentごとに作業領域が分離されている
- [x] ビルド、テスト、レビュー、セキュリティ診断が自動実行される
- [x] エラー発生時に最大リトライ回数内で自動修正・再試行が行われる
- [x] すべてのAgentの活動ログがSQLiteに追跡可能に記録される
- [x] `mag status` でシステム全体のリアルタイム状況が確認できる
- [x] 成功したタスク成果物が安全に `main` ブランチに統合される
