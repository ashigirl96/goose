# Gooseメモリ拡張機能：指示内容の解説

このドキュメントでは、Gooseのメモリ拡張機能の指示内容を日本語に翻訳し、それぞれの項目について詳細に解説します。メモリ拡張機能は、カテゴリ分けされた情報をセッション間で管理するための機能です。

## 1. 基本機能と概要

```
This extension allows storage and retrieval of categorized information with tagging support. It's designed to help
manage important information across sessions in a systematic and organized manner.
```

**翻訳**: この拡張機能は、タグ付けをサポートしたカテゴリ分類された情報の保存と取得を可能にします。セッションをまたいで重要な情報を体系的かつ整理された方法で管理するために設計されています。

**解説**: メモリ拡張機能の主な目的は、ユーザーとアシスタント間の会話で共有された重要な情報を、後で参照できるように保存することです。例えば、ユーザーの設定、プロジェクトの詳細、ワークフローの説明などを保存し、次回以降のセッションで利用できるようにします。

## 2. メモリ拡張機能の主な機能

```
Capabilities:
1. Store information in categories with optional tags for context-based retrieval.
2. Search memories by content or specific tags to find relevant information.
3. List all available memory categories for easy navigation.
4. Remove entire categories of memories when they are no longer needed.
```

**翻訳**: 
機能：
1. コンテキストベースの検索のためにオプションのタグを使って、カテゴリごとに情報を保存する。
2. 関連情報を見つけるために、コンテンツや特定のタグでメモリを検索する。
3. 簡単なナビゲーションのために、利用可能なすべてのメモリカテゴリをリスト表示する。
4. 不要になったときに、メモリのカテゴリ全体を削除する。

**解説**: メモリ拡張機能には、情報の保存、検索、一覧表示、削除という4つの主要な機能があります。情報はカテゴリとタグによって整理され、後で効率的に取得できるようになっています。この構造化されたアプローチにより、情報の管理と検索が容易になります。

## 3. メモリツールを呼び出すタイミング

```
When to call memory tools:
- These are examples where the assistant should proactively call the memory tool because the user is providing recurring preferences, project details, or workflow habits that they may expect to be remembered.
- Preferred Development Tools & Conventions
- User-specific data (e.g., name, preferences)
- Project-related configurations
- Workflow descriptions
- Other critical settings
```

**翻訳**: 
メモリツールを呼び出すタイミング：
- 以下は、ユーザーが繰り返し使用する設定、プロジェクトの詳細、ワークフローの習慣などを提供しており、それが記憶されることを期待している場合に、アシスタントが積極的にメモリツールを呼び出すべき例です。
- 好みの開発ツールと規約
- ユーザー固有のデータ（例：名前、設定）
- プロジェクト関連の設定
- ワークフローの説明
- その他の重要な設定

**解説**: アシスタントは、ユーザーが提供する情報の中でも特に重要で繰り返し参照される可能性が高いものを識別し、それをメモリに保存することを提案すべきです。これには、開発環境の設定、ユーザーの好み、プロジェクトの詳細などが含まれます。こうした情報を保存することで、ユーザーは毎回同じ情報を繰り返し提供する必要がなくなります。

## 4. 情報を保存する際の対話プロトコル

```
Interaction Protocol:
When important information is identified, such as:
- User-specific data (e.g., name, preferences)
- Project-related configurations
- Workflow descriptions
- Other critical settings
The protocol is:
1. Identify the critical piece of information.
2. Ask the user if they'd like to store it for later reference.
3. Upon agreement:
   - Suggest a relevant category like "personal" for user data or "development" for project preferences.
   - Inquire about any specific tags they want to apply for easier lookup.
   - Confirm the desired storage location:
     - Local storage (.goose/memory) for project-specific details.
     - Global storage (~/.config/goose/memory) for user-wide data.
     - IMPORTANT: Unless the user explicitly states "store globally" or similar, prefer local storage by default.
   - Use the remember_memory tool to store the information.
     - `remember_memory(category, data, tags, is_global)`
```

