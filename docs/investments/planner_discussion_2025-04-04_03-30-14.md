# Gooseのプランナーモード実装に関する議論

## 初期質問
**ユーザー**: gooseには、agentにplannerモードが内蔵されている。私は、これはgoose-mcpのmemoryのようにextensionsとして機能を切り分けたほうが考えているが、なぜplannerモードをagentに追加している？

**Goose**: こんにちは！Gooseにおけるplannerモードの実装について質問いただきました。まず、現在のGooseのコードを調査して、plannerモードがどのように実装されているか確認してみましょう。

[コード調査結果の詳細...]

## Plannerモードの現在の実装

GooseのPlannerモードは、現在Agent内に直接組み込まれています。主な実装ポイントは：

1. `Agent` トレイトに `get_plan_prompt()` メソッドが定義されている
2. `Capabilities` クラスに `get_planning_prompt()` メソッドがあり、ツール情報を用いてプランニングプロンプトを生成
3. プランニングプロンプトは別のモデルに送られ、ユーザーの要求を計画に変換する
4. `Session` クラスの `plan_with_reasoner_model` メソッドがプランニングの実行を担当
5. プランナーの応答は「明確にする質問」または「実行計画」のいずれかに分類される

### Plannerモードがagentに内蔵されている理由

コードから推測すると、Plannerモードが内蔵されている理由はいくつか考えられます：

1. **共有リソースへのアクセス**: Plannerモードは既存のAgent機能（ツール情報、システムプロンプト、拡張機能など）に密接に依存している
2. **アーキテクチャの単純化**: 同じ基盤コードを利用することで、実装が簡潔になる
3. **開発の初期段階**: この機能が比較的新しく、まだextensionとして分離するほど機能が確立されていない可能性
4. **低レベルの統合**: Plannerモードはエージェントの中核的な振る舞いに関わるため、内部実装として扱われている

### extensionとして分離する利点

質問者の指摘通り、Plannerモードをextensionとして分離することにはいくつかの利点があります：

1. **関心の分離**: plannerの役割と実行agentの役割を明確に分けることができる
2. **モジュール性の向上**: 機能追加・変更が局所的になり、コードの保守性が向上する
3. **カスタマイズの容易さ**: ユーザーが異なるplanner実装を選択できるようになる
4. **アーキテクチャの一貫性**: goose-mcpのmemoryのように他の機能と同様の設計パターンを維持できる

## エージェント会話履歴へのアクセスに関する議論

**ユーザー**: 私も特に `エージェントの会話履歴を直接修正する権限を持っています` というのがextensionとして分離しづらいところかと思うんだけど、あなたなら、これをextensionとしてどのように実装する？

**Goose**: 

# エージェント会話履歴を扱うPlanner Extensionの設計案

拡張機能としてPlannerを実装するには、以下の原則に従うことが重要です：

1. **明確なインターフェース定義**：拡張機能とエージェント間の通信を明確に定義する
2. **権限の制御**：会話履歴への特別なアクセス権限を制御する仕組みを導入する
3. **イベント駆動アーキテクチャ**：直接的な状態操作ではなく、イベントを通じてエージェントと通信する

## 設計案：EventとCallback中心のアーキテクチャ

### 1. 新しいMCPプロトコル拡張の定義

```rust
// プランナーへのリクエスト定義
struct PlanRequest {
    conversation_history: Vec<Message>,  // 会話履歴
    available_tools: Vec<ToolInfo>,      // 利用可能なツール情報
}

// プランナーからのレスポンス定義
enum PlanResponse {
    Plan {
        plan: String,
        clear_history: bool,  // 会話履歴をクリアするかどうかのフラグ
    },
    ClarifyingQuestions {
        questions: String,
    }
}
```

### 2. メッセージ履歴の操作を許可する特別な権限を持つ拡張機能タイプの導入

```rust
// 拡張機能の追加的な機能を定義する拡張特権
enum ExtensionPrivilege {
    Standard,                  // 標準的な拡張機能
    ConversationModification,  // 会話履歴の変更が可能な拡張機能
    SystemConfiguration,       // システム設定の変更が可能な拡張機能
}
```

### 3. コールバックベースのインターフェース実装

```rust
// MCPプロトコルに新しいコールバックインターフェースを追加
trait ConversationModifier {
    fn clear_conversation(&self) -> Result<(), Error>;
    fn replace_conversation(&self, messages: Vec<Message>) -> Result<(), Error>;
    fn append_to_conversation(&self, message: Message) -> Result<(), Error>;
}
```

