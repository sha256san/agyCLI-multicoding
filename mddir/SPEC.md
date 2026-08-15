# Multi-Agent Development Orchestrator 仕様書 (SPEC.md)

## 1. プロジェクト概要

### 1.1 目的
本プロジェクト（仮称: **`mag`** / **Multi-Agent Development Orchestrator**）は、複数のAIエージェントを独立したコンテナ環境で動作させ、1つの親エージェント（Manager）が統括・オーケストレーションを行うことで、**AI同士が組織的に役割分担しながらソフトウェアを自律開発・検証・統合するプラットフォーム**を構築する。

### 1.2 コアコンセプト
> **AIを増やすのではなく、AIを組織化する。**

単なるプロファイル管理ツール（Multigravity型）と異なり、開発（Developer）、テスト（Tester）、レビュー（Reviewer）、セキュリティ診断（Security）、調査・文書化（Researcher）といった役割分担と協調ループ、自己修復（Self-Repair）機能を提供する。

---

## 2. システムアーキテクチャ

### 2.1 全体構成
```text
Windows / Host
│
└── Ubuntu 26.04 (Host / Manager Node)
    │
    ├── Project Repository (Git main / Worktrees)
    │   ├── src/
    │   ├── tests/
    │   ├── docs/
    │   └── .git/
    │
    ├── Manager Agent (CLI / Daemon / Antigravity Orchestrator)
    │
    └── Communication Layer (HTTP / JSON REST API)
            │
            ▼
        Ubuntu 24.04 (Worker Host Node)
        │
        ├── account F (Worker Node Host Management)
        │   ├── Docker / Podman Daemon
        │   ├── Volume & Network Management
        │   └── Container Lifecycle
        │
        ├── Container 1 (account A): Developer Agent
        ├── Container 2 (account B): Tester Agent
        ├── Container 3 (account C): Reviewer Agent
        ├── Container 4 (account D): Security Agent
        └── Container 5 (account E): Research / Doc Agent
```

### 2.2 各環境・ノードの役割

| 環境 / アカウント | 役割 | 担当業務 |
|---|---|---|
| **Ubuntu 26.04** | プロジェクト統括 & Manager | Git管理、タスク管理、エージェント管理、結果収集・評価、最終判断、ログ管理 |
| **Ubuntu 24.04** | Agent実行基盤 | コンテナ実行環境、Workerプロセス実行、リソース分離、テスト実行基盤 |
| **account F** | Workerホスト管理 | Docker/Podman管理、コンテナ起動/停止/再起動、ネットワーク/Volume管理、ログ取得 |

---

## 3. エージェント定義 & 役割分担

```mermaid
graph TD
    User([ユーザー]) -->|自然言語指示 / コマンド| Manager[Manager Agent (Ubuntu 26.04)]
    Manager -->|タスク分解・ディスパッチ| Dev[Agent A: Developer]
    Manager -->|テスト指示| Test[Agent B: Tester]
    Manager -->|レビュー依頼| Rev[Agent C: Reviewer]
    Manager -->|セキュリティ診断| Sec[Agent D: Security]
    Manager -->|技術調査・文書化| Res[Agent E: Researcher]

    Dev -->|実装・コミット結果| Manager
    Test -->|テスト結果レポート| Manager
    Rev -->|レビュー結果レポート| Manager
    Sec -->|脆弱性診断レポート| Manager
    Res -->|調査・ドキュメント| Manager

    Manager -->|評価・判定| Eval{判定}
    Eval -->|不合格: 再指示| Dev
    Eval -->|合格: Git Merge| Complete([統合・成果物完成])
```

### 3.1 エージェント詳細

#### ① Manager Agent (親エージェント)
- **稼働環境**: Ubuntu 26.04
- **主な役割**: 要件分析、タスク分解（DAG生成）、Agent割り当て、結果評価、修正指示、Gitマージ、全体進捗監視。
- **制約**: 自ら大量の実装コードを書かず、ディスパッチと評価・統合に専念する。