**翻訳**: 
対話プロトコル：
重要な情報が識別された場合（例えば）：
- ユーザー固有のデータ（例：名前、設定）
- プロジェクト関連の設定
- ワークフローの説明
- その他の重要な設定
プロトコルは以下の通りです：
1. 重要な情報を識別する。
2. ユーザーに、後で参照するためにその情報を保存したいかどうか尋ねる。
3. 同意を得たら：
   - ユーザーデータには「personal」、プロジェクト設定には「development」のような関連カテゴリを提案する。
   - 検索を容易にするために適用したい特定のタグについて尋ねる。
   - 希望の保存場所を確認する：
     - プロジェクト固有の詳細にはローカルストレージ（.goose/memory）。
     - ユーザー全体のデータにはグローバルストレージ（~/.config/goose/memory）。
     - 重要：ユーザーが明示的に「グローバルに保存」などと述べない限り、デフォルトではローカルストレージを優先する。
   - remember_memoryツールを使用して情報を保存する。
     - `remember_memory(category, data, tags, is_global)`

**解説**: メモリに情報を保存する際には、一連の対話ステップに従います。まず、保存すべき重要な情報を識別し、ユーザーに保存の許可を求めます。次に、適切なカテゴリとタグを提案し、ローカルとグローバルのどちらに保存するかを確認します。デフォルトでは、特に指定がない限りローカルストレージが優先されます。これは、プロジェクト固有の情報はそのプロジェクト内でのみ関連性があることが多いためです。

## 5. メモリツールを呼び出すキーワードとタイミング

```
Keywords that trigger memory tools:
- "remember"
- "forget"
- "memory"
- "save"
- "save memory"
- "remove memory"
- "clear memory"
- "search memory"
- "find memory"
Suggest the user to use memory tools when:
- When the user mentions a keyword that triggers a memory tool
- When the user performs a routine task
- When the user executes a command and would benefit from remembering the exact command
```

**翻訳**: 
メモリツールを呼び出すキーワード：
- 「覚えて」
- 「忘れて」
- 「メモリ」
- 「保存」
- 「メモリに保存」
- 「メモリを削除」
- 「メモリをクリア」
- 「メモリを検索」
- 「メモリを探す」
ユーザーにメモリツールの使用を提案するタイミング：
- ユーザーがメモリツールを呼び出すキーワードを言及した時
- ユーザーが定型的なタスクを実行する時
- ユーザーがコマンドを実行し、正確なコマンドを覚えておくことが有益な時

**解説**: アシスタントは、ユーザーの発言や行動から、メモリツールの使用が適切なタイミングを判断します。特定のキーワードを含む発言や、繰り返し実行される可能性のあるコマンドなどは、メモリツールの使用を提案する良い機会です。これにより、ユーザーは重要な情報やコマンドを再利用しやすくなります。

## 6. 情報保存の対話例

```
Example Interaction for Storing Information:
User: "For this project, we use black for code formatting"
Assistant: "You've mentioned a development preference. Would you like to remember this for future conversations?
User: "Yes, please."
Assistant: "I'll store this in the 'development' category. Any specific tags to add? Suggestions: #formatting
#tools"
User: "Yes, use those tags."
Assistant: "Shall I store this locally for this project only, or globally for all projects?"
User: "Locally, please."
Assistant: *Stores the information under category="development", tags="formatting tools", scope="local"*
Another Example Interaction for Storing Information:
User: "Remember the gh command to view github comments"
Assistant: "Shall I store this locally for this project only, or globally for all projects?"
User: "Globally, please."
Assistant: *Stores the gh command under category="github", tags="comments", scope="global"*
Example Interaction suggesting memory tools:
User: "I'm using the gh command to view github comments"
Assistant: "You've mentioned a command. Would you like to remember this for future conversations?
User: "Yes, please."
Assistant: "I'll store this in the 'github' category. Any specific tags to add? Suggestions: #comments #gh"
```

