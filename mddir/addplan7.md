# agyCLI-multicoding 変更部分計画書
## Agentごとの別アカウント認証・コンテナ永続化・ターミナル切断対応

- 対象: `https://github.com/sha256san/agyCLI-multicoding`
- 方針: 既存の Manager / Developer / Tester / Reviewer / Security / Researcher 構成を維持し、必要な機能だけを追加する。
- 作成日: 2026-08-16

> **重要な前提**
>
> Antigravity CLI の公式READMEでは、認証は system keyring を利用し、利用可能なセッションがない場合は Google Sign-In にフォールバックすると説明されている。また Remote / SSH では認証URLを表示してローカルブラウザから認証できる。したがって、本計画では認証トークンを独自に抽出・コピーするのではなく、Antigravity CLI がサポートする認証フローをコンテナごとの独立した認証環境から利用する。
>
> コンテナ環境・headless Linuxではkeyring/Secret Serviceの永続化に問題が発生する報告があるため、**認証方式を実装前に実機で検証し、CLIがコンテナ内で選択する保存方式を確認する**ことを必須とする。CLIの内部仕様を推測して認証ファイルを直接操作する実装は採用しない。

---

# 1. 今回追加する機能

今回の変更範囲は次の5つに限定する。

```text
1. Agentごとの独立アカウント
2. Agentごとの永続認証領域
3. ManagerからAgentへの認証状態管理
4. ターミナル切断後もTaskを継続するDetached Execution
5. コンテナ再起動後の認証・Task復旧
```

最終形：

```text
                    agy CLI
                       |
                Persistent Manager
                       |
       +---------------+---------------+
       |               |               |
       v               v               v
 Developer          Tester          Reviewer
 Account A          Account B       Account C
 Auth-A             Auth-B          Auth-C
       |
       +-----------+-----------+
                   |
             Security / Researcher
             Account D / E
```

**1 Agent = 1 Container = 1 Account = 1 Persistent Auth Store**

を基本設計とする。

---

# 2. Antigravity CLI認証方式の実現性評価

## 2.1 確認できている仕様

Antigravity CLI公式READMEでは、

- system keyringで認証
- 有効なセッションがない場合はGoogle Sign-In
- Localではブラウザ認証
- Remote / SSHでは認証URLを表示
- `/logout`で保存された認証情報を消去

という仕様が公開されている。

また公式CLIはRemote / SSH環境を想定しているため、Dockerコンテナから認証URLを取得し、ホスト側ブラウザで認証する方式をMVPの第一候補とする。

## 2.2 コンテナでの問題

LinuxコンテナではGUIのsystem keyringが存在しない、D-Bus/Secret Serviceが利用できない、keyringがunlockできない、といった問題が報告されている。

そのため、

```text
Container
  |
  +-- system keyring
  |
  +-- D-Bus
  |
  +-- Secret Service
```

を前提としていきなり実装するのではなく、まず実機検証を行う。

## 2.3 採用基準

以下の優先順位で採用する。

### A. 公式CLIがコンテナ環境で永続的なファイル保存を選択する場合

その保存領域を**Agent専用Docker Volume**にする。

```text
agy-developer-auth
agy-tester-auth
agy-reviewer-auth
agy-security-auth
agy-researcher-auth
```

### B. system keyring / Secret Serviceが必須の場合

Agentごとに独立したSecret Service環境を構築し、関連する永続データをAgent専用Volumeへ保存する。

### C. 公式CLIがコンテナで安定して認証を永続化できない場合

認証情報を独自に抽出して保存するのではなく、

1. Antigravity CLIの対応バージョンを固定
2. 公式Remote/SSH OAuthフローを使用
3. CLIの保存方式に対応した永続領域を用意
4. 再ログインが必要になった場合は明示的に `auth status` で検出

とする。

---

# 3. Agent別アカウント設計

## 3.1 アカウント対応