#### ② Container 1 (account A) - Developer Agent
- **主な役割**: コード実装、バグ修正、リファクタリング、新機能開発。
- **権限**: 割り当てられたGit作業ブランチへのソース・テストコード書き込み、コミット作成。

#### ③ Container 2 (account B) - Tester Agent
- **主な役割**: ユニットテスト、結合テスト、ビルド検証、回帰テスト実行。
- **権限**: ソースコード読み取り、テスト実行、テスト結果ログ/レポートの出力。

#### ④ Container 3 (account C) - Reviewer Agent
- **主な役割**: 静的解析、コードレビュー（正確性・可読性・保守性・設計妥当性・パフォーマンス）。
- **権限**: ソースコード読み取り、レビューレポート出力（書き込み不可）。

#### ⑤ Container 4 (account D) - Security Agent
- **主な役割**: 脆弱性診断、依存関係チェック（CVE/OSV/cargo-audit/npm audit等）、秘密情報・危険コードの検出。
- **権限**: ソースコード読み取り、スキャナ実行、セキュリティレポート出力。

#### ⑥ Container 5 (account E) - Research / Documentation Agent
- **主な役割**: 技術調査、ライブラリAPI調査、設計ドキュメント作成、README・仕様書の保守。
- **権限**: ドキュメント領域への書き込み。

---

## 4. 通信仕様 & Worker API

Managerと各Worker Container間は、軽量な **HTTP + JSON REST API** で通信を行う。

### 4.1 エンドポイント一覧

| メソッド | パス | 説明 |
|---|---|---|
| `POST` | `/task` | ManagerからWorkerへタスク投入・実行開始 |
| `GET` | `/status` | Workerの現在の実行状態を取得 |
| `GET` | `/result` | タスク完了後の詳細実行結果を取得 |
| `POST` | `/cancel` | 実行中のタスクを中断・停止 |
| `GET` | `/health` | Workerおよびコンテナの健全性確認 |

### 4.2 API スキーマ定義

#### `POST /task` (リクエスト)
```json
{
  "task_id": "TASK-002",
  "type": "implementation",
  "title": "CLI parser implementation",
  "description": "clapを使用したCLIパーサーを実装し、--help出力を整形してください",
  "repository": "/workspace/project",
  "branch": "agent-a/task-002",
  "timeout_seconds": 300,
  "context_files": ["src/main.rs", "Cargo.toml"]
}
```

#### `GET /status` (レスポンス)
```json
{
  "agent_id": "agent-a",
  "role": "developer",
  "status": "RUNNING",
  "current_task_id": "TASK-002",
  "started_at": "2026-08-15T10:00:00Z",
  "progress_percent": 60
}
```

#### `GET /result` (レスポンス - 構造化データ)
```json
{
  "task_id": "TASK-002",
  "agent_id": "agent-a",
  "status": "SUCCESS",
  "summary": "CLIパーサーの実装を完了しました。引数バリデーションテストも追加済みです。",
  "files_changed": [
    "src/cli.rs",
    "src/main.rs",
    "Cargo.toml"
  ],
  "tests": [
    {
      "name": "test_cli_args_parsing",
      "passed": true,
      "duration_ms": 12
    }
  ],
  "commit": "a82f91c98e1",
  "errors": [],
  "execution_time_sec": 45.2
}
```

---

## 5. タスク管理 & 状態マシン

### 5.1 タスクライフサイクル
```text
[ PENDING ] ──(Assign)──> [ ASSIGNED ] ──(Start)──> [ RUNNING ]
                                                          │
          ┌───────────────────────────────────────────────┴───────────────┐
          │ (Success)                                                     │ (Failure)
          ▼                                                               ▼
   [ REVIEW / TESTING ]                                              [ FAILED ]
          │                                                               │
     ┌────┴───────────────┐                                          ┌────┴─────────────┐
     ▼ (Pass)             ▼ (Fail)                                   ▼ (Retry < 3)      ▼ (Retry >= 3)
[ COMPLETED ]         [ RETRY ]                                  [ RETRY ]     [ FAILED_PERMANENTLY ]
                          │                                          │                  │
                          └─────────────► [ RUNNING ] ◄──────────────┘                  ▼
                                                                                   (Alert User)
```

