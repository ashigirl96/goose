# Goose開発用クイックビルドコマンド

このドキュメントでは、既存のJustfileに開発専用の高速ビルドコマンドを追加する方法を紹介します。これにより、開発中のリリースビルド（`just release-binary`）の代わりに、より素早いフィードバックサイクルを実現できます。

## 追加するJustコマンド

以下のコマンドを既存の`Justfile`にコピー＆ペーストすることで、すぐに開発効率を向上させることができます。

```makefile
# 開発用の高速ビルド（デバッグビルド＋バイナリコピー）
dev-binary:
    @echo "Building debug version for development..."
    cargo build -p goose-cli
    @just copy-binary debug
    @echo "Debug binary ready for development!"

# 特定のクレートのみビルド＋実行（UIなし）
run-cli-only:
    @echo "Building and running CLI only..."
    cargo build -p goose-cli
    ./target/debug/goose-cli
    
# 変更監視による自動ビルド
watch-build:
    @echo "Watching for changes and rebuilding..."
    cargo watch -x 'build -p goose-cli' -s 'just copy-binary debug'
    
# デスクトップUI開発用（既存バイナリ使用）
run-ui-only:
    @echo "Starting UI with existing binary..."
    cd ui/desktop && npm run start-gui
```

## 使用方法

1. **開発中の高速ビルド**:
   ```bash
   just dev-binary
   ```
   これは通常の`just release-binary`の代わりに使用し、最適化の少ないデバッグビルドを行うことで、ビルド時間を短縮します。

2. **CLIのみの迅速なテスト**:
   ```bash
   just run-cli-only
   ```
   UIを起動せず、コマンドラインインタフェースのみを素早くテストしたい場合に使用します。

3. **変更の自動検知と再ビルド**:
   ```bash
   just watch-build
   ```
   ソースコードを変更すると自動的に再ビルドされるため、繰り返し手動でビルドコマンドを実行する必要がなくなります。
   ※この機能を使用するには事前に `cargo install cargo-watch` が必要です。

4. **UIのみの起動**:
   ```bash
   just run-ui-only
   ```
   バックエンドのコードに変更がなく、UIのみを変更・テストする場合に使用します。

## 前提条件

- `cargo-watch`を使用する場合は、事前にインストールが必要です:
  ```bash
  cargo install cargo-watch
  ```

## 開発時のおすすめワークフロー

1. バックエンドコードの修正中は `just dev-binary` または `just watch-build` を使用
2. UI開発中は `just run-ui-only` を使用
3. 最終テストやリリース準備時のみ `just release-binary` を使用

このような分離されたビルドプロセスにより、開発時のフィードバックループを短縮し、生産性を向上させることができます。