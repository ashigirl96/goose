---
sidebar_position: 4
---

# MCPアーキテクチャとツール選択プロセス

このドキュメントでは、Gooseのメッセージコントロールプロトコル（MCP）アーキテクチャと、ユーザーからの命令に基づいてどのようにツールを呼び出すかを説明します。

## 概要

メッセージコントロールプロトコル（MCP）は、Gooseのメインアプリケーションと様々な拡張機能間の通信を可能にする基本コンポーネントです。MCPは以下のような標準化された方法を提供します：

1. 拡張機能のケイパビリティをツールとして定義する
2. ユーザーの命令を処理して、適切な拡張機能とツールを選択する
3. ツール呼び出しを実行し、結果を会話フローに戻す

## MCP通信フロー

MCPアーキテクチャはクライアント・サーバーモデルに従っています：

1. **MCPサーバー**: 各拡張機能はMCPサーバーとして実行され、ツールと機能を提供します
2. **MCPクライアント**: Gooseのメインアプリケーションは各拡張機能へのクライアント接続を維持します
3. **JSON-RPCプロトコル**: クライアントとサーバー間の通信はJSON-RPCメッセージを使用します

```
ユーザー入力 → Gooseコア → LLM → ツール選択 → MCPクライアント → MCPサーバー（拡張機能） → ツール実行
```

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

## MCPサーバーの実装

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

## 処理フロー例

ユーザーがGooseにファイル一覧を要求した場合の全プロセス：

1. ユーザーが入力: 「現在のディレクトリのファイル一覧を表示して」
2. LLMがこのリクエストを分析し、`developer__shell`ツールを使用することを決定
3. LLMレスポンスにツール呼び出しが含まれる: `{name: "developer__shell", args: {command: "ls -la"}}`
4. Gooseはこのレスポンスを処理し、ツールリクエストを抽出
5. `dispatch_tool_call`が識別:
   - 拡張機能: `developer`
   - ツール: `shell`
6. `developer` MCPクライアントが`shell`ツールを引数`ls -la`で呼び出す
7. shellツールがコマンドを実行し、結果を返す
8. 結果は`ToolResponse`として会話に追加される
9. ツール結果を含めて会話が継続

## MCPアーキテクチャの利点

1. **モジュール性**: 拡張機能は独立して開発・展開可能
2. **柔軟性**: 拡張機能はどの言語でも実装可能
3. **セキュリティ**: 拡張機能は制御された通信を持つ別プロセスとして実行
4. **一貫性**: すべてのツール操作のための標準化されたプロトコル
5. **発見可能性**: LLMは説明に基づいてツールを発見して使用可能

このアーキテクチャにより、Gooseはコアシステムとその拡張機能の間にクリーンな分離を維持しながら、ユーザーのリクエストに動的に適応できます。