| Agent | Account | Auth Store |
|---|---|---|
| Developer | Account A | `agy-developer-auth` |
| Tester | Account B | `agy-tester-auth` |
| Reviewer | Account C | `agy-reviewer-auth` |
| Security | Account D | `agy-security-auth` |
| Researcher | Account E | `agy-researcher-auth` |

アカウント情報そのものをGit管理しない。

`Account A` 等は内部識別子であり、メールアドレスやOAuth tokenを設定ファイルへ直接保存しない。

---

# 4. Docker Volume設計

Dockerコンテナの書き込み可能レイヤーはコンテナ削除時に失われるため、認証状態などの永続データはVolumeへ分離する。

## 4.1 Volume

```yaml
volumes:
  developer_auth:
    name: agy_developer_auth

  tester_auth:
    name: agy_tester_auth

  reviewer_auth:
    name: agy_reviewer_auth

  security_auth:
    name: agy_security_auth

  researcher_auth:
    name: agy_researcher_auth
```

各Volumeは**1 Agent専用**とする。

## 4.2 Developer

```yaml
services:
  developer:
    volumes:
      - developer_auth:/home/agent/.agy-auth
```

ただし、`.agy-auth` は仮のマウント先であり、実際にはAntigravity CLIが使用する認証保存先を実機検証で確定する。

**CLIが使用する保存先が別である場合は、その実際の保存先をVolume化する。**

---

# 5. 認証情報をAgent間で共有しない

禁止：

```text
Developer
    |
    +---- auth volume ----+
                          |
Tester -------------------+
```

採用：

```text
Developer -> developer_auth
Tester    -> tester_auth
Reviewer  -> reviewer_auth
Security  -> security_auth
Researcher-> researcher_auth
```

各Volumeは他Agentへmountしない。

---

# 6. 認証初期化フロー

## 6.1 初回

```bash
agy agent auth developer
```

Manager：

```text
1. Developer Container起動
2. Developer Auth Volumeをmount
3. Antigravity CLI起動
4. 未認証を検出
5. Remote/SSH方式の認証URLを取得
6. URLをユーザーへ表示
7. ユーザーがAccount Aで認証
8. CLI側で認証完了
9. auth statusで確認
10. Volumeへ保存されたことを確認
```

Testerも同じ流れでAccount Bを使用する。

---

# 7. 認証URL方式

コンテナからブラウザを直接開くことは前提にしない。

```text
Developer Container
       |
       | OAuth URL
       v
Manager
       |
       v
Terminal
       |
       v
User Browser
       |
       v
Google Account A
       |
       v
Antigravity CLI
```

Antigravity CLI公式READMEにもRemote / SSH時に認証URLを表示する仕様があるため、この方式を第一候補とする。

---

# 8. `agy auth` コマンド

追加するCLI：

```bash
agy auth status
agy auth login <agent>
agy auth logout <agent>
agy auth verify <agent>
agy auth list
```

## 8.1 状態確認

```bash
agy auth status
```

出力：

```text
AGENT AUTHENTICATION

Developer    AUTHENTICATED
Tester       AUTHENTICATED
Reviewer     AUTHENTICATED
Security     AUTHENTICATED
Researcher   AUTHENTICATED
```

メールアドレスなどの個人情報はデフォルトでは表示しない。

必要な場合のみ：

```bash
agy auth status --verbose
```

---

# 9. 認証状態

認証状態は次の4状態で管理する。

```text
UNINITIALIZED
AUTHENTICATING
AUTHENTICATED
EXPIRED
```

異常：

```text
AUTH_ERROR
```

---

# 10. 認証Health Check

ManagerはAgentごとに定期的に認証状態を確認する。

```text
Manager
 |
 +-- Developer -> auth verify
 +-- Tester    -> auth verify
 +-- Reviewer  -> auth verify
 +-- Security  -> auth verify
 +-- Researcher-> auth verify
```

認証失敗時：

```text
AUTHENTICATED
      |
      v
AUTH_ERROR
      |
      v
User notification
```

**自動で別アカウントへ切り替えない。**

---

# 11. Account Isolation Test