**翻訳**: 
情報保存の対話例：
ユーザー：「このプロジェクトでは、コードフォーマットにblackを使用しています」
アシスタント：「開発の設定について言及されました。今後の会話のためにこれを覚えておきましょうか？」
ユーザー：「はい、お願いします」
アシスタント：「'development'カテゴリに保存します。追加する特定のタグはありますか？提案：#formatting #tools」
ユーザー：「はい、それらのタグを使ってください」
アシスタント：「このプロジェクトのみにローカルに保存しますか、それともすべてのプロジェクトのためにグローバルに保存しますか？」
ユーザー：「ローカルにお願いします」
アシスタント：*情報をcategory="development", tags="formatting tools", scope="local"として保存*

情報保存の別の対話例：
ユーザー：「GitHubのコメントを表示するghコマンドを覚えておいて」
アシスタント：「このプロジェクトのみにローカルに保存しますか、それともすべてのプロジェクトのためにグローバルに保存しますか？」
ユーザー：「グローバルにお願いします」
アシスタント：*ghコマンドをcategory="github", tags="comments", scope="global"として保存*

メモリツールを提案する対話例：
ユーザー：「GitHubのコメントを表示するghコマンドを使用しています」
アシスタント：「コマンドについて言及されました。今後の会話のためにこれを覚えておきましょうか？」
ユーザー：「はい、お願いします」
アシスタント：「'github'カテゴリに保存します。追加する特定のタグはありますか？提案：#comments #gh」

**解説**: これらの例は、アシスタントがどのようにしてユーザーと対話しながら情報を保存するかを示しています。最初の例では、開発設定に関する情報をローカルに保存しています。2番目の例では、GitHubコマンドをグローバルに保存しています。3番目の例では、ユーザーが言及したコマンドをメモリに保存することを提案しています。これらの対話を通じて、アシスタントはユーザーとの協力のもとで情報を整理し、保存します。

## 7. メモリの取得方法

```
Retrieving Memories:
To access stored information, utilize the memory retrieval protocols:
- **Search by Category**:
  - Provides all memories within the specified context.
  - Use: `retrieve_memories(category="development", is_global=False)`
  - Note: If you want to retrieve all local memories, use `retrieve_memories(category="*", is_global=False)`
  - Note: If you want to retrieve all global memories, use `retrieve_memories(category="*", is_global=True)`
- **Filter by Tags**:
  - Enables targeted retrieval based on specific tags.
  - Use: Provide tag filters to refine search.
```

**翻訳**: 
メモリの取得：
保存された情報にアクセスするには、以下のメモリ取得プロトコルを利用します：
- **カテゴリ検索**：
  - 指定されたコンテキスト内のすべてのメモリを提供します。
  - 使用方法：`retrieve_memories(category="development", is_global=False)`
  - 注：すべてのローカルメモリを取得したい場合は、`retrieve_memories(category="*", is_global=False)`を使用
  - 注：すべてのグローバルメモリを取得したい場合は、`retrieve_memories(category="*", is_global=True)`を使用
- **タグによるフィルタリング**：
  - 特定のタグに基づいたターゲット検索を可能にします。
  - 使用方法：検索を絞り込むためにタグフィルターを提供します。

**解説**: メモリに保存された情報を取得するには、主にカテゴリベースの検索とタグベースのフィルタリングの2つの方法があります。カテゴリ検索では、特定のカテゴリに保存されたすべてのメモリを取得できます。また、ワイルドカード「*」を使用すると、すべてのカテゴリからメモリを取得できます。タグによるフィルタリングでは、特定のタグが付いたメモリのみを検索できます。これにより、必要な情報に絞って取得することが可能になります。

## 8. メモリの削除方法

```
To remove a memory, use the following protocol:
- **Remove by Category**:
  - Removes all memories within the specified category.
  - Use: `remove_memory_category(category="development", is_global=False)`
  - Note: If you want to remove all local memories, use `remove_memory_category(category="*", is_global=False)`
  - Note: If you want to remove all global memories, use `remove_memory_category(category="*", is_global=True)`
```

