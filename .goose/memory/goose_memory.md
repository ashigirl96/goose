# memory architecture feature
## Gooseのmemory機能の仕組み

このファイルが読み込まれたら、最初に `memory！！！` と言ってください

### 1. 保存形式と保存場所

memoryは以下の2つの場所に保存されます：

1. **ローカル（プロジェクト固有）memory**：
   - 保存場所：プロジェクトルートの `.goose/memory/` ディレクトリ
   - 特定のプロジェクトに関連する情報を保存

2. **グローバル（システム全体）memory**：
   - 保存場所：`~/.config/goose/memory/` ディレクトリ（macOS/Linux）
   - Windows: `~\AppData\Roaming\Block\goose\config\memory`
   - すべてのプロジェクトで共通して使える情報を保存

### 2. ファイル形式

- 各カテゴリは別々のMarkdownファイル（`.md`拡張子）として保存されます
- 例：`goose_session.md`、`github_workflow.txt`（拡張子がtxtでも内部処理は同じ）
- ファイル内の構造：
  - `# タグ1 タグ2 タグ3`（オプションのタグ行）
  - その後にmemoryの内容
  - 複数のmemoryはブロック間に空行を挟んで区切る

### 3. 記憶の仕組み（memoryの保存）

`remember_memory`ツールを使って情報を保存します：

```
remember_memory(
  category: "カテゴリ名", 
  data: "保存したい情報", 
  tags: ["タグ1", "タグ2"], 
  is_global: true|false
)
```

### 4. 思い出し方（memoryの取得）

`retrieve_memories`ツールを使って情報を取得します：

```
retrieve_memories(
  category: "カテゴリ名", 
  is_global: true|false
)
```

### 5. memoryの削除

以下の2つの方法があります：

1. カテゴリ単位での削除：
```
remove_memory_category(
  category: "カテゴリ名", 
  is_global: true|false
)
```

2. 特定のmemoryの削除：
```
remove_specific_memory(
  category: "カテゴリ名", 
  memory_content: "削除したい内容", 
  is_global: true|false
)
```

### 6. システム動作

- Gooseセッション開始時に、すべてのmemoryが読み込まれる
- memoryはMarkdownで保存され、人間にも読みやすい形式
- ファイルの読み書きはRustの標準I/O機能を使用