必須テスト：

```text
Developer -> Account A
Tester    -> Account B
```

DeveloperコンテナからTesterの認証情報が取得できないことを確認する。

逆方向も同様。

```text
Developer -> cannot access tester_auth
Tester    -> cannot access developer_auth
```

---

# 12. ターミナル切断

今回のもう一つの重要機能。

```bash
agy run "プロジェクトを完成させる"
```

Task IDを発行：

```text
Task ID: task-0001
Status: RUNNING
```

その後、ユーザーがターミナルを閉じてもTaskは停止しない。

---

# 13. CLIとTaskの分離

現在のCLIプロセスにTaskを直接依存させない。

悪い構成：

```text
Terminal
  |
  v
agy process
  |
  v
Agent
```

Terminal終了：

```text
agy process -> killed
Agent -> stopped
```

採用：

```text
Terminal
  |
  v
agy client
  |
  v
Persistent Manager
  |
  v
Agent
```

Terminal終了：

```text
agy client -> disconnected
Manager -> running
Agent -> running
Task -> running
```

---

# 14. Task Persistence

最低限、以下を永続化する。

```text
task_id
project_id
status
prompt
current_agent
created_at
started_at
updated_at
completed_at
error
result
```

SQLiteをMVPの第一候補とする。

---

# 15. Session Persistence

```text
session_id
task_id
client_id
attached
created_at
last_seen
```

を保存する。

Terminal再接続時にTaskへattachできるようにする。

---

# 16. CLI

追加コマンド：

```bash
agy run "..."
agy run --detach "..."

agy task list
agy task status <task-id>
agy task stop <task-id>
agy task resume <task-id>

agy attach <task-id>
agy detach

agy logs <task-id>
```

---

# 17. Attach / Detach

## 開始

```bash
agy run --detach "Rustプロジェクトを実装"
```

## 後から確認

```bash
agy attach task-0001
```

## 状態だけ確認

```bash
agy task status task-0001
```

## ログ確認

```bash
agy logs task-0001
```

---

# 18. Manager常駐

ManagerはCLIの子プロセスではなく、独立したサービスとして起動する。

```text
agy CLI
   |
   | HTTP / Unix Socket
   v
Manager
```

Docker ComposeではManagerを常駐サービスとする。

---

# 19. Container Restart

Agentコンテナにはrestart policyを設定する。

```yaml
restart: unless-stopped
```

ただし、restart policyだけでTask状態を復元できるわけではない。

必ず、

```text
Docker restart
      |
      v
Manager
      |
      v
Task DB
      |
      v
Task recovery
```

を実装する。

---

# 20. Heartbeat

AgentからManagerへHeartbeatを送る。

```text
Developer
   |
   | heartbeat
   v
Manager
```

例えば10秒周期。

Managerが一定時間Heartbeatを受信しなければ、

```text
HEALTHY
   |
   v
UNRESPONSIVE
   |
   v
Container inspection
```

とする。

---

# 21. Task Recovery

Manager再起動時：

```text
Task DB
  |
  +-- RUNNING
  +-- WAITING
  +-- COMPLETED
  +-- FAILED
```

`RUNNING` / `WAITING` のTaskを検査する。

```text
Agent exists?
  |
  +-- YES -> restore
  |
  +-- NO  -> recreate
```

---

# 22. 重要：Taskを二重実行しない

Managerが再起動したとき、同じTaskを2つ起動しない。

Taskごとにlease/lockを持つ。

```text
task-0001
owner: developer
lease_until: ...
```

Lease期限切れを確認してからRecoveryする。

---

# 23. Agent Identity

Agentごとに固定IDを持つ。

```text
developer-01
tester-01
reviewer-01
security-01
researcher-01
```

将来的なAgent増設にも対応する。

---

# 24. AgentごとのLinuxユーザー

可能ならコンテナ内でもAgent専用Linuxユーザーを使用する。

```text
Developer -> user: developer
Tester    -> user: tester
Reviewer  -> user: reviewer
Security  -> user: security
Researcher-> user: researcher
```

