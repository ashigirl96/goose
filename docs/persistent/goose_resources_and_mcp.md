<!-- TOC -->
* [Gooseのリソースシステムとプラットフォームツール - MCPアーキテクチャ](#gooseのリソースシステムとプラットフォームツール---mcpアーキテクチャ)
  * [MCPアーキテクチャの概要](#mcpアーキテクチャの概要)
    * [MCP通信フロー](#mcp通信フロー)
    * [MCPサーバーの実装方法](#mcpサーバーの実装方法)
  * [リソースシステムの概要](#リソースシステムの概要)
  * [リソースの構造](#リソースの構造)
  * [ツール選択プロセス](#ツール選択プロセス)
    * [ステージ1: LLMベースのツール選択](#ステージ1-llmベースのツール選択)
    * [ステージ2: MCPクライアント/ツール解決](#ステージ2-mcpクライアントツール解決)
  * [プラットフォームツール](#プラットフォームツール)
    * [platform__read_resource](#platform__read_resource)
    * [platform__list_resources](#platform__list_resources)
  * [MCPとリソースシステムの関係](#mcpとリソースシステムの関係)
    * [MCPサーバーのリソース機能実装](#mcpサーバーのリソース機能実装)
    * [リソース機能の処理フロー](#リソース機能の処理フロー)
  * [リソースシステムの使用例](#リソースシステムの使用例)
    * [アクティブな拡張機能のリソース一覧取得](#アクティブな拡張機能のリソース一覧取得)
    * [特定の拡張機能のリソース一覧取得](#特定の拡張機能のリソース一覧取得)
    * [ファイルリソースの読み取り](#ファイルリソースの読み取り)
  * [処理フロー例](#処理フロー例)
  * [リソース対応拡張機能の例](#リソース対応拡張機能の例)
  * [システムプロンプトでの表示](#システムプロンプトでの表示)
  * [リソースの優先度とアクティブリソース](#リソースの優先度とアクティブリソース)
  * [MCPアーキテクチャの利点](#mcpアーキテクチャの利点)
  * [まとめ](#まとめ)
<!-- TOC -->

# Gooseのリソースシステムとプラットフォームツール - MCPアーキテクチャ

このドキュメントでは、Gooseのリソースシステムとモデルコンテキストプロトコル（Model Context Protocol、MCP）アーキテクチャについて詳細に説明します。

## MCPアーキテクチャの概要

モデルコンテキストプロトコル（Model Context Protocol、MCP）は、Gooseのメインアプリケーションと様々な拡張機能間の通信を可能にする基本コンポーネントです。MCPは以下のような標準化された方法を提供します：

1. 拡張機能のケイパビリティをツールとして定義する
2. ユーザーの命令を処理して、適切な拡張機能とツールを選択する
3. ツール呼び出しを実行し、結果を会話フローに戻す

### MCP通信フロー

MCPアーキテクチャはクライアント・サーバーモデルに従っています：

1. **MCPサーバー**: 各拡張機能はMCPサーバーとして実行され、ツールと機能を提供します
2. **MCPクライアント**: Gooseのメインアプリケーションは各拡張機能へのクライアント接続を維持します
3. **JSON-RPCプロトコル**: クライアントとサーバー間の通信はJSON-RPCメッセージを使用します

```
ユーザー入力 → Gooseコア → LLM → ツール選択 → MCPクライアント → MCPサーバー（拡張機能） → ツール実行
```

### MCPサーバーの実装方法

MCPサーバーは次の3つの方法で実装できます：

1. **組み込み型**: Gooseバイナリに埋め込まれた拡張機能
   - 別プロセスとして実行されますが、同じ実行可能ファイルを使用します
   - 標準入出力を介した通信

2. **標準入出力型**: 標準入出力を介して通信する外部拡張機能
   - どの言語でも実装可能
   - 子プロセスとして実行

3. **サーバー送信イベント（SSE）型**: RESTful HTTPエンドポイント
   - 異なるマシンで実行できるリモート拡張機能
   - HTTPストリーミングを介した通信

## リソースシステムの概要

Gooseのリソースシステムは、拡張機能が様々なデータをLLMに提供するための統一的な仕組みです。これにより、以下のような情報を簡単にLLMのコンテキストに取り込むことができます：

- ファイルの内容
- データベースのスキーマ
- アプリケーション固有の情報
- その他の構造化データ

リソースは、拡張機能と言語モデル間の情報交換の基本単位です。

## リソースの構造

各リソースは以下の主要な構成要素を持ちます：

```rust
pub struct Resource {
    /// URI representing the resource location (e.g., "file:///path/to/file" or "str:///content")
    pub uri: String,
    /// Name of the resource
    pub name: String,
    /// Optional description of the resource
    pub description: Option<String>,
    /// MIME type of the resource content ("text" or "blob")
    pub mime_type: String,
    pub annotations: Option<Annotations>,
}
```

- **URI**: リソースの場所を示す一意の識別子
- **名前**: リソースの名前（表示用）
- **MIMEタイプ**: リソースの種類（主に「text」または「blob」）
- **優先度**: リソースの重要性（0.0〜1.0の値、annotationsに含まれる）
- **タイムスタンプ**: リソースの最終更新時刻（annotationsに含まれる）

## ツール選択プロセス

ユーザーがGooseに命令を出すと、システムは拡張機能とツールを決定するために2段階のプロセスを実行します：

### ステージ1: LLMベースのツール選択

1. **ツール情報収集**:
   - Gooseは`capabilities.get_prefixed_tools()`を通じて、すべてのアクティブな拡張機能から利用可能なツールを収集します
   - 各ツールには拡張機能名が接頭辞として付けられます（例: `developer__shell`）
   - ツールには名前、説明、パラメータスキーマが含まれます

2. **LLM処理**:
   - ユーザーの命令、システムプロンプト、ツール情報は`provider().complete(...)`を通じてLLMに渡されます
   - LLMは命令を分析し、最も適切なツールを決定します
   - プロバイダー固有のフォーマットコンバーター（例: `google.rs`, `openai.rs`）がAPI固有のフォーマットを処理します

3. **レスポンス解析**:
   - LLMのレスポンスはプロバイダー固有のパーサー（例: `response_to_message()`）によって処理されます
   - LLMがツールを使用すると判断した場合、そのレスポンスにツール呼び出しが含まれます
   - これはツール名と引数を持つ`MessageContent::ToolRequest`に変換されます

LLMのツール呼び出しを変換するコード例（Google AI）:

```rust
if let Some(function_call) = part.get("functionCall") {
    let id = generate_random_id();
    let name = function_call["name"].as_str().unwrap_or_default().to_string();
    
    if !is_valid_function_name(&name) {
        // 無効な関数名の処理
    } else {
        if let Some(params) = function_call.get("args") {
            content.push(MessageContent::tool_request(
                id,
                Ok(ToolCall::new(&name, params.clone())),
            ));
        }
    }
}
```

### ステージ2: MCPクライアント/ツール解決

LLMが使用するツールを特定した後、Gooseはどの拡張機能がそのツールを処理するかを決定する必要があります：

1. **拡張機能の識別**:
   - `capabilities.rs`の`dispatch_tool_call`メソッドはツール呼び出しを処理します
   - ツール名の接頭辞を使用して、どの拡張機能が呼び出しを処理すべきかを識別します
   - `get_client_for_tool`メソッドは一致するMCPクライアントを検索します：

```rust
fn get_client_for_tool(&self, prefixed_name: &str) -> Option<(&str, McpClientBox)> {
    self.clients
        .iter()
        .find(|(key, _)| prefixed_name.starts_with(*key))
        .map(|(name, client)| (name.as_str(), Arc::clone(client)))
}
```

2. **ツール名の抽出**:
   - 拡張機能が識別されると、拡張機能固有のツール名が抽出されます：

```rust
let tool_name = tool_call
    .name
    .strip_prefix(client_name)
    .and_then(|s| s.strip_prefix("__"))
    .ok_or_else(|| ToolError::NotFound(tool_call.name.clone()))?;
```

3. **ツール実行**:
   - 識別されたMCPクライアントは特定のツールを呼び出すために使用されます
   - LLMからのパラメータがツールに渡されます

```rust
let client_guard = client.lock().await;
client_guard
    .call_tool(tool_name, tool_call.clone().arguments)
    .await
    .map(|result| result.content)
    .map_err(|e| ToolError::ExecutionError(e.to_string()))
```

## プラットフォームツール

Gooseには、リソースシステムにアクセスするための特別なツールが組み込まれています：

### platform__read_resource

```json
{
  "name": "platform__read_resource",
  "description": "Read a resource from an extension.\n\nResources allow extensions to share data that provide context to LLMs, such as\nfiles, database schemas, or application-specific information. This tool searches for the\nresource URI in the provided extension, and reads in the resource content. If no extension\nis provided, the tool will search all extensions for the resource.\n",
  "parameters": {
    "type": "object",
    "required": ["uri"],
    "properties": {
      "uri": {"type": "string", "description": "Resource URI"},
      "extension_name": {"type": "string", "description": "Optional extension name"}
    }
  }
}
```

このツールは特定のURIに対応するリソースの内容を読み取ります。`extension_name`パラメータが指定されていない場合は、すべての拡張機能から指定されたURIのリソースを検索します。

### platform__list_resources

```json
{
  "name": "platform__list_resources",
  "description": "List resources from an extension(s).\n\nResources allow extensions to share data that provide context to LLMs, such as\nfiles, database schemas, or application-specific information. This tool lists resources\nin the provided extension, and returns a list for the user to browse. If no extension\nis provided, the tool will search all extensions for the resource.\n",
  "parameters": {
    "type": "object",
    "properties": {
      "extension_name": {"type": "string", "description": "Optional extension name"}
    }
  }
}
```

このツールは利用可能なリソースの一覧を取得します。`extension_name`パラメータが指定されていない場合は、すべての拡張機能のリソース一覧を返します。

## MCPとリソースシステムの関係

リソースシステムはMCP（Model Context Protocol）の主要な機能の一つとして実装されています。これにより、拡張機能はLLMに対して構造化された方法でデータを提供できます。

### MCPサーバーのリソース機能実装

MCPサーバーは、初期化時に以下のようにリソース機能のサポートを宣言します：

```rust
fn capabilities(&self) -> ServerCapabilities {
    CapabilitiesBuilder::new()
        .with_resources(true, true)  // リソース機能をサポート
        .build()
}
```

リソース機能をサポートする拡張機能は、以下のインターフェースを実装する必要があります：

```rust
fn list_resources(&self) -> Vec<mcp_core::resource::Resource>;
fn read_resource(
    &self,
    uri: &str,
) -> Pin<Box<dyn Future<Output = Result<String, ResourceError>> + Send + 'static>>;
```

### リソース機能の処理フロー

`capabilities.rs`ファイルでは、`read_resource`および`list_resources`メソッドが実装されており、これらがLLMモデルから直接呼び出されるプラットフォームツールとマッピングされています：

```rust
async fn read_resource(&self, params: Value) -> Result<Vec<Content>, ToolError> {
    let uri = params
        .get("uri")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ToolError::InvalidParameters("Missing 'uri' parameter".to_string()))?;

    let extension_name = params.get("extension_name").and_then(|v| v.as_str());
    
    // 実装詳細...
}

async fn list_resources(&self, params: Value) -> Result<Vec<Content>, ToolError> {
    let extension = params.get("extension").and_then(|v| v.as_str());
    
    // 実装詳細...
}
```

## リソースシステムの使用例

### アクティブな拡張機能のリソース一覧取得

```json
{"name": "platform__list_resources", "arguments": {}}
```

### 特定の拡張機能のリソース一覧取得

```json
{"name": "platform__list_resources", "arguments": {"extension_name": "jetbrains"}}
```

### ファイルリソースの読み取り

```json
{"name": "platform__read_resource", "arguments": {"uri": "file:///path/to/project/file.py", "extension_name": "jetbrains"}}
```

## 処理フロー例

ユーザーがGooseにリソースを要求した場合の全体的な処理フロー：

1. ユーザーが入力: 「このプロジェクトのpackage.jsonファイルを表示して」
2. LLMがこのリクエストを分析し、`platform__read_resource`ツールを使用することを決定
3. LLMレスポンスにツール呼び出しが含まれる: `{name: "platform__read_resource", args: {uri: "file:///path/to/package.json", extension_name: "developer"}}`
4. Gooseはこのレスポンスを処理し、ツールリクエストを抽出
5. `dispatch_tool_call`が特別処理を行う:
   - `platform__read_resource`はプラットフォームツールとして認識される
   - `read_resource`メソッドが呼び出される
   - 指定されたURI（ここでは「file:///path/to/package.json」）が検索される
6. リソースが見つかった場合、その内容が返される
7. 結果は`ToolResponse`として会話に追加される
8. ツール結果を含めて会話が継続

## リソース対応拡張機能の例

以下の拡張機能はリソースシステムを実装しています：

1. **developer**: プロジェクトファイルや環境情報を提供
2. **memory**: Gooseの永続的な記憶を提供
3. **jetbrains**: IDE内のプロジェクト情報やコードファイルを提供
4. **computercontroller**: ローカルコンピューターのファイルやリソースへのアクセスを提供

## システムプロンプトでの表示

リソース機能をサポートする拡張機能がある場合、システムプロンプトには以下のように明記されます：

```markdown
## jetbrains
jetbrains supports resources, you can use platform__read_resource,
and platform__list_resources on this extension.

### Instructions
JetBrains IDE integration
```

## リソースの優先度とアクティブリソース

リソースには0.0から1.0の優先度があり、1.0の優先度を持つリソースは「アクティブ」とみなされます。アクティブなリソースは自動的にLLMのコンテキストに含まれます。

```rust
// Check if the resource is active
pub fn is_active(&self) -> bool {
    if let Some(priority) = self.priority() {
        (priority - 1.0).abs() < EPSILON
    } else {
        false
    }
}
```

## MCPアーキテクチャの利点

1. **モジュール性**: 拡張機能は独立して開発・展開可能
2. **柔軟性**: 拡張機能はどの言語でも実装可能
3. **セキュリティ**: 拡張機能は制御された通信を持つ別プロセスとして実行
4. **一貫性**: すべてのツール操作のための標準化されたプロトコル
5. **発見可能性**: LLMは説明に基づいてツールを発見して使用可能

## まとめ

Gooseのリソースシステムは、MCP（Model Context Protocol）アーキテクチャを活用して、拡張機能から言語モデルへの効率的なデータ提供を実現しています。`platform__read_resource`と`platform__list_resources`ツールを通じて、LLMはこれらのリソースに簡単にアクセスでき、拡張機能側は標準化されたインターフェースを実装するだけでリソース機能を提供できます。

このアーキテクチャにより、Gooseはコアシステムとその拡張機能の間にクリーンな分離を維持しながら、ファイルシステム、データベース、IDEのプロジェクトなど、様々なデータソースに接続できます。これにより、ユーザーのリクエストに対してより知識を持ったインテリジェントな応答を生成することが可能になります。