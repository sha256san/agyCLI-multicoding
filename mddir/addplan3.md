# Multi-Agent Development Orchestrator (`mag`) コンテナ内 agycli 統合 & リリース計画書 (addplan3.md)

本ドキュメントは、**コンテナ生成時の `agycli` 自動インストール**、**コンテナ内での標準ログイン自動連携**、および **GitHubへの `v0.2.0` リリースプッシュ** に関する詳細仕様です。

---

## 🎯 1. 主要要件

1. **コンテナ内への `agycli` 自動インストール**
   - 全 Worker コンテナ（Developer, Tester, Reviewer, Security, Researcher）および Manager コンテナのビルド・起動時に、`agycli` / `mag` 実行バイナリを `/usr/local/bin/` に配置。
   - コンテナ内のどのユーザー（`agent-a` 〜 `agent-e`）からも `agycli` コマンドが即座に使用可能であること。

2. **コンテナ内 `agycli` の自動ログイン連携**
   - ホスト側の認証情報（`.mag/credentials.json`）をコンテナ内と安全にマウント・共有。
   - コンテナ起動時に `agycli whoami` または `agycli status` で、ホスト側と同じログイン状態（Google OAuth / トークン認証）を自動認識。
   - コンテナ内からでも標準コマンド（`agycli login google` / `agycli login --token <TOKEN>`）による再ログイン・再認証が可能。

3. **動的コンテナ生成時 (`WorkerPoolManager`) の自動注入**
   - `crates/mag-container` の動的スケーリング・生成時にも、コンテナ起動オプション（ボリュームマウント・環境変数）により `agycli` と認証状態を自動適用。

4. **GitHub への `v0.2.0` タグ付け & プッシュ**
   - すべての変更をコミットし、`v0.2.0` の Git タグを作成。
   - `origin` (`git@github.com:sha256san/agyCLI-multicoding.git`) の `main` ブランチおよびタグをプッシュ。

---

## 🏗️ 2. アーキテクチャ構成

```text
Host System (Ubuntu 26.04)
├── ~/.cargo/bin/agycli (mag)
└── /workspace/.mag/credentials.json (Logged in as user@google.com)
            │
            │  [Volume Mount / Injection]
            ▼
Docker Containers (mag-agent-a .. mag-agent-e)
├── /usr/local/bin/agycli (Installed binary)
├── /usr/local/bin/mag
└── /workspace/.mag/credentials.json (Shared standard login credentials)
            │
            ▼
Container CLI Execution:
$ agycli whoami
Authenticated User:
  Provider: google
  Email:    user@google.com
```
