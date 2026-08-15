# Multi-Agent Development Orchestrator (`mag`) 拡張計画書 (addplan2.md)

本ドキュメントは、ユーザーからの追加要件（**コンテナ数の可変スケーリング** および **Google アカウント認証 / ログイン機能**、ならびに **実コード生成エンジンの強化**）に関する詳細設計仕様書です。

---

## 🎯 1. 主要追加要件

1. **コンテナ / エージェント数の動的可変化（Dynamic Agent/Container Scaling）**
   - ユーザーの要求規模やタスク量に応じて、起動するAgent（コンテナ）の数を1〜N台へ柔軟にスケール。
   - CLI引数（`--concurrency`, `--agents`, `--developers 3`）および `.mag/config.toml` からの動的コンテナ定義。
   - 複数Developerエージェントによる並列コード生成とコンフリクトフリーWorktreeマージ。

2. **Google ログイン & 認証基盤 (`mag login google` / `mag auth`)**
   - Google OAuth2.0 (Device Authorization Flow / PKCE Flow) をサポート。
   - ローカルCLIでの安全なトークン保存 (`.mag/credentials.json` または OSキーチェーン)。
   - ログイン状態の確認 (`mag whoami` / `mag status` での認証ユーザー表示)。
   - Google Gemini API / Cloud 連携時のシームレスなトークン利用。

3. **自然言語プロンプトからの実コード自動生成エンジンの強化**
   - 「`/home/guru/agytest にrustのプログラムを書いて`」などの指示から対象パス・言語・モジュールを自動抽出し、実際に動くプロジェクト（`Cargo.toml`, `src/main.rs`, `tests/` 等）を自動生成・検証・ビルド。

---

## 🏛️ 2. アーキテクチャ設計

### 2.1 コンテナ / エージェント動的スケーリング構成 (`mag-container` & `mag-scheduler`)

```text
┌─────────────────────────────────────────────────────────────┐
│                       Manager Agent                         │
│  - Task DAG Planner (Decomposes requirement into N tasks)   │
│  - Dynamic Worker Pool Manager (Spawns 1..N containers)    │
└──────────────────────────────┬──────────────────────────────┘
                               │ (Dynamic Pool: Scaling 1..N)
       ┌───────────────────────┼───────────────────────┐
       ▼                       ▼                       ▼
┌──────────────┐        ┌──────────────┐        ┌──────────────┐
│  Worker-1    │        │  Worker-2    │        │  Worker-N    │
│ (Developer-1)│        │ (Developer-2)│        │ (Security-1) │
│ Worktree-1   │        │ Worktree-2   │        │ Worktree-N   │
└──────────────┘        └──────────────┘        └──────────────┘
```

- **プール管理 (`WorkerPool`)**: 必要タスク数に応じてWorkerコンテナをオンデマンド起動・停止。
- **リソース最適化**: アイドル時の自動停止、最大並列数 (`max_workers`) 制御。

### 2.2 Google ログイン & 認証フロー (`mag-auth` / `mag-api`)

```text
User CLI (`mag login google`)
   │
   ├─► 1. Request Device Code from Google OAuth2 Server
   │   ◄── Return User Code + Verification URL (https://www.google.com/device)
   │
   ├─► 2. Display URL & User Code to CLI terminal / Open browser
   │   (User authorizes in browser)
   │
   ├─► 3. Poll Google Token Endpoint with Device Code
   │   ◄── Return Access Token + Refresh Token + ID Token (Email/Name)
   │
   └─► 4. Save to `.mag/credentials.json` (chmod 600)
       [✓] "Logged in as user@example.com"
```

---

## 📦 3. Crate 拡張方針

1. **`crates/mag-common`**:
   - `AuthConfig` (ユーザー情報, Access Token, Refresh Token, 有効期限)
   - `ScalePolicy` (最小/最大ワーカー数, 並列度)

2. **`crates/mag-container`**:
   - `spawn_worker_pool(count: usize)`
   - `scale_workers(target_count: usize)`

3. **`crates/mag-manager`**:
   - 自然言語解析の強化（指定パス `/home/guru/agytest` への Rust / Python プログラムの自動生成）
   - 並列Workerへのタスク自動振り分け

4. **`crates/mag-cli`**:
   - `mag login [google|gemini|token]`
   - `mag logout`
   - `mag whoami`
   - `mag scale --workers <N>` または `mag "<prompt>" --workers <N>`
