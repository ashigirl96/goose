# Model Context Protocol (MCP) プロンプトの概念

## 概要

プロンプトは、サーバーが再利用可能なプロンプトテンプレートとワークフローを定義し、クライアントがそれらをユーザーやLLM（大規模言語モデル）に簡単に提示できるようにする機能です。プロンプトは、一般的なLLMとのインタラクションを標準化し共有するための強力な方法を提供します。

プロンプトは「ユーザー制御」を前提として設計されています。つまり、サーバーからクライアントに公開され、ユーザーが明示的に選択して使用できることを意図しています。通常、プロンプトはユーザーインターフェイスのユーザー主導コマンド（スラッシュコマンドなど）を通じてトリガーされ、ユーザーが利用可能なプロンプトを自然に発見して呼び出すことができます。

## MCPにおけるプロンプトの機能

MCPのプロンプトは、以下のような機能を持つ事前定義されたテンプレートです：

- 動的な引数を受け付ける
- リソースからのコンテキストを含める
- 複数のインタラクションを連鎖させる
- 特定のワークフローを導く
- UIの要素（スラッシュコマンドなど）として表示する

## プロンプトの構造

各プロンプトは以下の要素で定義されます：

```json
{
  "name": "string", // プロンプトの一意の識別子
  "description": "string", // 人間が読めるプロンプトの説明
  "arguments": [ // オプションの引数リスト
    {
      "name": "string", // 引数の識別子
      "description": "string", // 引数の説明
      "required": boolean // 引数が必須かどうか
    }
  ]
}
```

## プロンプトの発見

クライアントは、`prompts/list`エンドポイントを通じて利用可能なプロンプトを発見できます：

```json
// リクエスト
{
  "method": "prompts/list"
}

// レスポンス
{
  "prompts": [
    {
      "name": "analyze-code",
      "description": "コードを分析して潜在的な改善点を見つける",
      "arguments": [
        {
          "name": "language",
          "description": "プログラミング言語",
          "required": true
        }
      ]
    }
  ]
}
```

## プロンプトの使用

プロンプトを使用するには、クライアントが`prompts/get`リクエストを送信します：

```json
{
  "method": "prompts/get",
  "params": {
    "name": "analyze-code",
    "arguments": {
      "language": "Python",
      "code": "def add(a, b):\n  return a + b"
    }
  }
}
```

## 動的なプロンプトとリソースの埋め込み

プロンプトは動的で、サーバーがリソースやコンテキストに基づいて内容を生成できます。`prompts/get`リクエストを処理する際：

1. サーバーは必要に応じてリソースやその他のコンテキストを取得します
2. テンプレートを処理して、引数や追加情報を挿入します
3. 最終的なプロンプトを返します

## 実装例

以下は、MCPサーバーでプロンプトを実装する完全な例です：

```javascript
import { Server } from "@modelcontextprotocol/sdk/server";
import { ListPromptsRequestSchema, GetPromptRequestSchema } from "@modelcontextprotocol/sdk/types";

const PROMPTS = {
  "git-commit": {
    name: "git-commit",
    description: "Gitコミットメッセージを生成する",
    arguments: [
      {
        name: "changes",
        description: "Git diffまたは変更の説明",
        required: true
      }
    ]
  },
  "explain-code": {
    name: "explain-code",
    description: "コードの動作を説明する",
    arguments: [
      {
        name: "code",
        description: "説明するコード",
        required: true
      },
      {
        name: "language",
        description: "プログラミング言語",
        required: false
      }
    ]
  }
};

const server = new Server(
  { name: "example-prompts-server", version: "1.0.0" },
  { capabilities: { prompts: {} } }
);

// 利用可能なプロンプトのリスト
server.setRequestHandler(ListPromptsRequestSchema, async () => {
  return { prompts: Object.values(PROMPTS) };
});

// 特定のプロンプトを取得
server.setRequestHandler(GetPromptRequestSchema, async (request) => {
  const prompt = PROMPTS[request.params.name];
  if (!prompt) {
    throw new Error(`プロンプトが見つかりません: ${request.params.name}`);
  }
  
  // 以下は簡略化された例です。実際の実装ではより複雑な処理が行われる場合があります。
  if (request.params.name === "git-commit") {
    const changes = request.params.arguments?.changes;
    if (!changes) {
      throw new Error("changesパラメータが必要です");
    }
    
    return {
      content: [
        {
          type: "text",
          text: `以下の変更に基づいて、明確で簡潔なGitコミットメッセージを作成してください：\n\n${changes}`
        }
      ]
    };
  }
  
  if (request.params.name === "explain-code") {
    const code = request.params.arguments?.code;
    const language = request.params.arguments?.language || "不明な言語";
    
    if (!code) {
      throw new Error("codeパラメータが必要です");
    }
    
    return {
      content: [
        {
          type: "text",
          text: `以下の${language}コードの動作を詳しく説明してください：\n\n\`\`\`${language}\n${code}\n\`\`\``
        }
      ]
    };
  }
});
```

## マルチステップワークフロー

プロンプトは一連のステップやワークフローを定義することもできます。これにより、複雑なタスクを段階的に進めることが可能になります。例えば：

1. コードを分析
2. バグを特定
3. 修正案を提案
4. テストケースを生成

## UI統合

プロンプトはクライアントUIで以下のように表示できます：

- スラッシュコマンド（例：`/analyze-code`）
- クイックアクション
- コンテキストメニュー項目
- コマンドパレットエントリ
- ガイド付きワークフロー
- インタラクティブフォーム

## 更新と変更

サーバーはプロンプトの変更についてクライアントに通知できます：

- サーバー機能：`prompts.listChanged`
- 通知：`notifications/prompts/list_changed`
- クライアントがプロンプトリストを再フェッチ

## ベストプラクティス

プロンプトを実装する際のベストプラクティス：

- 明確で説明的なプロンプト名を使用する
- プロンプトと引数の詳細な説明を提供する
- すべての必須引数を検証する
- 欠落した引数を適切に処理する
- プロンプトテンプレートのバージョン管理を検討する
- 動的コンテンツを適切にキャッシュする
- エラー処理を実装する
- 期待される引数形式を文書化する
- プロンプトの組み合わせを検討する
- さまざまな入力でプロンプトをテストする

## セキュリティの考慮事項

プロンプトを実装する際のセキュリティ考慮事項：

- すべての引数を検証する
- ユーザー入力をサニタイズする
- レート制限の検討
- アクセス制御の実装
- プロンプト使用の監査
- 機密データの適切な処理
- 生成されたコンテンツの検証
- タイムアウトの実装
- プロンプトインジェクションのリスクの検討
- セキュリティ要件の文書化
