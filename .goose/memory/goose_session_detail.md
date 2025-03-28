# goose session design architecture
# Gooseのセッション設計の解析

Gooseの`crates/goose-cli/src/session`ディレクトリを調査した結果、以下のように設計されていることがわかりました。

## 1. 全体的な構造

Gooseのセッション管理は、以下のファイルによって実装されています：

- **mod.rs**: セッションの中心的なコードを含むメインモジュール
- **builder.rs**: セッションの構築と初期化を担当
- **input.rs**: ユーザー入力の処理とコマンド解析
- **output.rs**: 出力表示と整形
- **completion.rs**: コマンド補完機能
- **prompt.rs**: プロンプト関連の機能
- **thinking.rs**: 思考表示機能

## 2. Session構造体

`Session`構造体は、セッション管理の中心として以下の重要なフィールドを持っています：

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

## 3. 主要な機能

### セッションの作成と再開

`build_session`関数はセッションの構築を担当します：
- 設定からプロバイダーとモデルを取得
- 新規セッションの作成または既存セッションの再開
- 拡張機能の初期化
- システムプロンプトの設定

### インタラクティブモード

`interactive`メソッドは、対話式セッションを実行し：
- コマンド履歴の管理
- ユーザー入力の処理
- エージェント応答の処理
- スラッシュコマンドの処理

### 入力処理

`input.rs`では、様々なタイプのユーザー入力を処理します：
- 通常のテキストメッセージ
- スラッシュコマンド（`/exit`, `/t`, `/extension`など）
- プロンプトコマンド
- プランモードコマンド

```rust
pub enum InputResult {
    Message(String),
    Exit,
    AddExtension(String),
    AddBuiltin(String),
    ToggleTheme,
    Retry,
    ListPrompts(Option<String>),
    PromptCommand(PromptCommandOptions),
    GooseMode(String),
    Plan(PlanCommandOptions),
    EndPlan,
}
```

### 出力表示

`output.rs`では、様々な出力形式の表示を管理します：
- Markdownのレンダリング
- ツールリクエストとレスポンスの表示
- エラーメッセージの表示
- テーマの切り替え（ライト/ダーク/ANSIモード）
- 「考え中」スピナーの表示

### 拡張機能の管理

セッションは複数の拡張機能をサポートしています：
- 組み込み拡張機能の追加
- 外部（stdio）拡張機能の追加
- プロンプト拡張機能

### プランニングモード

新しく追加された「プラン」モードがあり：
- ユーザーが計画を立てるための指示を提供
- モデルが計画または明確化質問を生成
- 確認後、計画に基づいて行動可能

## 4. 設計ポイント

### 永続化

メッセージ履歴はファイルに永続化され、セッションの中断と再開をサポートします：
```rust
session::persist_messages(&self.session_file, &self.messages, Some(provider)).await?;
```

### エラー処理

ツール実行中の中断や例外的なエラーに対処する機能を持っています：
```rust
async fn handle_interrupted_messages(&mut self, interrupt: bool) -> Result<()> {
    // エラー処理ロジック
}
```

### テーマサポート

ライト/ダーク/ANSIの3つのテーマをサポートし、ユーザーが切り替え可能です：
```rust
pub enum Theme {
    Light,
    Dark,
    Ansi,
}
```

### コマンド補完

ユーザー入力のコンテキスト対応補完機能を提供します：
```rust
let completer = GooseCompleter::new(self.completion_cache.clone());
editor.set_helper(Some(completer));
```

## 5. フロー制御

1. ユーザーが入力
2. 入力を解析して適切なコマンドに変換
3. コマンドに応じた処理を実行
4. 必要に応じてAIエージェントに処理を委譲
5. 結果を整形して表示
6. メッセージ履歴を更新・永続化

## まとめ

Gooseのセッション設計は、コマンドライン中心の対話型AIエージェントとして、強力な拡張性、柔軟な入出力処理、エラー耐性、そして高いカスタマイズ性を持つように設計されています。特にスラッシュコマンドと拡張機能のサポートにより、基本的なチャット機能を超えた多様な操作が可能になっています。
