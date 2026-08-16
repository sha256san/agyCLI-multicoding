# Multi-Agent Development Orchestrator (`mag` / `agycli`) 第5次拡張仕様書 (addplan5.md)

本ドキュメントは、**`agy` (Antigravity CLI) ネイティブの対話型インタラクティブターミナル（REPL/TUI）モード**、**AGY標準スラッシュコマンド群**、および **リアルタイム対話型マルチエージェントオーケストレーション** の詳細設計仕様書です。

---

## 🎯 1. 主要要件一覧

1. **`agy` 互換の対話型ターミナル REPL / TUI モード**
   - 引数なしで `agycli` または `mag` を起動した際、終了せず `agy` 本来のリッチな対話型プロンプト（`agycli ❯ `）を起動。
   - 美しいバナー、現在のログインユーザー情報、アクティブワーカー数、ワークスペースパスを表示。

2. **AGY 標準スラッシュコマンド (`/command`) の完全サポート**
   - `/help`: コマンド一覧・使用方法ヘルプ
   - `/status`: エージェント稼働状態、コンテナ認証、タスク統計の表示
   - `/doctor`: システム環境診断（`EnvDoctor`）
   - `/login [target]`: Google アカウントおよびコンテナ個別ログイン
   - `/whoami [container]`: ログイン中ユーザー情報の確認
   - `/workers [N]`: ワーカーコンテナ数の動的スケーリング
   - `/tasks`: タスク履歴・ステータス一覧
   - `/clear`: 画面クリア
   - `/exit` または `/quit`: セッション終了

3. **自然言語プロンプトの対話型ストリーミング実行**
   - プロンプト入力時（例: `/home/guru/agytest にrustのプログラムを書いて`）、5-Agent DAG 自律開発ループをリアルタイムに実行。
   - 完了後、自動 Git `main` マージを行い、再度対話型プロンプト（`agycli ❯ `）へ復帰。

4. **単発コマンド・サブコマンドとの完全互換性**
   - `agycli status` や `agycli login "agent-a"` などの非対話コマンドも従来通り即座に実行可能。

---

## 🏗️ 2. インタラクティブ画面設計

```text
  ___         _   _                    _ _           
 / _ \       | | (_)                  (_) |          
/ /_\ \ _ __ | |_ _  __ _ _ __ __ ___  _| |_ _   _   
|  _  || '_ \| __| |/ _` | '__/ _` \ \/ / | __| | | |  
| | | || | | | |_| | (_| | | | (_| |>  <| | |_| |_| |  
\_| |_/|_| |_|\__|_|\__, |_|  \__,_/_/\_\_|\__|\__, |  
                     __/ |                      __/ |  
                    |___/                      |___/   
 Multi-Agent Software Development Orchestrator (`agycli` v0.2.2)

 Workspace: /home/guru/agyCLI++
 User:      developer@google.com (Google Developer)
 Workers:   5 active agents (developer, tester, reviewer, security, researcher)
 Status:    READY  |  Type /help for slash commands or enter your instruction.

agycli ❯ 
```
