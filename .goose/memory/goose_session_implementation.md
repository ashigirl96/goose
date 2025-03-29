# goose session implementation architecture design
# Gooseのセッション実装の詳細

1. **セッションの全体構造**:
   - `crates/goose-cli/src/session`ディレクトリに実装
   - 主要ファイル: `mod.rs`(メイン), `builder.rs`(構築), `input.rs`(入力処理), `output.rs`(出力表示), `completion.rs`(補完), `prompt.rs`(プロンプト), `thinking.rs`(思考表示)

2. **Session構造体**:
   ```rust
   pub struct Session {
       agent: Box<dyn Agent>,           // AIエージェント
       messages: Vec<Message>,          // メッセージ履歴
       session_file: PathBuf,           // セッションファイルへのパス
       completion_cache: Arc<std::sync::RwLock<CompletionCache>>, // 補完用キャッシュ
       debug: bool,                     // デバッグモードフラグ
       run_mode: RunMode,               // 実行モード (Normal/Plan)
   }
   ```

3. **セッション構築と初期化**:
   - `build_session`関数: プロバイダー・モデル取得、新規/既存セッション処理、拡張機能初期化、システムプロンプト設定

4. **インタラクティブモード**:
   - `interactive`メソッド: コマンド履歴管理、入力処理、AIエージェント通信、スラッシュコマンド処理

5. **入力処理**:
   - `input.rs`: 通常メッセージ、スラッシュコマンド、プロンプトコマンド、プランモードコマンドの処理
   - `InputResult` enum: 様々な入力タイプを表現

6. **出力表示**:
   - `output.rs`: Markdownレンダリング、ツール表示、エラー表示、テーマ管理
   - テーマ: Light/Dark/ANSIの3種類

7. **拡張機能管理**:
   - 動的拡張機能追加: `add_extension`と`add_builtin`メソッド
   - 組み込み拡張機能、外部(stdio)拡張機能、プロンプト拡張機能のサポート

8. **プランニングモード**:
   - プラン作成と実行の仕組み
   - 別AIモデル(リーズナー)を使用した計画生成

9. **メッセージ永続化**:
   - `persist_messages`関数: セッション中断/再開のためのファイル保存

10. **エラー処理**:
    - `handle_interrupted_messages`メソッド: ツール実行中断や例外処理

11. **セッションフロー**:
    - 入力 → 解析 → コマンド実行 → AIエージェント委譲 → 結果表示 → 履歴更新

12. **デスクトップクライアント連携**:
    - Electronアプリから`goosed`サーバー起動
    - SSEストリーミングによるリアルタイム通信

特に重要な点として、メッセージの永続化、拡張機能のサポート、柔軟な入出力処理、エラー耐性、プランニングモード、マルチモーダルインターフェースを持つ設計が特徴。