### 4. イベント駆動型のアーキテクチャ

```rust
// 拡張機能が発行できるイベント
enum ExtensionEvent {
    PlanCreated {
        plan: String,
        action: ConversationAction,
    },
    QuestionsGenerated {
        questions: String,
    },
}
```

[詳細な実装例と説明...]

## 通信コストに関する議論

**ユーザー**: たしかにそうかなと思ったんだけど、agentだと内部で必要最低限の通信で済んでいたのに、MCPを呼び出すために毎回通信をする必要が合ったりするのかな？

**Goose**: ご質問いただきありがとうございます。おっしゃる通り、通信コストは重要な考慮点です。

## エージェント内蔵とMCP拡張機能の通信コスト比較

### 現在のエージェント内蔵実装の通信フロー

現在の内蔵実装では、最小限の内部呼び出しで処理が完結しています：

```
[ユーザー] → [Goose CLI] → [Agent内のget_plan_prompt()] → [同じプロセス内の推論実行]
                                 ↓
                      [同じプロセス内で会話履歴操作] → [結果表示]
```

### MCP拡張機能としての実装の通信フロー

MCP拡張機能として実装した場合、プロセス間通信が発生します：

```
[ユーザー] → [Goose CLI] → [MCP Client] → [IPC/ソケット] → [MCP Server (Planner)]
                                                               ↓
                                                          [推論実行]
                                                               ↓
[結果表示] ← [会話履歴操作] ← [Goose CLI] ← [MCP Client] ← [MCP Server (Planner)]
```

### 通信コストを最適化する方法

1. **リファレンスベースの通信モデル**
2. **増分更新モデル**
3. **キャッシュ層の導入**
4. **共有メモリの使用**
5. **ストリーミングプロトコルの活用**

### 現実的な実装案：ハイブリッドアプローチ

```rust
// コアライブラリとしてのPlanner実装
pub mod planner {
    pub struct Planner {
        provider: Box<dyn Provider>,
    }
    
    impl Planner {
        pub fn new(provider: Box<dyn Provider>) -> Self { ... }
        
        pub fn generate_plan(&self, conversation: &[Message], tools: &[ToolInfo]) -> Result<PlanResult> { ... }
        
        pub fn classify_response(&self, response: &str) -> Result<PlanResponseType> { ... }
    }
}

// MCP拡張機能として実装するラッパー
struct PlannerExtension {
    // コアライブラリのインスタンスを内部で保持
    core_planner: planner::Planner,
}
```

## 新しいプランナー機能の実装提案

**ユーザー**: 現在、作ろうとしているplannerがある

1. ユーザが「〇〇を実現したい」や「〇〇について教えてほしい」と聞く
2. gooseがまず実現したいことに必要な情報を調査したり、不明瞭なときはユーザに逆に質問をして精度を高める
3. ユーザとgooseとのやり取りで、gooseがユーザの質問に答えられる情報が集まったり、planを立てるために必要な情報を集めたら、「investmentsディレクトリに保存をしますか？」と聞いて yes だったら保存をするようにする
4. 最初の質問が「実現をしたい」だったら、gooseはそれに対応するplan.mdやtodo.mdを作成する
5. gooseはplan.md通りに実行をする。１つが終わるたびにtodo.mdにチェックをつけていく
6. 全てが終わったら「記憶を消去しますか？」と聞いて、yesだったら会話履歴を削除する

というものを作りたい。この場合、agentとして新しいversionを作るべきか、mcpとして切り出すべきかどちらが良いと思う？

**Goose**: 

# Plannerの実装方針: Agent vs MCP Extension

ご提案のPlannerフローは非常に興味深く、ユーザーにとって価値の高い機能ですね。

## 提案されたPlannerの機能

1. **情報収集フェーズ**: ユーザーの要求を理解し、必要に応じて質問で明確化
2. **知識管理**: 収集した情報を構造化して保存
3. **計画生成**: 実行計画(plan.md)とタスクリスト(todo.md)の作成
4. **計画実行と追跡**: 計画に従って実行し、進捗を記録
5. **会話履歴管理**: 必要に応じて会話履歴をリセット

## 各アプローチの比較

### AgentとしてのPlanner実装

