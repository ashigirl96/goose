# Model Context Protocol (MCP) ツールの概念

## 概要

ツール（Tools）は、Model Context Protocol（MCP）における強力なプリミティブであり、サーバーがクライアントに実行可能な機能を公開することを可能にします。ツールを通じて、LLM（大規模言語モデル）は外部システムと対話したり、計算を実行したり、現実世界でアクションを実行したりすることができます。

ツールは「モデル制御」を前提として設計されています。つまり、サーバーからクライアントにツールが公開され、AIモデルが（ユーザーの承認を得て）自動的にそれらを呼び出せるようにすることを意図しています。

MCPにおけるツールは、クライアントが呼び出し、LLMがアクションを実行するために使用できる実行可能な関数をサーバーが公開することを可能にします。ツールの主な側面は以下の通りです：

- **発見（Discovery）**: クライアントは `tools/list` エンドポイントを介して利用可能なツールを一覧表示できる
- **呼び出し（Invocation）**: ツールは `tools/call` エンドポイントを使用して呼び出され、サーバーが要求された操作を実行して結果を返す
- **柔軟性（Flexibility）**: ツールは単純な計算から複雑なAPI連携まで幅広い機能を提供できる

リソース（Resources）と同様に、ツールは一意の名前で識別され、その使用方法を導くための説明を含めることができます。ただし、リソースとは異なり、ツールは状態を変更したり外部システムと対話したりできる動的な操作を表します。

## ツール定義の構造

各ツールは以下の構造で定義されます：

```json
{
  "name": "string", // ツールの一意の識別子
  "description": "string", // 人間が読めるツールの説明
  "inputSchema": { // ツールのパラメータのJSON Schema
    "type": "object",
    "properties": {
      // ツール固有のパラメータ
    }
  },
  "annotations": { // オプションのツール動作に関するヒント
    "title": "string", // ツールの人間が読めるタイトル（UI表示用）
    "readOnlyHint": boolean, // trueの場合、ツールは環境を変更しない
    "destructiveHint": boolean, // trueの場合、ツールは破壊的な更新を実行する可能性がある
    "idempotentHint": boolean, // trueの場合、同じ引数で繰り返し呼び出しても追加の効果はない
    "openWorldHint": boolean // trueの場合、ツールは外部エンティティと対話する
  }
}
```

## ツールの実装

以下は、MCPサーバーで基本的なツールを実装する例です：

```javascript
const server = new Server(
  { name: "example-server", version: "1.0.0" },
  { capabilities: { tools: {} } }
);

// 利用可能なツールを定義
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [{
      name: "calculate_sum",
      description: "2つの数値を足し合わせる",
      inputSchema: {
        type: "object",
        properties: {
          a: { type: "number" },
          b: { type: "number" }
        },
        required: ["a", "b"]
      }
    }]
  };
});

// ツールの実行を処理
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  if (request.params.name === "calculate_sum") {
    const { a, b } = request.params.arguments;
    return {
      content: [
        {
          type: "text",
          text: String(a + b)
        }
      ]
    };
  }
  throw new Error("Tool not found");
});
```

Pythonでの実装例：

```python
from mcp.server import Server
import mcp.types as types

# サーバーを初期化
server = Server("example-server")

# 利用可能なツールを定義
@server.list_tools()
async def list_tools() -> list[types.Tool]:
    return [
        types.Tool(
            name="calculate_sum",
            description="2つの数値を足し合わせる",
            input_schema={
                "type": "object",
                "properties": {
                    "a": {"type": "number"},
                    "b": {"type": "number"}
                },
                "required": ["a", "b"]
            }
        )
    ]

# ツールの実行を処理
@server.call_tool()
async def call_tool(name: str, arguments: dict) -> types.ToolResult:
    if name == "calculate_sum":
        a = arguments.get("a", 0)
        b = arguments.get("b", 0)
        return types.ToolResult(
            content=[
                types.Content(
                    type="text",
                    text=str(a + b)
                )
            ]
        )
    raise ValueError(f"ツールが見つかりません: {name}")
```

## ツールのパターン例

サーバーが提供できるツールの種類の例をいくつか示します：

### システム操作

ローカルシステムと対話するツール：

- ファイル操作（読み取り、書き込み、一覧表示）
- プロセス管理（開始、停止、ステータス確認）
- システム情報（メモリ使用量、CPU負荷、ディスク容量）
- ネットワーク操作（接続確認、ポートスキャン）