認証Volumeの所有者も対応するAgentに限定する。

---

# 25. Secret Leakage対策

認証情報を以下へ保存しない。

```text
Git repository
Dockerfile
Docker image
docker-compose.yml
Task DB
Agent message
Prompt
Standard output
Debug log
```

ログには、

```text
token=********
credential=********
```

のようなmaskingを適用する。

---

# 26. `docker compose down` と `down -v`

通常：

```bash
docker compose down
```

では認証Volumeを維持する。

一方、

```bash
docker compose down -v
```

はVolume削除を伴うため、認証状態を失う可能性がある。

したがって、通常のCLIではVolume削除を安全確認なしに実行しない。

---

# 27. `agy clean`

以下を明確に分離する。

```bash
agy clean containers
agy clean cache
agy clean auth --agent developer
agy clean all
```

認証削除には必ず確認を要求する。

```text
WARNING:
This will remove Developer authentication state.

Continue? [y/N]
```

---

# 28. Project DataとAuth Dataを分離

```text
Agent Container
|
+-- /workspace
|     └── project files
|
+-- /agent-state
|     └── task/session state
|
+-- Antigravity auth storage
      └── dedicated persistent volume
```

認証データをプロジェクトVolumeへ混在させない。

---

# 29. Compose設計

概念：

```yaml
services:

  manager:
    restart: unless-stopped

  developer:
    restart: unless-stopped
    volumes:
      - developer_auth:<verified-auth-path>
      - developer_state:/agent-state

  tester:
    restart: unless-stopped
    volumes:
      - tester_auth:<verified-auth-path>
      - tester_state:/agent-state

  reviewer:
    restart: unless-stopped
    volumes:
      - reviewer_auth:<verified-auth-path>
      - reviewer_state:/agent-state

  security:
    restart: unless-stopped
    volumes:
      - security_auth:<verified-auth-path>
      - security_state:/agent-state

  researcher:
    restart: unless-stopped
    volumes:
      - researcher_auth:<verified-auth-path>
      - researcher_state:/agent-state
```

`<verified-auth-path>` はAntigravity CLIの実機検証後に決定する。

---

# 30. なぜ固定パスを最初から決めないのか

Antigravity CLIはsystem keyringを利用することが公式仕様として示されている。

さらに公開Issueでは、環境によってSecret Service / D-Busやコンテナ内のfile-based storageなど、保存経路が異なる報告がある。

そのため、

```text
~/.xxx/auth.json
```

などを推測して固定するのは危険。

実装時に、

```bash
agy
```

でログイン

↓

認証成功

↓

`agy auth status`

↓

再起動

↓

再確認

を行い、実際の保存機構を特定する。

---

# 31. 認証実証試験

実装開始直後に以下のProof of Conceptを作る。

## Test A

```text
Container A
Account A
Login
Exit
Start again
```

期待：

```text
AUTHENTICATED
```

## Test B

```text
Container A
Account A

Container B
Account B
```

期待：

```text
A != B
```

## Test C

```text
Container A
Account A
docker restart
```

期待：

```text
AUTHENTICATED
```

## Test D

```text
Container A
Account A
docker rm container
docker recreate container
reuse auth volume
```

期待：

```text
AUTHENTICATED
```

## Test E

```text
Terminal close
Manager remains running
Agent remains running
```

期待：

```text
Task = RUNNING
```

---

# 32. 失敗条件

以下の場合は本番実装へ進まない。

```text
1. コンテナ再起動で毎回ログインが必要
2. Agent AがAgent Bの認証情報へアクセス可能
3. Terminal終了でTaskが停止する
4. Manager再起動でTaskを失う
5. 認証情報がログへ出る
6. 認証情報をGit管理する必要がある
7. 非公式なtoken抽出が必要
```

---

# 33. 実装Phase

## Phase A — 認証PoC

