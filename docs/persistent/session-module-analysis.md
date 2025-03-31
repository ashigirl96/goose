<!-- TOC -->
* [Gooseのセッションモジュール解析](#gooseのセッションモジュール解析)
  * [基本構造](#基本構造)
  * [主要コンポーネント](#主要コンポーネント)
    * [Session構造体](#session構造体)
    * [実行モード](#実行モード)
    * [CompletionCache構造体](#completioncache構造体)
  * [主要機能](#主要機能)
    * [セッション管理](#セッション管理)
    * [対話処理](#対話処理)
    * [プランニングモード](#プランニングモード)
    * [例外処理](#例外処理)
    * [コマンド補完](#コマンド補完)
  * [process_agent_responseメソッドの詳細解析](#process_agent_responseメソッドの詳細解析)
    * [1. レスポンスストリームの設定](#1-レスポンスストリームの設定)
    * [2. 非同期メッセージ処理ループ](#2-非同期メッセージ処理ループ)
    * [3. ツール確認リクエスト処理](#3-ツール確認リクエスト処理)
    * [4. 通常メッセージの処理](#4-通常メッセージの処理)
    * [5. エラー処理](#5-エラー処理)
    * [6. 割り込み処理](#6-割り込み処理)
    * [7. 処理の流れと特徴](#7-処理の流れと特徴)
  * [セッションフロー](#セッションフロー)
  * [特筆すべき設計ポイント](#特筆すべき設計ポイント)
<!-- TOC -->


# Gooseのセッションモジュール解析

`crates/goose-cli/src/session/mod.rs`ファイルは、Gooseの対話型セッション管理を担当する中心的なモジュールです。このドキュメントでは、その機能と役割を詳細に解説します。

## 基本構造

このモジュールは、以下のサブモジュールをインポートして機能を実装しています：

- `builder`: セッションの構築と初期化
- `completion`: コマンド補完機能
- `input`: ユーザー入力の処理
- `output`: 出力表示と整形
- `prompt`: プロンプト関連の機能
- `thinking`: 思考表示機能（AIが考えている間の表示）

## 主要コンポーネント

### Session構造体

```rust
pub struct Session {
    agent: Box<dyn Agent>,           // AIエージェント
    messages: Vec<Message>,          // メッセージ履歴
    session_file: PathBuf,           // セッションファイルへのパス
    completion_cache: Arc<std::sync::RwLock<CompletionCache>>, // 補完用キャッシュ
    debug: bool,                     // デバッグモードフラグ
    run_mode: RunMode,               // 実行モード (Normal/Plan)
}
```

### 実行モード

```rust
pub enum RunMode {
    Normal,  // 通常の対話モード
    Plan,    // 計画モード
}
```

### CompletionCache構造体

コマンド補完のキャッシュを管理します：

```rust
struct CompletionCache {
    prompts: HashMap<String, Vec<String>>,
    prompt_info: HashMap<String, output::PromptInfo>,
    last_updated: Instant,
}
```

## 主要機能

### セッション管理

1. **セッション初期化**：
   - 既存のセッションファイルからメッセージを読み込む
   - 新規セッションを作成
   - デバッグモードの設定

2. **拡張機能管理**：
   - `add_extension`: 外部（stdio）拡張機能の追加
   - `add_builtin`: 組み込み拡張機能の追加

3. **プロンプト管理**：
   - `list_prompts`: 利用可能なプロンプトの一覧表示
   - `get_prompt_info`: 特定のプロンプトの詳細情報取得
   - `get_prompt`: プロンプトのメッセージ取得

### 対話処理

1. **メッセージ処理**：
   - `process_message`: 単一メッセージの処理と応答取得
   - `process_agent_response`: エージェントからの応答処理

2. **インタラクティブセッション**：
   - `interactive`: 対話式セッションの実行
   - 入力取得、応答処理、履歴管理を担当
   - コマンド履歴の永続化

3. **非対話モード**：
   - `headless`: 単一メッセージ処理と終了

### プランニングモード

1. **計画生成**：
   - `plan_with_reasoner_model`: 別のAIモデル（リーズナー）を使用した計画生成
   - `classify_planner_response`: 計画かそれとも質問かの判別

2. **計画実行**：
   - ユーザーの確認後、計画に基づいてアクションを実行
   - 必要に応じて設定を一時的に変更

### 例外処理

1. **割り込み処理**：
   - `handle_interrupted_messages`: ツール実行中の中断処理
   - Ctrl+Cによる割り込みへの対応
   - エラー発生時のリカバリ

2. **メッセージ永続化**：
   - セッションファイルへのメッセージ保存
   - 割り込み後の状態復帰

### コマンド補完

1. **補完キャッシュ**：
   - `update_completion_cache`: 補完データの更新
   - `invalidate_completion_cache`: キャッシュの無効化

2. **カスタム補完**：
   - `GooseCompleter`: カスタム補完ヘルパー
   - コンテキスト対応の補完機能

## process_agent_responseメソッドの詳細解析

`process_agent_response`メソッドはGooseの中核となる処理であり、AIエージェントとのやり取りを管理します。このメソッドは以下の重要な役割を果たしています：

### 1. レスポンスストリームの設定

```rust
async fn process_agent_response(&mut self, interactive: bool) -> Result<()> {
    let session_id = session::Identifier::Path(self.session_file.clone());
    let mut stream = self
        .agent
        .reply(
            &self.messages,
            Some(SessionConfig {
                id: session_id,
                working_dir: std::env::current_dir()
                    .expect("failed to get current session working directory"),
            }),
        )
        .await?;
    // ...
}
```

- **セッションID**: セッションファイルのパスをセッション識別子として使用
- **設定情報**: 作業ディレクトリなどの情報をエージェントに提供
- **ストリーミング**: エージェントからの応答をストリームとして取得（チャンク単位で受信）

### 2. 非同期メッセージ処理ループ

```rust
use futures::StreamExt;
loop {
    tokio::select! {
        result = stream.next() => {
            match result {
                Some(Ok(message)) => {
                    // メッセージ処理...
                }
                Some(Err(e)) => {
                    // エラー処理...
                }
                None => break,
            }
        }
        _ = tokio::signal::ctrl_c() => {
            // Ctrl+C割り込み処理...
        }
    }
}
```

- **非同期ストリーム**: `futures::StreamExt`を使用したストリーム処理
- **tokio::select!**: 複数の非同期イベントを同時に待機
  - ストリームからの次のメッセージ
  - Ctrl+C割り込みシグナル
- **マッチング処理**: 受信結果に応じた適切な処理分岐

### 3. ツール確認リクエスト処理

```rust
if let Some(MessageContent::ToolConfirmationRequest(confirmation)) = message.content.first() {
    output::hide_thinking();

    // Format the confirmation prompt
    let prompt = "Goose would like to call the above tool, do you approve?".to_string();

    // Get confirmation from user
    let confirmed = cliclack::confirm(prompt).initial_value(true).interact()?;
    self.agent.handle_confirmation(confirmation.id.clone(), confirmed).await;
}
```

- **確認リクエスト検出**: メッセージの先頭がツール確認リクエストの場合
- **思考表示の非表示化**: ユーザーの確認を求める前に「考え中」表示を無効化
- **ユーザー確認**: `cliclack::confirm`を使用した対話的な確認画面表示
- **確認結果処理**: ユーザーの選択（承認/拒否）をエージェントに通知

### 4. 通常メッセージの処理

```rust
else {
    self.messages.push(message.clone());

    // No need to update description on assistant messages
    session::persist_messages(&self.session_file, &self.messages, None).await?;

    if interactive {output::hide_thinking()};
    output::render_message(&message, self.debug);
    if interactive {output::show_thinking()};
}
```

- **メッセージ追加**: 受信したメッセージを履歴に追加
- **永続化**: メッセージをセッションファイルに保存（description更新なし）
- **思考表示の管理**: 対話モードの場合、表示/非表示を切り替え
- **メッセージレンダリング**: `output::render_message`を使用したメッセージ表示

### 5. エラー処理

```rust
Some(Err(e)) => {
    eprintln!("Error: {}", e);
    drop(stream);
    if let Err(e) = self.handle_interrupted_messages(false).await {
        eprintln!("Error handling interruption: {}", e);
    }
    output::render_error(
        "The error above was an exception we were not able to handle.\n\
        These errors are often related to connection or authentication\n\
        We've removed the conversation up to the most recent user message\n\
        - depending on the error you may be able to continue",
    );
    break;
}
```

- **エラー出力**: エラー内容のコンソール表示
- **ストリーム破棄**: `drop(stream)`でストリームを解放
- **割り込み処理**: `handle_interrupted_messages`メソッドによる状態復旧
- **エラーメッセージ**: ユーザーへの説明メッセージ表示
- **処理終了**: ループを抜けて処理を終了

### 6. 割り込み処理

```rust
_ = tokio::signal::ctrl_c() => {
    drop(stream);
    if let Err(e) = self.handle_interrupted_messages(true).await {
        eprintln!("Error handling interruption: {}", e);
    }
    break;
}
```

- **シグナル検出**: `tokio::signal::ctrl_c()`によるCtrl+C検出
- **ストリーム破棄**: 進行中のストリームを解放
- **割り込み処理**: ユーザー起因の割り込みとして状態を復旧
- **処理終了**: ループを抜けて処理を終了

### 7. 処理の流れと特徴

1. **ストリーミング応答**: 
   - 大きなレスポンスを小さなチャンクで逐次処理
   - ユーザーにリアルタイムフィードバックを提供

2. **並行処理**:
   - tokioのasync/await機能を活用
   - メッセージ受信処理と割り込み検出を並行実行

3. **インタラクティブ制御**:
   - `interactive`フラグによる動作の切り替え
   - 対話モードと非対話モードの両方をサポート

4. **堅牢なエラー処理**:
   - 様々なエラーケースに対応
   - 接続問題や認証エラー時の回復処理

5. **永続化との連携**:
   - レスポンス受信ごとにセッション状態を更新
   - メッセージ履歴の一貫性を維持

このメソッドは、ユーザーとAIエージェント間の通信を管理するだけでなく、ツール確認、エラー処理、割り込み処理など、重要な対話要素を統合して制御する中心的な役割を果たしています。

## セッションフロー

典型的なセッションフローは以下の通りです：

1. セッション初期化（既存ファイルからの読み込みまたは新規作成）
2. インタラクティブループ開始
3. ユーザー入力の取得と解析
4. 入力タイプに応じた処理：
   - メッセージ：AIエージェントに送信して応答を処理
   - コマンド：拡張機能追加、テーマ切り替えなどの操作
   - プロンプト：特定のプロンプトの実行
   - プランニング：計画モードでの対話
5. 結果の表示と履歴の更新
6. ループ継続（終了コマンドまで）

## 特筆すべき設計ポイント

1. **モジュラー設計**：
   - 機能ごとに分離されたサブモジュール
   - 関心の分離による保守性の向上

2. **拡張性**：
   - 動的な拡張機能の追加サポート
   - プラグイン的なアーキテクチャ

3. **エラー耐性**：
   - 例外的な状況への対応
   - 割り込み処理とリカバリ機能

4. **状態永続化**：
   - メッセージ履歴の保存と復元
   - コマンド履歴のグローバル管理

5. **ユーザーエクスペリエンス**：
   - カスタマイズ可能なテーマ
   - コマンド補完機能
   - 「考え中」表示

6. **マルチモードサポート**：
   - 通常対話モードとプランニングモード
   - ヘッドレスモード（非対話）

以上の分析から、`session/mod.rs`はGooseの対話型インターフェースの中心的なコンポーネントであり、ユーザー入力の処理からAIエージェントとの通信、結果の表示、そして状態管理までを担当していることがわかります。また、拡張性と堅牢性を考慮した設計により、様々な使用シナリオに対応できる柔軟な構造となっています。