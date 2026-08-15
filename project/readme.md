# Multi-Agent Development Orchestrator (`mag`)

> **AIを増やすのではなく、AIを組織化する。**  
> 複数のAIエージェントが独立したコンテナ環境で協調し、自律的にソフトウェアを開発・検証・統合する次世代オーケストレーションプラットフォーム。

---

## 📖 概要

**Multi-Agent Development Orchestrator (`mag`)** は、単一のAIに頼るのではなく、**Manager（統括）**、**Developer（実装）**、**Tester（テスト）**、**Reviewer（レビュー）**、**Security（セキュリティ）**、**Researcher（調査）** の専門エージェントをコンテナ単位で組織化し、ソフトウェア開発サイクル（設計→実装→テスト→レビュー→セキュリティ→自己修復→統合）を完全自動化するシステムです。

```text
                        Manager (Ubuntu 26.04)
                                  │
      ┌───────────────┬───────────┴───────────┬───────────────┐
      ▼               ▼                       ▼               ▼
 Developer          Tester                 Reviewer        Security
 (Agent A)        (Agent B)               (Agent C)       (Agent D)
      │               │                       │               │
      └───────────────┼───────────────────────┴───────────────┘
                      ▼
            Manager (Evaluation & Merge)
                      ▼
               Project Artifacts
```

---

## ✨ 主な特徴

- 🏢 **組織化された役割分担**: 各AIに特化したロール（実装/テスト/レビュー/セキュリティ/調査）を付与。
- 📦 **完全なコンテナ隔離**: WorkerはDockerコンテナ内で独立実行され、依存関係の衝突や環境汚染を防止。
- 🔄 **自己修復 (Self-Repair) ループ**: ビルド・テスト失敗時に自動で原因を解析し、修正指示を再試行（最大3回）。
- 🌳 **Git Worktree 分離**: Agentごとに専用ブランチ・作業領域を割り当て、コンフリクトを回避。
- 🛡️ **堅牢な安全装置**: タイムアウト制限、コマンドホワイトリスト、危険操作時の人間確認ゲートを標準装備。
- 🧩 **専門ツール連携**: `envdoctor` (環境起因エラー診断) や `jpcargo` (Rust日本語エラー解析) との連携。

---

## 👥 エージェント一覧

| エージェント | 役割 | 担当内容 |
|---|---|---|
| **Manager** | 全体統括 | 要件分析、タスク分解、ディスパッチ、品質評価、Gitマージ |
| **Agent A (Developer)** | 実装 | ソースコード作成、バグ修正、リファクタリング |
| **Agent B (Tester)** | テスト | 単体・結合テスト、ビルド検証、回帰テスト実行 |
| **Agent C (Reviewer)** | レビュー | 可読性、保守性、設計妥当性、パフォーマンス検証 |
| **Agent D (Security)** | セキュリティ | 脆弱性診断 (CVE/OSV)、依存関係スキャン、シークレット検出 |
| **Agent E (Researcher)** | 調査 & 文書 | 技術仕様調査、設計書作成、README/CHANGELOG保守 |

---

## 🚀 CLI インターフェース (予定)

```bash
# プロジェクトの初期化
mag init my-project

# オーケストレーターおよびWorkerコンテナの起動
mag start

# 全Agentおよびタスクの稼働状態確認
mag status

# 自然言語による開発タスク投入
mag "FastAPIを使用したJWT認証付きユーザー管理APIを構築してください"

# 各種管理
mag logs agent-a   # ログの確認
mag test           # テストサイクルの強制実行
mag review         # レビューの強制実行
mag stop           # システムの安全停止
```

---

## 📂 プロジェクト構成

```text
agyCLI++/
├── mddir/                      # プロジェクトドキュメント体系
│   ├── SPEC.md                 # 要件・仕様定義書
│   ├── TODO.md                 # 未実装タスク & ロードマップ
│   ├── MEMORY.md               # 知識ベース・ADR・設計原則
│   ├── AGENTS.md               # AIエージェント行動規範・ルール
│   ├── CHANGELOG.md            # 変更履歴
│   └── plan.md                 # マスターアーキテクチャ詳細構想
│
└── project/                    # 実装コードベース
    ├── readme.md               # 本ドキュメント
    └── src/                    # ソースコード
        ├── manager/            # Manager Agent & スケジューラ
        ├── worker/             # Worker Agent & 実行エンジン
        └── common/             # 共通データスキーマ & ユーティリティ
```

---

## 📚 ドキュメント詳細

- 📘 [仕様書 (SPEC.md)](file:///home/guru/agyCLI++/mddir/SPEC.md)
- 📝 [実装計画 & ロードマップ (TODO.md)](file:///home/guru/agyCLI++/mddir/TODO.md)
- 🧠 [知識ベース & 設計原則 (MEMORY.md)](file:///home/guru/agyCLI++/mddir/MEMORY.md)
- 🤖 [エージェント行動規範 (AGENTS.md)](file:///home/guru/agyCLI++/mddir/AGENTS.md)
- 📜 [変更履歴 (CHANGELOG.md)](file:///home/guru/agyCLI++/mddir/CHANGELOG.md)
- 🗺️ [マスター計画 (plan.md)](file:///home/guru/agyCLI++/mddir/plan.md)