- [ ] Antigravity CLIのバージョン固定
- [ ] Docker内インストール
- [ ] Developerコンテナ作成
- [ ] Account Aでログイン
- [ ] auth status確認
- [ ] コンテナ再起動
- [ ] コンテナ再作成
- [ ] Volume再利用
- [ ] 再認証なしで動作することを確認

---

## Phase B — Multi Account

- [ ] Tester Account B
- [ ] Reviewer Account C
- [ ] Security Account D
- [ ] Researcher Account E
- [ ] Volume分離
- [ ] Linux user分離
- [ ] Cross-access test

---

## Phase C — Persistent Manager

- [ ] Manager daemon
- [ ] SQLite
- [ ] Task model
- [ ] Session model
- [ ] Event model
- [ ] Agent registry

---

## Phase D — Detached Execution

- [ ] `agy run`
- [ ] `agy run --detach`
- [ ] `agy attach`
- [ ] `agy detach`
- [ ] `agy task list`
- [ ] `agy task status`
- [ ] `agy logs`

---

## Phase E — Recovery

- [ ] Heartbeat
- [ ] Healthcheck
- [ ] Manager restart recovery
- [ ] Agent restart recovery
- [ ] Task lease
- [ ] Duplicate execution prevention

---

## Phase F — Security

- [ ] Credential masking
- [ ] Volume permissions
- [ ] Agent isolation
- [ ] Audit log
- [ ] Auth deletion confirmation
- [ ] Secret scanning

---

# 34. 最終動作

初回：

```bash
agy auth login developer
```

Account Aで認証。

```bash
agy auth login tester
```

Account Bで認証。

以下同様に各Agentを認証する。

---

# 35. 開発開始

```bash
agy run --detach "このRustプロジェクトを完成させてください"
```

Manager：

```text
Task: task-0001

Researcher -> RUNNING
Developer  -> WAITING
Tester     -> WAITING
Reviewer   -> WAITING
Security   -> WAITING
```

---

# 36. Terminalを閉じる

```text
Terminal
   X
```

しかし、

```text
Manager       RUNNING
Researcher    RUNNING
Developer     RUNNING
Task-0001     RUNNING
```

を維持。

---

# 37. 翌日再接続

```bash
agy task list
```

```text
task-0001    RUNNING    74%
```

または、

```bash
agy attach task-0001
```

```text
Researcher  COMPLETE
Developer   COMPLETE
Tester      RUNNING
Reviewer    WAITING
Security    WAITING
```

---

# 38. アカウント状態

```bash
agy auth status
```

```text
Developer   Account A   AUTHENTICATED
Tester      Account B   AUTHENTICATED
Reviewer    Account C   AUTHENTICATED
Security    Account D   AUTHENTICATED
Researcher  Account E   AUTHENTICATED
```

---

# 39. MVP完成条件

以下をすべて満たしたら今回の変更を完成とする。

- [ ] 5 Agentがそれぞれ別Accountでログインできる
- [ ] 各Agentの認証状態が独立している
- [ ] Agent AからAgent Bの認証状態を取得できない
- [ ] コンテナ再起動後も認証状態を復元できる
- [ ] コンテナ再作成 + 同じVolumeで認証状態を復元できる
- [ ] Terminalを閉じてもTaskが継続する
- [ ] `agy attach`で再接続できる
- [ ] Manager再起動後にTaskを復元できる
- [ ] Agent停止後にTaskを復旧できる
- [ ] 認証情報がログに出ない
- [ ] Gitへ認証情報が入らない
- [ ] 認証情報の削除を明示的に操作できる

---

# 40. 実装上の重要な判断

## 採用するもの

```text
Docker Compose
Docker Named Volume
Persistent Manager
SQLite
Agent-specific Auth Store
Agent-specific Linux User
Heartbeat
Healthcheck
Task Lease
Attach / Detach
Official OAuth Flow
```

## 採用しないもの

```text
認証tokenの手動抽出
認証tokenの.env保存
認証tokenのGit保存
全Agentで同じAuth Volumeを共有
Hostの秘密鍵ringを無条件でbind mount
Terminal processをTask Managerとして利用
tmuxだけに依存したTask管理
```