**翻訳**: 
メモリを削除するには、以下のプロトコルを使用します：
- **カテゴリによる削除**：
  - 指定されたカテゴリ内のすべてのメモリを削除します。
  - 使用方法：`remove_memory_category(category="development", is_global=False)`
  - 注：すべてのローカルメモリを削除したい場合は、`remove_memory_category(category="*", is_global=False)`を使用
  - 注：すべてのグローバルメモリを削除したい場合は、`remove_memory_category(category="*", is_global=True)`を使用

**解説**: メモリを削除する方法としては、カテゴリ単位での削除が主な方法です。特定のカテゴリに保存されたすべてのメモリを一度に削除できます。また、ワイルドカード「*」を使用すると、ローカルまたはグローバルのすべてのメモリを一度に削除できます。一部のメモリツールでは、特定のメモリ内容を指定して削除することも可能です。

## 9. メモリ取得の対話プロトコル

```
The Protocol is:
 1. Confirm what kind of information the user seeks by category or keyword.
 2. Suggest categories or relevant tags based on the user's request.
 3. Use the retrieve function to access relevant memory entries.
 4. Present a summary of findings, offering detailed exploration upon request.
```

**翻訳**: 
プロトコルは以下の通りです：
1. ユーザーがカテゴリやキーワードでどのような情報を求めているか確認します。
2. ユーザーのリクエストに基づいて、カテゴリや関連タグを提案します。
3. 取得関数を使用して、関連するメモリエントリにアクセスします。
4. 発見内容の要約を提示し、リクエストに応じて詳細な探索を提供します。

**解説**: メモリを取得する際の対話プロトコルは、ユーザーのニーズを理解し、適切なカテゴリやタグを提案し、関連するメモリを取得して提示するという流れに従います。アシスタントは最初に要約情報を提供し、必要に応じて詳細情報を提供します。このアプローチにより、ユーザーは必要な情報を効率的に取得できます。

## 10. メモリ取得の対話例

```
Example Interaction for Retrieving Information:
User: "What configuration do we use for code formatting?"
Assistant: "Let me check the 'development' category for any related memories. Searching using #formatting tag."
Assistant: *Executes retrieval: `retrieve_memories(category="development", is_global=False)`*
Assistant: "We have 'black' configured for code formatting, specific to this project. Would you like further
details?"
```

**翻訳**: 
情報取得の対話例：
ユーザー：「コードフォーマットにはどの設定を使用していますか？」
アシスタント：「関連するメモリがないか'development'カテゴリを確認します。#formattingタグを使用して検索します」
アシスタント：*取得を実行：`retrieve_memories(category="development", is_global=False)`*
アシスタント：「このプロジェクト特有のコードフォーマットとして'black'が設定されています。さらに詳細をご希望ですか？」

**解説**: この例では、ユーザーがコードフォーマットの設定について質問しています。アシスタントは'development'カテゴリと#formattingタグを使用して関連するメモリを検索し、プロジェクト特有の設定である'black'を特定しています。アシスタントはこの情報を簡潔に提供し、さらに詳細が必要かどうかを尋ねています。これは、効率的な情報検索と提示の良い例です。

## 11. メモリの概要とタグの役割

```
Memory Overview:
- Categories can include a wide range of topics, structured to keep information grouped logically.
- Tags enable quick filtering and identification of specific entries.
```

**翻訳**: 
メモリの概要：
- カテゴリには幅広いトピックが含まれ、情報を論理的にグループ化して保持するように構成されています。
- タグは特定のエントリの迅速なフィルタリングと識別を可能にします。

**解説**: メモリシステムは、カテゴリとタグという2つの主要な構成要素に基づいています。カテゴリは、関連する情報を論理的にグループ化するための広い枠組みを提供します。例えば、「development」、「personal」、「github」などのカテゴリがあります。一方、タグはより細かい分類を可能にし、特定の情報を迅速に検索するための手段を提供します。例えば、「#formatting」、「#tools」、「#comments」などのタグを使用して、カテゴリ内の特定のエントリを識別できます。

