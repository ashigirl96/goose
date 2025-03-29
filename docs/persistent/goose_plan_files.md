# Gooseプラン機能の関連ファイル構造

このドキュメントでは、Gooseのプラン機能に関連する主要なファイルとその役割について説明します。プラン機能はGooseのコードベース内の様々な場所に実装が分散しており、この全体像を理解することで機能の仕組みをより深く把握できます。

## コアファイル

```mermaid
%%{init: { 'theme': 'monokai' } }%%
graph TD
    A[crates/goose-cli/src/session/mod.rs] --> B[crates/goose-cli/src/session/input.rs]
    A --> C[crates/goose-cli/src/session/output.rs]
    A --> D[crates/goose/src/prompts/plan.md]
    E[crates/goose/src/agents/agent.rs] --> F[crates/goose/src/agents/capabilities.rs]
    E --> G[crates/goose/src/agents/reference.rs]
```

### 1. セッション管理関連ファイル

#### `crates/goose-cli/src/session/mod.rs`

プラン機能の中核となるファイルで、以下の主要な実装が含まれています：

- `RunMode` 列挙型: 通常モードと計画モードを区別
- `plan_with_reasoner_model` メソッド: 計画作成と実行のメインロジックを処理
- `classify_planner_response` 関数: 応答が「計画」か「明確化質問」かを判断

最も重要な部分は `plan_with_reasoner_model` メソッドで、このメソッドがプランナーモデルとのやり取りや計画実行の流れを制御しています。

```rust
async fn plan_with_reasoner_model(
    &mut self,
    plan_messages: Vec<Message>,
    reasoner: Box<dyn Provider + Send + Sync>,
) -> Result<(), anyhow::Error> {
    // プランプロンプトの取得
    let plan_prompt = self.agent.get_plan_prompt().await?;
    // プランナーモデルに計画作成を要求
    let (plan_response, _usage) = reasoner.complete(&plan_prompt, &plan_messages, &[]).await?;
    // 応答の表示
    output::render_message(&plan_response, self.debug);
    // 応答タイプの分類
    let planner_response_type =
        classify_planner_response(plan_response.as_concat_text(), self.agent.provider().await)
            .await?;

    // 応答タイプに基づく処理
    match planner_response_type {
        PlannerResponseType::Plan => {
            // ユーザーに計画実行の確認
            let should_act = cliclack::confirm("Do you want to clear message history & act on this plan?")
                .initial_value(true)
                .interact()?;
            if should_act {
                // 計画実行のためのセットアップと実行
                // ...
            } else {
                // 計画を会話に追加
                self.messages.push(plan_response);
            }
        }
        PlannerResponseType::ClarifyingQuestions => {
            // 明確化質問を会話に追加
            self.messages.push(plan_response);
        }
    }

    Ok(())
}
```

#### `crates/goose-cli/src/session/input.rs`

ユーザー入力、特に `/plan` コマンドの処理を担当します：

- `InputResult::Plan` 列挙値: プラン命令の結果を表現
- `PlanCommandOptions` 構造体: プラン命令のオプションを保持
- `parse_plan_command` 関数: ユーザー入力からプラン命令を解析

```rust
#[derive(Debug)]
pub struct PlanCommandOptions {
    pub message_text: String,
}

fn parse_plan_command(input: String) -> Option<InputResult> {
    let options = PlanCommandOptions {
        message_text: input.trim().to_string(),
    };

    Some(InputResult::Plan(options))
}
```

#### `crates/goose-cli/src/session/output.rs`

プラン機能のユーザーインターフェース表示を担当します：

- `render_enter_plan_mode` 関数: プランモード開始時のメッセージ表示
- `render_act_on_plan` 関数: 計画実行時のメッセージ表示
- `render_exit_plan_mode` 関数: プランモード終了時のメッセージ表示

```rust
pub fn render_enter_plan_mode() {
    println!(
        "\n{} {}\n",
        style("Entering plan mode.").green().bold(),
        style("You can provide instructions to create a plan and then act on it. To exit early, type /endplan")
            .green()
            .dim()
    );
}

pub fn render_act_on_plan() {
    println!(
        "\n{}\n",
        style("Exiting plan mode and acting on the above plan")
            .green()
            .bold(),
    );
}

pub fn render_exit_plan_mode() {
    println!("\n{}\n", style("Exiting plan mode.").green().bold());
}
```

### 2. エージェント関連ファイル

#### `crates/goose/src/agents/agent.rs`

エージェントの基本的なインターフェースを定義します：

- `Agent` トレイト: プラン機能を含むエージェントの基本機能を定義
- `get_plan_prompt` メソッド: エージェントがプランプロンプトを提供するためのインターフェース

