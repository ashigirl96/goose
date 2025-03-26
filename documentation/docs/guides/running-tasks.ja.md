---
sidebar_position: 7
---
# タスクの実行

Goose CLIを使用する際、`goose run`コマンドにファイルや指示を渡してタスクやワークフローを実行できます。これは単純な一行コマンドでも、ファイルに保存された複雑な指示のセットでも可能です。

## 基本的な使い方

`goose run`コマンドは新しいセッションを開始し、提供された引数を使用して実行を開始し、タスクが完了すると自動的にセッションを終了します。

Gooseでタスクを実行するには複数の方法があります。[オプションの一覧](/docs/guides/goose-cli-commands.md#run-options)をご覧ください。

### コマンドでテキストを指定する
```bash
goose run -t "ここに指示を入力"
```

`-t`フラグを使用すると、コマンドに直接テキスト指示を渡すことができます。これは、Gooseとのインタラクティブなセッションを必要としない、素早い一回限りのコマンドに最適です。指示が実行され、セッションは終了します。CI/CDパイプラインでの使用や、他のスクリプトと一緒に実行するなどの例が考えられます。

### 指示ファイルを使用する
複雑な指示やオートメーション化したいワークフローがある場合、それらをファイルに保存して`goose run`コマンドに渡すことができます：

```bash
goose run -i instructions.md
```

プロジェクト依存関係のセキュリティ監査を実行する指示ファイルの例：

```md
# 依存関係セキュリティ監査

1. プロジェクト依存関係の分析：
   - package.jsonとrequirements.txtファイルを確認
   - バージョン付きですべての依存関係をリスト
   - 古いパッケージを特定

2. セキュリティチェック：
   - npm audit（JavaScriptパッケージ用）を実行
   - Pythonパッケージの既知の脆弱性をチェック
   - 重大なセキュリティ問題がある依存関係を特定

3. アップグレード計画の作成：
   - 即時更新が必要なパッケージのリスト
   - 最新バージョンでの破壊的変更の注記
   - 必要な更新の影響の見積もり

重要度レベルを強調した結果を「security_audit.md」に保存。
```

### 標準入力を使用する
`-i -`を使用して標準入力から指示をGooseに渡すこともできます。これは他のツールやスクリプトからコマンドをGooseにパイプしたい場合に便利です。

#### シンプルなエコーパイプ

```bash
echo "2+2はいくつですか？" | goose run -i -
```

#### 複数行の指示
```bash
cat << EOF | goose run -i -
以下のタスクを手伝ってください：
1. 85の15%を計算する
2. 32°Cを華氏に変換する
EOF
```

## 主な機能

### インタラクティブモード

タスク終了時にGooseを終了させたくない場合は、`-s`または`--interactive`フラグを渡して、初期コマンドの処理後にインタラクティブセッションを開始できます：

```bash
goose run -i instructions.txt -s
```

これは、初期コマンドが処理された後もGooseでの作業を続けたい場合に便利です。

### セッション管理

セッションに名前を付けて管理できます：

```bash
# 名前付きの新しいセッションを開始
goose run -n my-project -t "初期指示"

# 前のセッションを再開
goose run -n my-project -r
```

### 拡張機能の使用

タスク実行時に特定の拡張機能が利用可能であることを確認したい場合は、引数でこれを指定できます。これは`--with-extension`または`--with-builtin`フラグを使用して行えます：

- 組み込み拡張機能の使用例（developerおよびcomputercontroller拡張機能）

```bash
goose run --with-builtin "developer,computercontroller" -t "あなたの指示"
```

- カスタム拡張機能の使用

```bash
goose run --with-extension "ENV1=value1 custom-extension-args" -t "あなたの指示"
```

## 一般的なユースケース

### スクリプトファイルの実行

指示ファイル（例：`build-script.txt`）を作成：
```text
現在のブランチを確認
テストスイートを実行
ドキュメントをビルド
```

そして実行：
```bash
goose run -i build-script.txt
```

### クイックコマンド

一回限りのコマンドにはテキストオプションを使用：
```bash
goose run -t "現在のgitブランチとmainを比較するCHANGELOG.mdエントリを作成"
```

### 開発ワークフロー

特定の拡張機能を使用してセッションを開始：
```bash
goose run --with-builtin "developer,git" -n dev-session -s
```

### オプションの組み合わせ

複数のオプションを組み合わせて強力なワークフローを作成できます：

```bash
# 複数のオプションを組み合わせた複雑な例
goose run \
  --with-builtin "developer,git" \
  --with-extension "API_KEY=xyz123 custom-tool" \
  -n project-setup \
  -t "プロジェクトを初期化" 
```

このコマンドは：
1. developerとgitの組み込み拡張機能を読み込む
2. APIキーを持つカスタム拡張機能を追加
3. セッションに「project-setup」という名前を付ける
4. 「プロジェクトを初期化」という指示で開始
5. コマンドの処理後、自動的に終了