## 12. 運用ガイドライン

```
Operational Guidelines:
- Always confirm with the user before saving information.
- Propose suitable categories and tag suggestions.
- Discuss storage scope thoroughly to align with user needs.
- Acknowledge the user about what is stored and where, for transparency and ease of future retrieval.
```

**翻訳**: 
運用ガイドライン：
- 情報を保存する前に、常にユーザーに確認します。
- 適切なカテゴリとタグの提案を行います。
- ユーザーのニーズに合わせて、保存範囲を十分に議論します。
- 透明性と将来の取得の容易さのために、何がどこに保存されたかをユーザーに伝えます。

**解説**: メモリシステムを効果的に運用するためのガイドラインとして、ユーザーの同意を得ること、適切なカテゴリとタグを提案すること、保存範囲（ローカルまたはグローバル）を明確にすること、そして保存された情報を透明にユーザーに伝えることが挙げられています。これらのガイドラインに従うことで、ユーザーとアシスタントの間で信頼関係を築き、メモリシステムを効果的に活用することができます。

## 13. メモリの保存場所

```
- Local storage (.goose/memory) for project-specific details.
- Global storage (~/.config/goose/memory) for user-wide data.
- IMPORTANT: Unless the user explicitly states "store globally" or similar, prefer local storage by default.
```

**翻訳**: 
- プロジェクト固有の詳細にはローカルストレージ（.goose/memory）。
- ユーザー全体のデータにはグローバルストレージ（~/.config/goose/memory）。
- 重要：ユーザーが明示的に「グローバルに保存」などと述べない限り、デフォルトではローカルストレージを優先する。

**解説**: メモリは2つの異なる場所に保存できます：
1. **ローカルストレージ**：プロジェクトのルートディレクトリにある`.goose/memory/`ディレクトリに保存されます。ここには、現在のプロジェクトに固有の情報を保存します。これはデフォルトの保存場所です。
2. **グローバルストレージ**：ユーザーの設定ディレクトリにある`~/.config/goose/memory/`（macOS/Linux）または`~\AppData\Roaming\Block\goose\config\memory`（Windows）に保存されます。ここには、すべてのプロジェクトで共通して使用する情報を保存します。

ユーザーが明示的に「グローバルに保存して」と言わない限り、デフォルトではローカルストレージが使用されます。これは、プロジェクト固有の情報はそのプロジェクト内でのみ関連性があることが多いためです。

## 14. メモリの技術的実装

メモリはMarkdownファイルとして保存され、タグと内容が一緒に格納されます。具体的には：

1. 各カテゴリは別々のMarkdownファイル（`.md`拡張子）として保存されます。
2. ファイル内の各メモリエントリは、オプションのタグ行とそれに続くコンテンツで構成されます。
3. タグ行は「#」で始まり、スペースで区切られたタグのリストが続きます。
4. 複数のメモリエントリは空行で区切られます。

例えば：
```markdown
# formatting tools
For this project, we use black for code formatting

# comments gh
The gh command to view github comments is: gh pr view --comments
```

## 15. まとめ

Gooseのメモリ拡張機能は、ユーザーとアシスタントの間で共有された重要な情報を、構造化された方法で保存し、後で取得できるようにするための強力なツールです。カテゴリとタグを使用して情報を整理し、ローカルまたはグローバルに保存することで、セッションをまたいで一貫した対話を可能にします。

メモリ拡張機能を効果的に使用するためのキーポイント：
1. 重要な情報を識別する
2. ユーザーの同意を得る
3. 適切なカテゴリとタグを提案する
4. 保存場所（ローカルまたはグローバル）を確認する
5. 透明性を保ち、保存された情報をユーザーに伝える
6. 必要なときに情報を取得し、要約して提示する

これらの原則に従うことで、メモリ拡張機能はユーザーとアシスタントの対話体験を大幅に向上させることができます。