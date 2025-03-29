# Gooseのデバッグガイド

Gooseアプリケーションでのデバッグを効果的に行うための包括的なガイドです。

## デバッグ出力方法

Gooseセッションでデバッグ出力を行う場合、`println!`マクロは表示されません。代わりに以下のログシステムを使用します。

### tracingフレームワークの使用

Gooseは`tracing`クレートを使用しています。このフレームワークは構造化ロギングを提供します。

```rust
use tracing::{trace, debug, info, warn, error};

// 各レベルのログ出力例
trace!("最も詳細なトレース情報: {}", 値);
debug!("デバッグメッセージ: {}", 値);
info!("情報メッセージ");
warn!("警告メッセージ");
error!("エラーメッセージ");
```

### ログレベルの優先順位

1. `error!` - 最も重要な問題（常に表示）
2. `warn!` - 警告情報
3. `info!` - 一般的な情報
4. `debug!` - デバッグ情報
5. `trace!` - 最も詳細な情報

## ログファイルの場所

Gooseはログファイルを以下の場所に保存します：

- **macOS/Linux**: `~/.local/state/goose/logs/cli/YYYY-MM-DD/YYYYMMDD_HHMMSS-session_name.log`
- **Windows**: `~\AppData\Roaming\Block\goose\data\logs\cli\YYYY-MM-DD\YYYYMMDD_HHMMSS-session_name.log`

## デバッグ方法

### 1. コマンドラインフラグの使用

`--debug`フラグを使用すると、より詳細なログが表示されます：

```bash
goose session --debug
```

### 2. 環境変数でログレベルを設定

```bash
# デバッグレベルのログを表示
RUST_LOG=debug goose session

# より詳細なトレースレベルのログを表示
RUST_LOG=trace goose session

# 特定のモジュールのみログレベルを変更
RUST_LOG=goose=debug,goose_cli=trace goose session
```

### 3. ログファイルの確認

最新のログファイルを表示する例：

```bash
# 日付のディレクトリ内の最新のログファイルを表示
cat ~/.local/state/goose/logs/cli/$(date +%Y-%m-%d)/$(ls -t ~/.local/state/goose/logs/cli/$(date +%Y-%m-%d) | head -1)
```

### 4. ログ設定の詳細

Gooseのログ設定は`crates/goose-cli/src/logging.rs`で定義されています。デフォルトでは以下の設定になっています：

- 詳細なログ（DEBUG以上）はJSONフォーマットでファイルに保存
- コンソールには警告（WARN）以上のみ表示
- デバッグモード時には追加情報が表示される

## デバッグのヒント

### なぜprintln!が表示されないか

- **出力リダイレクト**: Gooseは対話型インターフェイスのために標準出力をカスタム処理に置き換えています
- **非同期処理**: 非同期ストリーム処理のため、タイミングによっては出力が失われる可能性があります
- **意図的な抑制**: ユーザー体験を妨げないよう設計されています

### 効果的なデバッグの方法

1. 適切なログレベルを使用する（`debug!`は開発中、`info!`は一般情報）
2. 構造化されたデータをログに含める
3. コンテキスト情報を十分に提供する
4. エラー状況を詳細に記録する

## 実装例

セッション内でのデバッグの実装例：

```rust
use anyhow::Result;
use tracing::{debug, info, warn, error};

async fn process_function() -> Result<()> {
    // デバッグ情報を記録
    debug!("関数が開始されました");
    
    let some_value = 42;
    info!("処理中の値: {}", some_value);
    
    if some_value > 100 {
        warn!("値が予想より大きいです: {}", some_value);
    }
    
    match some_operation() {
        Ok(result) => {
            debug!("操作結果: {:?}", result);
            Ok(())
        },
        Err(e) => {
            error!("エラーが発生しました: {}", e);
            Err(e.into())
        }
    }
}
```

## 高度なデバッグテクニック

### 条件付きコンパイル

```rust
// デバッグビルド時のみコンパイルされるコード
#[cfg(debug_assertions)]
debug!("デバッグビルドでのみ表示されるメッセージ");
```

### スパンの使用

より構造化されたログのために`tracing`のスパンを使用します：

```rust
use tracing::{info, info_span};

fn complex_operation() {
    // この操作に関連するすべてのログがグループ化される
    let span = info_span!("complex_operation", operation_id = 123);
    let _enter = span.enter();
    
    info!("操作を開始します");
    // 処理コード
    info!("操作を完了しました");
}
```

### Gooseモードとデバッグの組み合わせ

```bash
# 自動モードでデバッグ出力を有効にする
GOOSE_MODE=auto goose session --debug
```