```rust
#[async_trait]
pub trait Agent: Send + Sync {
    // ...その他のメソッド...

    /// Get the plan prompt, which will be used with the planner (reasoner) model
    async fn get_plan_prompt(&self) -> anyhow::Result<String>;

    // ...その他のメソッド...
}
```

#### `crates/goose/src/agents/capabilities.rs`

エージェントの機能を提供するクラスです：

- `get_planning_prompt` メソッド: プランプロンプトの生成を担当
- 利用可能なツール情報の収集

```rust
pub async fn get_planning_prompt(&self, tools_info: Vec<ToolInfo>) -> String {
    let mut context: HashMap<&str, Value> = HashMap::new();
    context.insert("tools", serde_json::to_value(tools_info).unwrap());

    prompt_template::render_global_file("plan.md", &context).expect("Prompt should render")
}
```

#### `crates/goose/src/agents/reference.rs`

リファレンス実装のエージェントで、プラン機能の実装を含みます：

- `get_plan_prompt` メソッドの実装: 実際のプランプロンプト生成

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

### 3. プロンプトテンプレート

#### `crates/goose/src/prompts/plan.md`

プランナーモデルへのプロンプトテンプレートを提供します：

- 「プランナー」AIとしての役割定義
- 利用可能なツールのリスト
- 計画作成に関するガイドライン

```markdown
You are a specialized "planner" AI. Your task is to analyze the user's request from the chat messages and create either:
1. A detailed step-by-step plan (if you have enough information) on behalf of user that another "executor" AI agent can follow, or
2. A list of clarifying questions (if you do not have enough information) prompting the user to reply with the needed clarifications

{% if (tools is defined) and tools %} ## Available Tools
{% for tool in tools %}
**{{tool.name}}**
Description: {{tool.description}}
Parameters: {{tool.parameters}}

{% endfor %}
{% else %}
No tools are defined.
{% endif %}
## Guidelines
1. Check for clarity and feasibility
  - If the user's request is ambiguous, incomplete, or requires more information, respond only with all your clarifying questions in a concise list.
  - If available tools are inadequate to complete the request, outline the gaps and suggest next steps or ask for additional tools or guidance.
2. Create a detailed plan
  - Once you have sufficient clarity, produce a step-by-step plan that covers all actions the executor AI must take.
  - Number the steps, and explicitly note any dependencies between steps (e.g., "Use the output from Step 3 as input for Step 4").
  - Include any conditional or branching logic needed (e.g., "If X occurs, do Y; otherwise, do Z").
3. Provide essential context
  - The executor AI will see only your final plan (as a user message) or your questions (as an assistant message) and will not have access to this conversation's full history.
  - Therefore, restate any relevant background, instructions, or prior conversation details needed to execute the plan successfully.
4. One-time response
  - You can respond only once.
  - If you respond with a plan, it will appear as a user message in a fresh conversation for the executor AI, effectively clearing out the previous context.
  - If you respond with clarifying questions, it will appear as an assistant message in this same conversation, prompting the user to reply with the needed clarifications.
5. Keep it action oriented and clear
  - In your final output (whether plan or questions), be concise yet thorough.
  - The goal is to enable the executor AI to proceed confidently, without further ambiguity.
```

## 関連ファイルの相互作用

プラン機能は上記のファイルが連携して動作します：

1. `input.rs` がユーザーコマンド「/plan」を解析
2. `session/mod.rs` の `RunMode` が計画モードに変更され、`plan_with_reasoner_model` が呼び出される
3. エージェントの `get_plan_prompt` メソッドが `plan.md` テンプレートを利用してプロンプトを生成
4. プランナーモデルが応答を生成し、`classify_planner_response` が応答をカテゴライズ
5. 計画が承認されると、会話履歴がクリアされ、計画がユーザーメッセージとして追加される
6. 通常のエージェントフローでこの計画が実行される

## 環境変数による設定

```
GOOSE_PLANNER_PROVIDER    プランナーに使用するプロバイダー（OpenAI、Anthropicなど）
GOOSE_PLANNER_MODEL       プランナーに使用するモデル（GPT-4、Claudeなど）
```

これらの環境変数が設定されていない場合、通常のGoose設定から標準プロバイダーとモデルが使用されます。

## ファイル間の依存関係

プラン機能は広範なコードベースにまたがっているため、変更を加える際には複数のファイルに影響する可能性があります。特に以下の依存関係に注意が必要です：

1. `session/mod.rs` がエージェントの `get_plan_prompt` メソッドに依存
2. エージェントの実装が `capabilities.rs` の `get_planning_prompt` に依存
3. `get_planning_prompt` が `plan.md` テンプレートに依存

これらの依存関係を理解することで、機能の変更や拡張を適切に行うことができます。