### 5.2 タスク状態定義一覧
- `PENDING`: タスク生成済み、未割り当て
- `ASSIGNED`: 特定Agentに割り当て完了
- `RUNNING`: Worker上で実行中
- `REVIEW`: Reviewer Agentによる検証中
- `TESTING`: Tester Agentによるテスト実行中
- `COMPLETED`: すべての検証を通過し完了
- `FAILED`: 実行エラーまたは検証不合格
- `RETRY`: エラー修正指示を伴う再試行中
- `FAILED_PERMANENTLY`: 最大試行回数到達による中断、ユーザー介入待ち

---

## 6. Git & ワークスペース管理

### 6.1 ブランチ運用
競合を防止するため、各Agentに独立したブランチおよびGit Worktreeを割り当てる。

```text
Project Git Repository
├── main (保護ブランチ)
│
├── agent-a/task-001 (Developer作業ブランチ)
├── agent-b/task-002 (Tester作業ブランチ)
├── agent-c/task-003 (Reviewer作業ブランチ)
├── agent-d/task-004 (Security作業ブランチ)
└── agent-e/task-005 (Research作業ブランチ)
```

### 6.2 統合ルール
1. Workerは作業完了時に自身のリモートブランチへコミット・プッシュする。
2. Reviewer / Tester / Security の全パスを確認後、Managerが`main`へファストフォワードまたはスカッシュマージを行う。

---

## 7. 安全装置 & リソース制限

AIエージェントの暴走やリソース枯渇を防ぐための厳格な安全基準を設ける。

### 7.1 安全ガードレール
- **`MAX_RETRY`**: デフォルト最大3回。超過時は自動停止しユーザーへエスカレーション。
- **`MAX_EXECUTION_TIME`**: タスク単位の最大実行時間（デフォルト300秒）。
- **`COMMAND_ALLOWLIST`**: コンテナ内で実行可能なコマンドを厳格にホワイトリスト化。
- **人間承認ゲート (Human-in-the-loop)**: ファイル削除（`rm -rf`）、外部ネットワーク接続、重要設定変更時はManagerがユーザーに確認を求める。

### 7.2 コンテナ制限
- 非rootユーザーでの実行
- コンテナあたりのCPUコア数、メモリ上限（例: 2GB）、プロセス数制限（pids-limit）
- 必要最小限のボリュームマウント（Workspace以外はRead-Only）

---

## 8. 外部連携システム

### 8.1 envdoctor 連携
ビルド・実行エラー時に「コードの問題か」「環境の問題か」を切り分ける環境診断を実施。
- PATH、コンパイラバージョン、ライブラリ依存関係、Docker環境の自動チェック。

### 8.2 jpcargo 連携
Rustプロジェクトにおけるコンパイルエラー・clippy警告を日本語解析し、ManagerおよびDeveloper Agentへ高精度な修正コンテキストを提供。

### 8.3 セキュリティスキャナ連携
- `cargo-audit`, `npm audit`, `pip-audit`, `Semgrep`, `Trivy`, `OSV-Scanner` による定期自動スキャン。

---

## 9. CLI仕様 (仮称: `mag`)

```bash
# プロジェクト初期化
mag init <project-name>

# オーケストレーター & Workerコンテナ起動
mag start

# 全Agent & タスク稼働状況確認
mag status

# タスク投入 (自然言語)
mag "PythonでFastAPIを使用したユーザー認証APIを作成してください"

# 各種管理コマンド
mag stop               # 停止
mag agents             # Agent一覧・状態
mag task list          # タスク一覧
mag logs <agent-id>    # ログ表示
mag test               # テスト実行
mag review             # レビュー実行
mag merge              # 成果物のマージ
```