**メリット**:
- **低いコミュニケーションコスト**: 内部APIによる直接的で高速な通信
- **深い統合**: 会話履歴、ツール情報、エージェント状態への完全アクセス
- **シンプルなアーキテクチャ**: 単一プロセス内で完結し、複雑なIPC不要
- **開発速度**: 既存のエージェント機能を直接活用できる

**デメリット**:
- **コードの結合度**: エージェントコードとプランナーコードが緊密に結合
- **拡張性の制限**: エージェント外からのアクセスや再利用が難しい
- **テスト難易度**: エージェント全体を含めたテストが必要
- **バージョン管理の複雑さ**: エージェント本体と同時に更新が必要

### MCP Extensionとしての実装

**メリット**:
- **モジュール性**: 明確な責任分離と独立した開発サイクル
- **バージョン独立性**: エージェント本体と独立して更新可能
- **再利用性**: 他のツールやエージェントからも利用可能
- **拡張性**: 将来的な機能追加が容易

**デメリット**:
- **通信オーバーヘッド**: プロセス間通信のコスト
- **実装の複雑さ**: 適切なインターフェース設計と通信プロトコルが必要
- **会話履歴へのアクセス**: 会話履歴の操作には特別なメカニズムが必要
- **開発工数**: インターフェース定義や通信層の実装が必要

## 推奨アプローチ

**ハイブリッドアプローチ**: MCP Extension + コアライブラリアプローチ

1. **コアライブラリとしてのPlanner機能実装**:
   ```
   crates/goose-planner/src/
       - core.rs (基本的なプランニングロジック)
       - model.rs (データモデル定義)
       - storage.rs (計画の保存・読み込み)
       - executor.rs (計画実行ロジック)
   ```

2. **MCPインターフェースレイヤーの実装**
3. **エージェント統合レイヤーの実装**

### 実装戦略

以下のような段階的アプローチが効果的です：

1. コアライブラリの開発
2. エージェント統合
3. MCP拡張機能としての公開

## コード例: 会話管理

```rust
// コアライブラリ内の抽象インターフェース
pub trait ConversationManager {
    fn get_conversation(&self) -> Vec<Message>;
    fn clear_conversation(&mut self) -> Result<()>;
    fn append_message(&mut self, message: Message) -> Result<()>;
    fn save_to_storage(&self, path: &Path, metadata: &PlanMetadata) -> Result<()>;
}
```

## 結論

提案している機能は、会話履歴の管理、計画の生成と実行、ファイル操作など複雑な機能を含んでいます。将来的な拡張性と再利用性を考慮すると、**MCP Extension + コアライブラリアプローチ**が最適です。

ただし、開発リソースや時間的制約が厳しい場合は、まず**エージェント直接統合**として実装し、後にモジュール化を進めるというアプローチも現実的です。

## プランナーの応答分類に関する議論

**ユーザー**: classify_planner_responseで判断基準が "plan" という文字列を含んでいるかどうかと雑な実装な気がするんだけど、crates/goose/src/prompts/plan.md にはそういう planかどうか判断するのかわかるの？

**Goose**: 

## 分析: classify_planner_responseの問題点

ご指摘の通り、現在の`classify_planner_response`関数の実装には以下の問題点があります：

1. **単純な文字列マッチングに依存**: 
   ```rust
   let predicted = result.as_concat_text();
   if predicted.to_lowercase().contains("plan") {
       Ok(PlannerResponseType::Plan)
   } else {
       Ok(PlannerResponseType::ClarifyingQuestions)
   }
   ```
   これは「plan」という単語が含まれているかだけで判断しており、非常に単純な実装です。

2. **言語モデル依存の脆さ**: 
   ```rust
   let (result, _usage) = provider
       .complete(
           "Reply only with the classification label: \"plan\" or \"clarifying questions\"",
           &[message],
           &[],
       )
       .await?;
   ```
   モデルに特定のフォーマットで応答するよう指示していますが、モデルがこれに従わない場合も考えられます。

3. **曖昧なケースの処理**: 例えば、計画と質問の両方を含む応答の場合、どちらに分類されるべきかが明確ではありません。

## plan.mdテンプレートとの関係

plan.mdテンプレートには、明確に2つの異なるモードが定義されています：

1. **計画モード**: 
   ```markdown
   1. A detailed step-by-step plan (if you have enough information) on behalf of user that another "executor" AI agent can follow
   ```

2. **質問モード**:
   ```markdown
   2. A list of clarifying questions (if you do not have enough information) prompting the user to reply with the needed clarifications
   ```

