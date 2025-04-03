//! セッション構築とエクステンション管理のサンプル
//! このサンプルでは、以下の機能を説明します：
//! - build_session関数の使用方法
//! - add_extension関数によるエクステンションの追加
//! - メモリーの統合過程
//! 
//! 実行方法: cargo run --package goose-cli --example session_builder

use goose::message::Message;
use goose_cli::session::build_session;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦢 Gooseセッションビルダーの例");
    
    // 1. 設定の読み込み
    println!("1. 設定に関する情報");
    println!("   設定は~/.config/goose/config.yamlから読み込まれます");
    println!("   プロバイダーとモデル情報は環境変数からも読み込み可能です");
    
    // 2. セッションファイルの準備
    println!("2. セッションを構築中...");

    // 3. セッションの構築
    let mut session = build_session(
        None,  // セッションID
        false,             // 既存セッションの再開か否か
        Vec::new(),        // 拡張機能リスト
        Vec::new(),        // 組み込み拡張機能リスト
        true,              // デバッグモード
    ).await;
    println!("   セッションの構築完了");
    
    // 4. 組み込みエクステンションの追加
    println!("4. 組み込みエクステンションを追加中...");
    if let Err(e) = session.add_builtin("developer".to_string()).await {
        println!("   エクステンションの追加エラー: {}", e);
    } else {
        println!("   'developer'エクステンションを追加しました");
    }
    
    // 5. 外部エクステンションの追加方法
    println!("5. 外部エクステンションの追加方法（実行はしません）");
    println!("   実行例: session.add_extension(\"path/to/extension\").await?;");
    
    // 6. メモリーの統合過程を説明
    println!("6. メモリー統合の流れ");
    println!("   a. build_session()関数内でエクステンションが読み込まれます");
    println!("   b. ExtensionManager::get_all()でインストール済みの拡張機能が読み込まれます");
    println!("   c. memory拡張機能もロードされ、.goose/memory/と~/.config/goose/memory/からメモリーを読み込みます");
    println!("   d. Session::new()が呼ばれ、セッションとメッセージ履歴が初期化されます");
    
    // 7. メッセージ処理の例
    println!("7. メッセージ処理の例");
    let message = "Gooseのセッション管理について教えてください";
    println!("   メッセージ: {}", message);
    println!("   (メッセージ処理は複雑なためシミュレーションのみです)");
    
    // メッセージを追加するだけで、実際の処理は行わない
    let user_message = Message::user().with_text(message);
    
    // 8. セッション保存の説明
    println!("8. セッション保存の仕組み");
    println!("   セッション終了時やセッション中断時に、メッセージ履歴が自動的に保存されます");
    println!("   保存先: ~/.goose/sessions/またはセッションIDで指定されたパス");
    println!("   次回起動時にresume=trueでセッションを再開できます");
    
    // 9. 「〇〇を思い出して」処理の流れ
    println!("\n9. 「〇〇を思い出して」処理の流れ");
    println!("    a. ユーザーが「〇〇を思い出して」と入力");
    println!("    b. session.process_message()が呼ばれ、メッセージがLLMに渡される");
    println!("    c. LLMが「思い出して」キーワードを認識し、memory__retrieve_memoriesツールを呼び出す");
    println!("    d. メモリー拡張機能がカテゴリに基づいてメモリーを検索");
    println!("    e. 結果がLLMに返され、ユーザー向けに整形された応答が生成される");
    
    // 10. 「〇〇を覚えておいて」処理の流れ
    println!("\n10. 「〇〇を覚えておいて」処理の流れ");
    println!("    a. ユーザーが「〇〇を覚えておいて」と入力");
    println!("    b. session.process_message()が呼ばれ、メッセージがLLMに渡される");
    println!("    c. LLMが「覚えておいて」キーワードを認識し、確認メッセージを生成");
    println!("    d. 確認完、memory__remember_memoryツールを呼び出し、メモリーを保存");
    println!("    e. カテゴリとスコープに基づいて適切なディレクトリに保存");
    println!("    f. 保存確認がユーザーに表示される");
    
    println!("\n🦢 サンプル実行終了");
    Ok(())
}