### API連携

外部APIをラップするツール：

- 天気情報の取得
- ニュース記事の検索
- 株価データの取得
- 翻訳サービスの利用
- カレンダー予定の管理

### データ処理

データを変換または分析するツール：

- CSVデータの処理
- JSONデータの検証と変換
- テキスト分析（要約、感情分析）
- 画像処理（リサイズ、フィルタ適用）
- データマイニング操作

## ベストプラクティス

ツールを実装する際のベストプラクティス：

- 明確で説明的な名前と説明を提供する
- パラメータには詳細なJSON Schema定義を使用する
- ツールの説明にモデルの使用方法を示す例を含める
- 適切なエラー処理と検証を実装する
- 長時間操作に対する進捗報告を使用する
- ツールの操作を焦点を絞った原子的なものにする
- 期待される戻り値の構造をドキュメント化する
- 適切なタイムアウトを実装する
- リソースを多く消費する操作にはレート制限を検討する
- デバッグとモニタリングのためにツールの使用状況をログに記録する

## セキュリティの考慮事項

ツールを公開する際のセキュリティ考慮事項：

### 入力の検証
- すべてのパラメータをスキーマに対して検証する
- ファイルパスとシステムコマンドをサニタイズする
- URLと外部識別子を検証する
- パラメータのサイズと範囲をチェックする
- コマンドインジェクションを防止する

### アクセス制御
- 必要に応じて認証を実装する
- 適切な認可チェックを使用する
- ツールの使用状況を監査する
- リクエストにレート制限を設ける
- 不正使用を監視する

### エラー処理
- 内部エラーをクライアントに公開しない
- セキュリティ関連のエラーをログに記録する
- タイムアウトを適切に処理する
- エラー後にリソースをクリーンアップする
- 戻り値を検証する

## ツールの発見と更新

MCPは動的なツールの発見をサポートしています：

- クライアントはいつでも利用可能なツールを一覧表示できる
- サーバーは `notifications/tools/list_changed` を使用してツールが変更されたときにクライアントに通知できる
- ツールは実行時に追加または削除できる
- ツール定義は更新できる（ただし、これは慎重に行うべき）

## エラー処理

ツールのエラーは、MCPプロトコルレベルのエラーとしてではなく、結果オブジェクト内で報告する必要があります。これにより、LLMはエラーを確認し、潜在的にそれを処理できます。ツールがエラーに遭遇した場合：

- 結果内で `isError` を `true` に設定する
- `content` 配列にエラーの詳細を含める

以下はツールの適切なエラー処理の例です：

```javascript
// ツールの実行を処理
server.setRequestHandler(CallToolRequestSchema, async (request) => {
  try {
    if (request.params.name === "divide") {
      const { a, b } = request.params.arguments;
      
      if (b === 0) {
        return {
          isError: true,
          content: [
            {
              type: "text",
              text: "ゼロで除算することはできません。"
            }
          ]
        };
      }
      
      return {
        content: [
          {
            type: "text",
            text: String(a / b)
          }
        ]
      };
    }
  } catch (error) {
    return {
      isError: true,
      content: [
        {
          type: "text",
          text: `エラーが発生しました: ${error.message}`
        }
      ]
    };
  }
  
  throw new Error("ツールが見つかりません");
});
```

このアプローチにより、LLMはエラーが発生したことを認識し、潜在的に修正措置を講じたり、ユーザーの介入を要求したりすることができます。

## ツールのアノテーション

ツールのアノテーションは、ツールの動作に関する追加のメタデータを提供し、クライアントがツールの提示方法と管理方法を理解するのに役立ちます。これらのアノテーションはツールの性質と影響を記述するヒントですが、セキュリティ上の決定のために使用すべきではありません。

### ツールアノテーションの目的

ツールアノテーションはいくつかの主要な目的を果たします：

- モデルのコンテキストに影響を与えずにUX固有の情報を提供する
- クライアントがツールを適切に分類し提示するのに役立つ
- ツールの潜在的な副作用に関する情報を伝える
- ツール承認のための直感的なインターフェースの開発を支援する

### 利用可能なツールアノテーション

MCP仕様では、ツールに対して以下のアノテーションを定義しています：

