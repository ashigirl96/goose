# Gooseにおけるmermaidの表示設定

Gooseのドキュメントでmermaid図を作成する際は、color schemaとして「monokai」を採用します。このドキュメントでは、Gooseでmermaidを使用する際の標準的な設定と使用例を示します。

## 標準設定

Gooseでmermaidを使用する際は、以下のように`%%{init}%%`セクションでmonokaiテーマを指定してください：

````markdown
```mermaid
%%{init: { 'theme': 'monokai' } }%%
// ここにmermaidの図を記述
```
````

## 使用例

### シーケンス図の例

````markdown
```mermaid
%%{init: { 'theme': 'monokai' } }%%
sequenceDiagram
    participant User as ユーザー
    participant Session as Gooseセッション
    participant Planner as プランナーモデル
    participant Executor as エグゼキューターモデル
    
    User->>Session: /plan コマンド実行
    Session->>Session: RunMode::Plan に変更
    Session->>User: "Entering plan mode" メッセージ表示
    User->>Session: タスク説明入力
    Session->>Planner: タスク説明を送信
    Planner->>Session: 計画または明確化質問を返す
```
````

### フローチャートの例

````markdown
```mermaid
%%{init: { 'theme': 'monokai' } }%%
graph TD
    A[開始] --> B{条件判断}
    B -->|条件満たす| C[処理1]
    B -->|条件満たさない| D[処理2]
    C --> E[終了]
    D --> E
```
````

### クラス図の例

````markdown
```mermaid
%%{init: { 'theme': 'monokai' } }%%
classDiagram
    class Session {
        -RunMode mode
        -Vec<Message> messages
        +plan_with_reasoner_model()
        +process_agent_response()
    }
    class Agent {
        +get_plan_prompt()
        +reply()
    }
    Session --> Agent
```
````

### ガントチャートの例

````markdown
```mermaid
%%{init: { 'theme': 'monokai' } }%%
gantt
    title Goose開発スケジュール
    dateFormat  YYYY-MM-DD
    section 機能開発
    プラン機能実装      :a1, 2023-01-01, 30d
    テスト             :after a1, 15d
    ドキュメント作成    :after a1, 20d
```
````

### 状態遷移図の例

````markdown
```mermaid
%%{init: { 'theme': 'monokai' } }%%
stateDiagram-v2
    [*] --> 通常モード
    通常モード --> 計画モード: /plan
    計画モード --> 通常モード: 計画を実行
    計画モード --> 計画モード: 明確化質問
    計画モード --> 通常モード: /endplan
    通常モード --> [*]: /exit
```
````

## monokaiテーマの特徴

monokaiテーマは以下の特徴を持ちます：

- ダークな背景色
- 鮮やかな色のコントラスト
- 長時間見ても目に優しい配色
- コード編集でも人気の高いカラースキーム

## 適用時の注意点

1. GitHub上でmarkdownファイルを閲覧する場合、カスタムテーマが適用されない場合があります
2. ローカルでの表示時と、各種ドキュメントサイトでの表示が異なる場合があります
3. 印刷時には色が適切に出力されるか確認してください

## まとめ

Gooseでドキュメントを作成する際は、mermaid図にmonokaiテーマを適用することで、一貫性のある視覚的な表現を提供します。これにより、Gooseのドキュメント全体の一貫性が向上し、読みやすさが向上します。