tmuxは手動デバッグ用途として使用可能だが、製品の永続Task機構にはしない。

---

# 41. 変更後のアーキテクチャ

```text
                         User
                          |
                          v
                     ┌────────┐
                     │ agy CLI│
                     └───┬────┘
                         |
                  HTTP / Unix Socket
                         |
                         v
              ┌─────────────────────┐
              │ Persistent Manager  │
              │                     │
              │ Task Manager        │
              │ Session Manager     │
              │ Agent Scheduler     │
              │ Recovery Manager    │
              │ Auth Status Manager │
              └──────────┬──────────┘
                         |
       +-----------------+-----------------+
       |                 |                 |
       v                 v                 v
  Developer          Tester            Reviewer
  Account A          Account B         Account C
  Auth Volume A      Auth Volume B     Auth Volume C
       |                 |                 |
       +-----------------+-----------------+
                         |
                  +------+------+
                  |             |
                  v             v
              Security      Researcher
              Account D     Account E
              Auth D        Auth E
```

---

# 42. 既存計画書との差分

今回の変更では、以前の計画に含まれていた一般的な機能をすべて再設計するのではなく、以下だけを具体化する。

```text
追加
├── Per-Agent Account
├── Per-Agent Auth Volume
├── Auth Status
├── Auth Login/Logout
├── Container Auth Persistence
├── Authentication PoC
├── Terminal Detach
├── Persistent Manager
├── Task Persistence
├── Task Recovery
├── Heartbeat
└── Cross-Agent Credential Isolation

変更
├── Authentication Manager
└── Session Manager

廃止
└── 「tmuxを中心にした永続実行」という設計
```

---

# 43. 技術的な実現性評価

| 機能 | 実現性 | 方針 |
|---|---:|---|
| Agent別Container | 非常に高い | 現行Docker構成を拡張 |
| Agent別Volume | 非常に高い | Named Volume |
| Agent別Linux user | 高い | Dockerfileで作成 |
| Detached Manager | 非常に高い | Managerを独立サービス化 |
| SQLite Task DB | 非常に高い | MVP採用 |
| Heartbeat | 非常に高い | Manager API |
| Healthcheck | 非常に高い | Docker Compose |
| Task Recovery | 高い | DB + lease |
| Attach/Detach | 高い | CLIとManagerを分離 |
| 5アカウント同時利用 | 技術的には可能性あり | 利用規約・プラン条件を確認 |
| Antigravity認証永続化 | 中程度 | コンテナ環境でPoC必須 |
| Headless OAuth | 中程度 | 公式Remote/SSHフローを使用 |
| system keyring永続化 | 中程度 | D-Bus/Secret Service検証必須 |
| 独自token保存 | 非推奨 | 実装しない |

---

# 44. 最重要リスク

今回最大の技術リスクはDockerそのものではなく、

> **Antigravity CLIのコンテナ内認証状態を、Agentごとに独立させながら再起動後も安全に永続化できるか**

である。

したがって、実装順序は必ず、

```text
Authentication PoC
        ↓
Persistence PoC
        ↓
Multi-account PoC
        ↓
Manager integration
        ↓
Task persistence
        ↓
Detach / Attach
```

とする。

認証PoCを成功させる前に大量のManager機能を実装しない。

---

# 45. 完成後のユーザー体験

最終的には、

```bash
agy auth login developer
agy auth login tester
agy auth login reviewer
agy auth login security
agy auth login researcher
```

で各Agentを別アカウントに設定。

その後、

```bash
agy run --detach "プロジェクトを完成させてください"
```

だけで開発開始。

ユーザーはターミナルを閉じる。

AI側：

```text
Manager       RUNNING
Developer     RUNNING
Tester        RUNNING
Reviewer      RUNNING
Security      RUNNING
Researcher    RUNNING
```

後から、

```bash
agy task list
agy attach <task-id>
```

で復帰する。

これを今回の変更の完成形とする。
