# Multi-Agent Development Orchestrator (`mag` / `agycli`) 拡張仕様書 (addplan4.md)

本ドキュメントは、**コンテナ個別ログイン (`agycli login <container_name>`)**、**ログイン永続化 & 再起動時自動再認証**、**少数ワーカー時のマルチロール動的協調（タスク協力型キュー）**、**Worktreeによる並列競合防止**、および **タスク完了後の自動mainマージ & GitHub `v0.2.1` リリースプッシュ** に関する詳細設計仕様書です。

---

## 🎯 1. 主要要件一覧

1. **コンテナ指定ログイン (`agycli login <container_name>`)**
   - `agycli login agent-a` や `agycli login mag-agent-b` のようにコンテナ名を指定して実行可能。
   - 内部で対象コンテナの `agy auth login` を起動し、ブラウザ認証URLをターミナルに表示してユーザーがログイン。
   - 認証クレデンシャルはホスト永続領域 (`.mag/containers/<container_name>/credentials.json`) に保存。

2. **コンテナ停止・再起動・再インストール時の認証永続化 (Zero Auth Loss)**
   - コンテナを再ビルド・再起動・アップデートしても、永続ボリューム／ホスト側マウントによりログイン情報が維持され、起動時に自動で再認証・ロード。

3. **メンバー数に応じた動的マルチロール & タスク協力型ディスパッチ**
   - ワーカー数が少ない場合（例: 2台）でも、固定ロールに縛られず複数ロール（Agent 1: 調査+実装, Agent 2: テスト+レビュー+セキュリティ）を兼任。
   - ロール固定型ではなく、タスクが終わり次第、次の待機中タスクを空いたコンテナが協力して取得・実行する **タスク協調キュー (Work-Stealing Task Queue)**。

4. **コンテナ並列作業時のコンフリクト防止 (Git Worktree 分離)**
   - 各コンテナ・各タスクごとに独立した作業ディレクトリ（`.mag/worktrees/agent-<id>-task-<task_id>`）および専用ブランチを割り当て、物理的なファイル衝突を完全に防止。

5. **タスク完了時の自動 `main` マージ & `v0.2.1` タグ付け・プッシュ**
   - 全タスクのテスト・レビューが完了後、ManagerがWorktreeブランチを自動で `main` に安全にマージ。
   - `v0.2.1` タグを生成し、GitHubリモート (`origin main --tags`) に自動プッシュ。

---

## 🏗️ 2. アーキテクチャ設計

### 2.1 コンテナ指定ログイン & 永続化フロー

```text
Host CLI: $ agycli login "agent-a"
   │
   ├─► 1. Identify container "mag-agent-a" or "agent-a"
   ├─► 2. Execute / trigger OAuth browser login session
   │      - Request Device/Browser URL: https://www.google.com/device
   │      - User authorizes in browser
   │
   ├─► 3. Save to Host Persistent Directory:
   │      `.mag/containers/agent-a/credentials.json`
   │      (Mounted into container `/home/agent-a/.mag/credentials.json`)
   │
   └─► 4. On Container Restart / Update:
          Container immediately reads credentials -> Status: AUTHENTICATED
```

### 2.2 動的マルチロール & 協調型タスクキュー

```text
Available Workers: 2 (Agent-1, Agent-2)
Task DAG: [T1: Spec] ─► [T2: Impl] ─► [T3: Test] ─► [T4: Review] ─► [T5: Security]

Execution Timeline:
Agent-1 (Idle)  ──► Claims T1 (Spec)   ──► Finished T1 ──► Claims T2 (Impl)
Agent-2 (Idle)  ──► Waits for T2 Ready ──► Claims T3 (Test) ──► Claims T4 (Review) ──► Claims T5 (Security)

Merge Phase:
Manager Agent ──► Validates all task results ──► Merges branch to `main` ──► Git Tag v0.2.1 & Push
```
