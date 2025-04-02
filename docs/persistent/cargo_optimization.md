# Cargoビルド設定の最適化

このドキュメントでは、Goose開発用のCargoビルド設定を最適化する方法を紹介します。これらの設定を適用することで、開発中のビルド時間を短縮できます。

## Cargo設定ファイル

Cargoの設定ファイルを使用すると、ビルドプロセスをカスタマイズできます。以下の設定例を参考に設定ファイルを作成してください。

### 1. グローバル設定 (`~/.cargo/config.toml`)

このファイルはすべてのRustプロジェクトに影響します。システム全体の設定としておすすめです。

```toml
[build]
# インクリメンタルコンパイルを有効化（再ビルド時間の短縮）
incremental = true

# 並列ジョブ数（CPUコア数に応じて調整）
jobs = 8

# ターゲットディレクトリをSSDに配置（オプション）
# target-dir = "/path/to/ssd/rust-target"

# コンパイルキャッシュツールを使用（事前にインストールが必要）
# rustc-wrapper = "sccache"

[profile.dev]
# 最小限の最適化（0:なし、1:最小限）- トレードオフによっては1にする価値あり
opt-level = 0

# デバッグ情報を含める
debug = true
```

### 2. プロジェクト固有の設定 (`.cargo/config.toml`)

プロジェクトのルートディレクトリに`.cargo`フォルダを作成し、その中に`config.toml`を配置することで、Gooseプロジェクト固有の設定を追加できます。

```toml
[build]
# Goose固有の設定

# デバッグビルドのみでLLVM時間計測を有効化（オプション）
# rustflags = ["-Ztime-passes"] # 要 nightly

# ビルド統計情報の収集（オプション）
# rustflags = ["-Ztime-passes", "-Zinstrument-coverage"]

[profile.dev.package."*"]
# 依存パッケージは最小限の最適化
opt-level = 1

# 特定のパッケージはより最適化
[profile.dev.package.tokenizers]
opt-level = 2

[profile.dev.package.regex]
opt-level = 2
```

### 3. プロジェクトの`Cargo.toml`に追加する設定

Gooseプロジェクトのルート`Cargo.toml`に以下のセクションを追加することで、開発ビルドを最適化できます。

```toml
# Cargo.tomlに追加

[profile.dev]
# 基本的な最適化を有効化
opt-level = 1

# デバッグ情報を含める
debug = true 

# リンク時最適化を無効化
lto = false

# コード生成ユニット数を増やして並列化
codegen-units = 256

[profile.dev.package."*"]
# 依存クレートは若干最適化
opt-level = 1

# 開発用の軽量プロファイル
[profile.dev-fast]
inherits = "dev"
opt-level = 0
debug = false
split-debuginfo = "unpacked"
debug-assertions = false
overflow-checks = false
incremental = true
codegen-units = 256
```

## コンパイルキャッシュツール（オプション）

Sccacheなどのコンパイルキャッシュツールを使用すると、ビルド時間をさらに短縮できます。

```bash
# sccacheのインストール
cargo install sccache

# 環境変数で有効化
export RUSTC_WRAPPER=sccache

# キャッシュのステータス確認
sccache --show-stats
```

## ビルド時間の計測

ビルド最適化の効果を測定するために、以下のコマンドでビルド時間を計測できます：

```bash
# リリースビルドの時間計測
time just release-binary

# デバッグビルドの時間計測
time just run-dev
```

## まとめ

適切な設定を行うことで、Gooseの開発サイクルを効率化し、より素早くフィードバックを得ることができます。最適な設定は開発環境やプロジェクトの規模によって異なるため、いくつかの設定を試して最適なものを見つけることをおすすめします。