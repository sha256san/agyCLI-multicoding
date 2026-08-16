# Multi-Agent Development Orchestrator (`mag` / `agycli`) 第6次拡張仕様書 (addplan6.md)

本ドキュメントは、**エージェント初期クリーン状態**、**事前エージェント別ログイン連携 (`agycli login <agent-name>`)**、**`agent.md` によるログイン状態・アカウント管理**、**ログイン済みアカウントに基づく Manager 動的タスク割り振り**、および **`task.md` による実行詳細記録・自己修復・最終結果レポート出力** に関する詳細仕様書です。

---

## 🎯 1. 主要要件一覧

1. **エージェントの初期クリーン状態**
   - 初回起動時や新規プロジェクト時は何もないクリーンな状態で待機。

2. **事前ログイン & アカウント紐付け (`agycli login <agent-name>`)**
   - ユーザーが事前に対象エージェント（例: `agycli login agent-a` や `agycli login dev-1`）を指定してログイン。
   - バックグラウンドで `agy` 認証プロセスを実行し、発行された認証URL（`https://www.google.com/device`）と認証コードをターミナルに表示してブラウザでログイン。
   - ログイン成功時にアカウント情報をエージェントと紐付けて `.mag/containers/<agent-name>/credentials.json` に安全に保管。
   - ログイン状態およびアカウント一覧を **`agent.md`** に自動記録・更新。
   - ログイン完了後、何個のアカウントがログイン中かを Manager Agent に報告し、エージェントは待機状態（READY / STANDBY）となる。

3. **ログイン済みアカウントに基づく Manager の動的タスク割り振り**
   - ユーザーが `agycli "<タスク内容>"` を実行した際、Manager Agent は `agent.md` およびログイン済みアカウント一覧を確認。
   - **実際にログインしているエージェント群に対して、役割・タスクを適切に割り振る**。

4. **`task.md` による実行詳細記録 & 自己修復ループ**
   - タスク分解時、実行内容の詳細計画を **`task.md`** に書き出す。
   - 各エージェントがタスクを完了するごとに、実行結果・差分・テストログを **`task.md`** に追記。
   - 完了ログを Manager Agent が検証し、不具合があれば修正指示を出して再割り当て・再実行（自己修復ループ）。
   - 最終的に全タスクの実行結果・生成ファイル・検証ステータスを **`task.md`** にまとめてユーザーに報告。

---

## 🏗️ 2. ワークフロー設計

```text
[Step 1: 事前ログイン]
User ──► $ agycli login agent-a
          ├─► agy auth login (URL & Verification Code)
          ├─► ユーザーがブラウザで認証完了
          ├─► .mag/containers/agent-a/credentials.json に保存
          └─► agent.md を自動更新 & Manager に「1アカウント待機中」を報告

[Step 2: タスク実行 & ログ記録]
User ──► $ agycli "Rustで高速なキャッシュサーバーを実装して"
          ├─► Manager: agent.md を確認（ログイン中エージェントを検出）
          ├─► task.md を新規作成・詳細タスク計画を出力
          ├─► ログイン済みエージェントへタスクを割り振って実行
          │     ├─ Agent A (Developer): 実装 ──► task.md にログ追記
          │     ├─ Agent B (Tester): テスト検証 ──► task.md にログ追記
          │     └─ Manager: 完了ログを検証（失敗時は修正再実行）
          └─► 最終結果サマリーを task.md にまとめ、ユーザーへ報告
```
# 追加修正！！！！
```

     ▄▀▀▄
    ▀▀▀▀▀▀
   ▀▀▀▀▀▀▀▀
  ▄▀▀    ▀▀▄
 ▄▀▀      ▀▀▄

 Your browser should open automatically. If not:

 https://accounts.google.com/o/oauth2/auth?access_type=offline&client_id=1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com&code_challen
 ge=ChINlJI5L3cwf-wMdxY4zhetloIjj5lq792H5-tmL2g&code_challenge_method=S256&prompt=consent&redirect_uri=https%3A%2F%2Fantigravity.google%2Foauth-callback&respon
 se_type=code&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcloud-platform+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email+https%3A%2F%2Fwww.googlea
 pis.com%2Fauth%2Fuserinfo.profile+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcclog+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fexperimentsandconfigs+https%3A%2F%
 2Fwww.googleapis.com%2Fauth%2Faicode+openid&state=DadSgkul3RkXfC0lLdec0Q                                                                                      

 Copy and paste the URL or click on the link below:
 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 → Click here to authenticate
 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

 If you aren't automatically redirected, paste the authorization code below:

 authorization code...


  shift+up/down Navigate


```
- こんな感じのURLを転送する形に変更してください！！！

```

     ▄▀▀▄
    ▀▀▀▀▀▀
   ▀▀▀▀▀▀▀▀
  ▄▀▀    ▀▀▄
 ▄▀▀      ▀▀▄

 Welcome to the Antigravity CLI. You are currently not signed in.

 Select login method:
 > 1. Google OAuth
   2. Use a Google Cloud project

 [Use arrow keys to navigate, Enter to select]





```
バックグラウンドで実行します、ここで1をエンターで選択して、でてきたURLをターミナルで表示してユーザーがブラウザで認証する形です．
認証
```

     ▄▀▀▄
    ▀▀▀▀▀▀
   ▀▀▀▀▀▀▀▀
  ▄▀▀    ▀▀▄
 ▄▀▀      ▀▀▄

 Your browser should open automatically. If not:

 https://accounts.google.com/o/oauth2/auth?access_type=offline&client_id=1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com&code_challen
 ge=o_qANMMW9afOKvcghCIX7M12sm1lQe4LYfNjjely5Is&code_challenge_method=S256&prompt=consent&redirect_uri=https%3A%2F%2Fantigravity.google%2Foauth-callback&respon
 se_type=code&scope=https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcloud-platform+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fuserinfo.email+https%3A%2F%2Fwww.googlea
 pis.com%2Fauth%2Fuserinfo.profile+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fcclog+https%3A%2F%2Fwww.googleapis.com%2Fauth%2Fexperimentsandconfigs+https%3A%2F%
 2Fwww.googleapis.com%2Fauth%2Faicode+openid&state=qln7FPSRCn8Ln_HYVptwbw                                                                                      

 Copy and paste the URL or click on the link below:
 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 → Click here to authenticate
 ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────

 If you aren't automatically redirected, paste the authorization code below:

 authorization code...


  shift+up/down Navigate
```

このURLをコピーしてターミナルに表示してください

```
guru@DESKTOP-8S8S33I:~/agyCLI++$ agy

      ▄▀▀▄        Antigravity CLI 1.1.13
     ▀▀▀▀▀▀       koishi040@gmail.com (Google AI Pro)
    ▀▀▀▀▀▀▀▀      Gemini 3.7 Flash (High)
   ▄▀▀    ▀▀▄     ~/agyCLI++
  ▄▀▀      ▀▀▄

────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
>
────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
? for shortcuts 
```
になるのでここは表示しなくていいです