しかし、このテンプレートには「この出力が計画かどうかを判断する方法」についての明示的な指示はありません。つまり:

- テンプレートは出力のフォーマットを指示していますが、その出力を区別するための明確なマーカーやフォーマットの指定がない
- `classify_planner_response`関数は、このテンプレートの出力を解釈するために使われているが、その解釈方法が単純すぎる

## 改善案

この問題を解決するためには、以下のようなアプローチが考えられます：

### 1. plan.mdテンプレートの改善

テンプレートに明示的なマーカーを追加する：

```markdown
When providing a plan, start your response with: "## PLAN"
When providing clarifying questions, start your response with: "## QUESTIONS"
```

これにより、出力を簡単に分類できるようになります。

### 2. classify_planner_response関数の改善

```rust
pub async fn classify_planner_response(
    message_text: String,
    provider: Arc<Box<dyn Provider>>,
) -> Result<PlannerResponseType> {
    // 明示的なマーカーを最初に確認
    if message_text.trim().starts_with("## PLAN") {
        return Ok(PlannerResponseType::Plan);
    }
    if message_text.trim().starts_with("## QUESTIONS") {
        return Ok(PlannerResponseType::ClarifyingQuestions);
    }

    // 明示的なマーカーがない場合は、より詳細な分析を行う
    let prompt = format!(
        "Analyze the text below and classify it as either a 'plan' or 'clarifying questions'. \
        A 'plan' is a detailed step-by-step sequence of actions to accomplish a task. \
        'Clarifying questions' are questions that seek additional information to better understand a request.\n\n\
        Guidelines for classification:\n\
        - If the text primarily contains numbered steps or instructions, classify as 'plan'\n\
        - If the text primarily asks questions or requests more information, classify as 'clarifying questions'\n\
        - If the text contains both, classify based on what appears to be the primary intent\n\n\
        Text to classify:\n---\n{message_text}\n---\n\
        Classification (respond with only one word, either 'plan' or 'clarifying questions'):"
    );

    let message = Message::user().with_text(&prompt);
    let (result, _usage) = provider
        .complete(
            "You are a text classifier. Reply only with the exact classification label - either 'plan' or 'clarifying questions'.",
            &[message],
            &[],
        )
        .await?;

    let predicted = result.as_concat_text().trim().to_lowercase();
    
    // より堅牢な分類ロジック
    if predicted.contains("plan") && !predicted.contains("clarifying") {
        Ok(PlannerResponseType::Plan)
    } else if predicted.contains("clarifying") || predicted.contains("questions") {
        Ok(PlannerResponseType::ClarifyingQuestions)
    } else {
        // デフォルトの振る舞い - テキスト中に「step」や数字のリストが含まれているかをヒューリスティックに判断
        let contains_numbered_steps = message_text.lines()
            .any(|line| line.trim().starts_with("1.") || line.trim().starts_with("Step 1"));
            
        if contains_numbered_steps {
            Ok(PlannerResponseType::Plan)
        } else {
            Ok(PlannerResponseType::ClarifyingQuestions)
        }
    }
}
```

### 3. 構造化された出力フォーマットの使用

JSONやYAMLなどの構造化フォーマットを使用して、計画と質問を明確に区別することもできます：

```markdown
When providing a plan, format your response as follows:
```json
{
  "type": "PLAN",
  "content": "Your detailed plan here..."
}
```

When providing clarifying questions, format your response as follows:
```json
{
  "type": "QUESTIONS",
  "content": "Your clarifying questions here..."
}
```
```

そして、`classify_planner_response`関数でこのJSONを解析します。

## 結論

ご指摘の通り、現在の実装は非常に単純で、"plan"という文字列を含むかどうかだけで判断しています。一方、plan.mdテンプレートには、出力が計画か質問かを明示的に区別するための指示が含まれていません。

この状況を改善するには：

1. **テンプレートにマーカーの指示を追加する**
2. **より堅牢な分類ロジックを実装する**
3. **構造化されたフォーマットを使用する**

これらの改善により、計画と質問の分類がより正確になり、ユーザー体験も向上するでしょう。

ただし、現在のテンプレートは「計画作成」と「質問生成」の2つのモードを持つガイドラインのように設計されており、どちらのモードを選ぶかは言語モデル自体の判断に委ねられています。そのため、出力を明確に分類するためのラベルやマーカーが含まれていないというのが現状です。