# Gooseのツール管理メカニズム解析

Gooseフレームワークでは、`get_prefixed_tools`メソッドとその周辺の処理が、AIモデルに提供するツールを管理する重要な役割を果たしています。この文書では、Gooseがどのようにツールを収集し、フィルタリングしているかを詳細に解説します。

## ツール収集の基本プロセス

`capabilities.rs`の`get_prefixed_tools`メソッドは、以下のように実装されています：

```rust
pub async fn get_prefixed_tools(&mut self) -> ExtensionResult<Vec<Tool>> {
    let mut tools = Vec::new();
    for (name, client) in &self.clients {
        let client_guard = client.lock().await;
        let mut client_tools = client_guard.list_tools(None).await?;

        loop {
            for tool in client_tools.tools {
                tools.push(Tool::new(
                    format!("{}__{}", name, tool.name),
                    &tool.description,
                    tool.input_schema,
                ));
            }

            // exit loop when there are no more pages
            if client_tools.next_cursor.is_none() {
                break;
            }

            client_tools = client_guard.list_tools(client_tools.next_cursor).await?;
        }
    }
    Ok(tools)
}
```

このメソッドは、**登録されているすべての拡張機能（クライアント）からすべてのツールを収集し**、拡張機能名をプレフィックスとして追加したリストを返します。

## ツールのフィルタリングと拡張

しかし、実際には各Agent実装（`TruncateAgent`や`SummarizeAgent`など）の`reply`メソッド内で、特定の条件に基づいてツールが追加または除外されます。

TruncateAgentの`reply`メソッドの先頭部分で以下の処理が行われています：

```rust
async fn reply(
    &self,
    messages: &[Message],
    session: Option<SessionConfig>,
) -> anyhow::Result<BoxStream<'_, anyhow::Result<Message>>> {
    let mut messages = messages.to_vec();
    let reply_span = tracing::Span::current();
    let mut capabilities = self.capabilities.lock().await;
    let mut tools = capabilities.get_prefixed_tools().await?;
    let mut truncation_attempt: usize = 0;

    // Load settings from config
    let config = Config::global();
    let goose_mode = config.get_param("GOOSE_MODE").unwrap_or("auto".to_string());

    // we add in the 2 resource tools if any extensions support resources
    // TODO: make sure there is no collision with another extension's tool name
    let read_resource_tool = Tool::new(
        "platform__read_resource".to_string(),
        indoc! {r#"
            Read a resource from an extension.
            // ... description ...
        "#}.to_string(),
        json!({
            // ... schema ...
        }),
    );

    let list_resources_tool = Tool::new(
        "platform__list_resources".to_string(),
        indoc! {r#"
            List resources from an extension(s).
            // ... description ...
        "#}.to_string(),
        json!({
            // ... schema ...
        }),
    );

    if capabilities.supports_resources() {
        tools.push(read_resource_tool);
        tools.push(list_resources_tool);
    }

    let config = capabilities.provider().get_model_config();
    let mut system_prompt = capabilities.get_system_prompt().await;
    let mut toolshim_tools = vec![];
    if config.toolshim {
        // If tool interpretation is enabled, modify the system prompt to instruct to return JSON tool requests
        system_prompt = modify_system_prompt_for_tool_json(&system_prompt, &tools);
        // make a copy of tools before empty
        toolshim_tools = tools.clone();
        // pass empty tools vector to provider completion since toolshim will handle tool calls instead
        tools = vec![];
    }
    
    // ... 以下省略 ...
```

## ツール管理の主なポイント

1. **基本ツールセットの取得**：
   - `capabilities.get_prefixed_tools().await?`で、すべての拡張機能から登録されているすべてのツールを取得

2. **リソースツールの条件付き追加**：
   - リソース対応拡張機能が存在する場合（`capabilities.supports_resources()`が`true`の場合）のみ、2つの特殊なリソース関連ツールを追加：
     - `platform__read_resource`：リソース読み取りツール
     - `platform__list_resources`：リソース一覧取得ツール

3. **ツールシム対応**（特にollama向け）：
   - `config.toolshim`が`true`の場合：
     - システムプロンプトを修正してJSON形式のツールリクエストを指示
     - 現在のツールのコピーを`toolshim_tools`として保存
     - 実際に提供されるツールリストを空に設定（`tools = vec![]`）
     - この場合、ツールはシステムプロンプト内の指示として組み込まれる

4. **GOOSEモードによる実行時のツール処理**：
   - AIモデルがツールを呼び出そうとする場合、GOOSEモードに基づいて処理方法が決定される：
     - `auto`モード：すべてのツールを自動的に実行
     - `approve`モード：ユーザー承認が必要（事前に承認済みのものを除く）
     - `smart_approve`モード：読み取り専用ツールは自動実行、それ以外は承認が必要
     - `chat`モード：ツール実行をスキップし、代わりに説明を提供

## 具体的なフィルタリングと選択ロジック

1. **拡張機能レベルでのフィルタリング**：
   - `get_prefixed_tools`は基本的に**すべての拡張機能からすべてのツールを収集**
   - 特定の拡張機能を除外するようなフィルタリングは行われない

2. **ツールレベルでのフィルタリング**：
   - **追加のみ**：基本セットのツールは除外されず、特定の条件下で追加のツールが追加される
   - **リソースツール**：リソース対応拡張機能がある場合のみ追加される特殊なツール
   - **ツールシム**：特定の提供者（主にollama）の場合、特殊な処理が行われる

3. **実行時のフィルタリング**：
   - `smart_approve`モードでは、`detect_read_only_tools`関数を使用して読み取り専用ツールを検出
   - 実行前にツールの属性に基づいた選別が行われる

4. **get_plan_prompt時のツール処理**：
   - 計画生成時には、ツールの情報（名前、説明、パラメータ名）が抽出され、計画プロンプトに含まれる
   ```rust
   async fn get_plan_prompt(&self) -> anyhow::Result<String> {
       let mut capabilities = self.capabilities.lock().await;
       let tools = capabilities.get_prefixed_tools().await?;
       let tools_info = tools
           .into_iter()
           .map(|tool| ToolInfo::new(&tool.name, &tool.description, get_parameter_names(&tool)))
           .collect();

       let plan_prompt = capabilities.get_planning_prompt(tools_info).await;

       Ok(plan_prompt)
   }
   ```

## 結論

Gooseのツール管理メカニズムには以下の特徴があります：

1. **包括的収集**：基本的にはすべての拡張機能からすべてのツールを収集
2. **条件付き追加**：特定の条件（リソース対応など）に基づいて特殊なツールを追加
3. **モデル依存の処理**：一部のモデル（ollama）では特殊な処理を行う
4. **実行時フィルタリング**：GOOSEモードや他の条件に基づいてツール実行を制御

重要なのは、**ツールの除外**というよりも、**必要に応じたツールの追加**が行われていることです。これにより、拡張機能が提供するすべての機能をユーザーが利用できるようになっています。ただし、特定の条件下では、特殊なツールが追加されたり、ツールの処理方法が変わったりします。