| アノテーション | 型 | デフォルト | 説明 |
|--------------|-------|---------|----------|
| title | string | - | ツールのUI表示用の人間が読めるタイトル |
| readOnlyHint | boolean | false | trueの場合、ツールは環境を変更しない |
| destructiveHint | boolean | true | trueの場合、ツールは破壊的な更新を実行する可能性がある（readOnlyHintがfalseの場合のみ有意） |
| idempotentHint | boolean | false | trueの場合、同じ引数で繰り返し呼び出しても追加の効果はない（readOnlyHintがfalseの場合のみ有意） |
| openWorldHint | boolean | true | trueの場合、ツールは外部エンティティと対話する |

### 使用例

異なるシナリオでアノテーションを使用してツールを定義する方法の例：

```javascript
// 読み取り専用ツール
{
  name: "get_weather",
  description: "指定された場所の現在の天気情報を取得します",
  inputSchema: {
    type: "object",
    properties: {
      location: { type: "string" }
    },
    required: ["location"]
  },
  annotations: {
    title: "天気情報を取得",
    readOnlyHint: true,
    openWorldHint: true  // 外部APIを使用
  }
}

// 状態を変更するが破壊的ではないツール
{
  name: "add_todo",
  description: "Todoリストに新しいアイテムを追加します",
  inputSchema: {
    type: "object",
    properties: {
      title: { type: "string" },
      due_date: { type: "string", format: "date" }
    },
    required: ["title"]
  },
  annotations: {
    title: "Todo追加",
    readOnlyHint: false,
    destructiveHint: false,  // 既存のデータを変更しない
    idempotentHint: false,   // 同じ引数で呼び出すと重複アイテムが作成される
    openWorldHint: false     // 閉じたシステム内でのみ動作
  }
}

// 破壊的な更新を行うツール
{
  name: "delete_file",
  description: "指定されたファイルを削除します",
  inputSchema: {
    type: "object",
    properties: {
      path: { type: "string" }
    },
    required: ["path"]
  },
  annotations: {
    title: "ファイル削除",
    readOnlyHint: false,
    destructiveHint: true,   // データを永久に削除する
    idempotentHint: true,    // 同じファイルを複数回削除しても追加の効果はない
    openWorldHint: false     // ローカルシステム内でのみ動作
  }
}
```

### サーバー実装におけるアノテーションの統合

サーバー実装でアノテーションを使用する例：

```javascript
server.setRequestHandler(ListToolsRequestSchema, async () => {
  return {
    tools: [{
      name: "calculate_sum",
      description: "2つの数値を足し合わせる",
      inputSchema: {
        type: "object",
        properties: {
          a: { type: "number" },
          b: { type: "number" }
        },
        required: ["a", "b"]
      },
      annotations: {
        title: "数値を足し算",
        readOnlyHint: true,  // 状態を変更しない
        openWorldHint: false // 外部システムとやり取りしない
      }
    }]
  };
});
```

### ツールアノテーションのベストプラクティス

- **副作用について正確に記述する**: ツールが環境を変更するかどうか、およびそれらの変更が破壊的かどうかを明確に示す
- **説明的なタイトルを使用する**: ツールの目的を明確に説明する人間向けのタイトルを提供する
- **べき等性を適切に示す**: 同じ引数による繰り返し呼び出しが追加の効果を持たない場合のみ、ツールをべき等としてマークする
- **適切なオープン/クローズドワールドのヒントを設定する**: ツールがデータベースなどの閉じたシステムと対話するか、ウェブなどのオープンシステムと対話するかを示す
- **アノテーションはヒントであることを忘れない**: ToolAnnotationsのすべてのプロパティはヒントであり、ツールの動作を忠実に記述することは保証されていない。クライアントはアノテーションのみに基づいてセキュリティ上の重要な決定を行うべきではない

## ツールのテスト

MCPツールの包括的なテスト戦略には以下が含まれるべきです：

- **機能テスト**: ツールが有効な入力で正しく実行され、無効な入力を適切に処理することを検証する
- **統合テスト**: 実際のおよびモックの依存関係を使用して、外部システムとのツールの相互作用をテストする
- **セキュリティテスト**: 認証、認可、入力のサニタイズ、およびレート制限を検証する
- **パフォーマンステスト**: 負荷下での動作、タイムアウト処理、およびリソースのクリーンアップをチェックする
- **エラー処理**: ツールがMCPプロトコルを通じてエラーを適切に報告し、リソースをクリーンアップすることを確認する
