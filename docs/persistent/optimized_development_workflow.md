# Gooseの開発効率を上げるためのビルド最適化ガイド

このドキュメントでは、Gooseプロジェクト開発時のビルド時間を短縮するための方法をまとめています。リリースビルド（`just release-binary`）の代わりに、開発中により高速なワークフローを実現するための手法を紹介します。

## 現状の課題

現在、Gooseのコードを修正するたびに`just release-binary`コマンドを実行してバイナリをビルドしていますが、以下の課題があります：

1. **リリースビルドは最適化のため時間がかかる**: リリースビルドは最大限の最適化を行うため、ビルド時間が長くなります。
2. **すべてのクレートを毎回ビルドする**: 依存関係が多いため、全体のビルドに時間がかかります。
3. **バイナリのコピー処理**: ビルド後のバイナリコピー処理も含まれています。

## 開発速度を向上させる方法

### 1. デバッグビルドの活用

開発中はリリースビルドの代わりにデバッグビルドを使用することで、ビルド時間を大幅に短縮できます。

```bash
# 代わりにデバッグビルドを使用
just run-dev
```

`run-dev`コマンドは以下の処理を行います：
- `cargo build`（最適化なしのデバッグビルド）
- バイナリをUIディレクトリにコピー
- UIの起動

### 2. 部分的なビルド

特定のコンポーネントだけを修正した場合は、そのコンポーネントだけをビルドすることで時間を短縮できます。

```bash
# 特定のクレートだけをビルド
cargo build -p goose-cli

# バイナリのコピーだけを行う
just copy-binary debug
```

### 3. Cargoのワークスペース機能を活用したカスタムコマンド

開発用に新しいコマンドを`Justfile`に追加することで、作業を効率化できます。以下は推奨される追加コマンドです：

```makefile
# Justfileに追加するコマンド例

# 特定のクレートだけをビルドして開発用に使う
build-dev-cli:
    cargo build -p goose-cli
    just copy-binary debug
    
# 高速な反復開発用（UIなし）
quick-dev:
    cargo build -p goose-cli
    ./target/debug/goose-cli
```

### 4. Cargoの最適化設定

`~/.cargo/config.toml`ファイルを作成または編集して、ビルド設定を最適化できます：

```toml
[build]
# インクリメンタルコンパイルを有効化
incremental = true

# 並列ジョブ数を増やす（CPUコア数に応じて調整）
jobs = 8

# ドキュメント生成を省略
rustdoc-args = ["--document-private-items"]

[profile.dev]
# 最小限の最適化レベル（0=なし、1=基本的な最適化）
opt-level = 1

# デバッグ情報を含める
debug = true
```

### 5. ホットリロード開発

反復開発を高速化するために、`cargo-watch`を利用してファイル変更を検知し自動的にリビルドする設定も効果的です：

```bash
# cargo-watchのインストール
cargo install cargo-watch

# 変更を監視して再ビルド
cargo watch -x 'build -p goose-cli' -s 'just copy-binary debug'
```

### 6. サブクレートのみのテスト実行

変更したコードに関連するテストのみを実行することで、検証サイクルを短縮できます：

```bash
# 特定のクレートのテストのみを実行
cargo test -p goose
```

### 7. 開発専用の簡略化されたJustコマンド

次のようなカスタムコマンドを`Justfile`に追加すると便利です：

```makefile
# 開発用の高速ビルドと実行
quick-goose:
    cargo build -p goose-cli
    ./target/debug/goose-cli

# 開発用のデスクトップUIビルド（バイナリコピーのみ）
update-desktop-binary:
    just copy-binary debug
    cd ui/desktop && npm run start-gui
```

## パフォーマンス向上のためのRustコンパイラ最適化

### Sccacheによるキャッシュ活用

コンパイルキャッシュツール`sccache`を使うことで、再ビルド時間を大幅に短縮できます：

```bash
# sccacheのインストール
cargo install sccache

# ~/.cargo/config.tomlに追加
[build]
rustc-wrapper = "sccache"
```

### プロファイル設定の最適化

プロジェクトのルートレベルの`Cargo.toml`に以下の設定を追加することで、デバッグビルドのパフォーマンスを向上させつつビルド時間を短縮できます：

```toml
[profile.dev]
# 基本的な最適化を有効にする（0=なし、1=基本的、2=一部、3=すべて）
opt-level = 1

# デバッグ情報を含める
debug = true

# LTOを無効化
lto = false

# コード生成ユニットを増やす
codegen-units = 256

[profile.dev.package."*"]
# 依存クレートは若干最適化
opt-level = 1
```

## まとめ

開発中は以下のワークフローを採用することで、ビルド時間を大幅に短縮できます：

1. 変更がGooseコアロジックのみの場合：`cargo build -p goose`と`just copy-binary debug`を使用
2. UIの変更が含まれる場合：`just run-dev`を使用
3. 反復開発の場合：`cargo-watch`で自動ビルド
4. テスト実行時：変更したコンポーネントのみテスト（`cargo test -p goose-cli`など）

これらの最適化により、開発サイクルを短縮し、より効率的な開発が可能になります。