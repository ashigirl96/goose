//! 「〇〇を思い出してください」の処理フローを説明するサンプル
//! このサンプルでは、メモリー拡張機能の処理フローと
//! ユーザーの「〇〇を思い出して」リクエストがどのように処理されるかを示します。
//!
//! 実行方法: cargo run --package goose-cli --example memory_recall

use goose::{
    agent::Agent,
    config::{Config, ProviderConfig},
    core::{types::Message, AgentResponse, ToolCall, ToolResponse},
    models::provider_config::ModelParams,
    models::provider_config::Provider,
    session::BuildSessionOptions,
};
use goose_cli::{
    config::load_config,
    session::{build_session, Session},
};
use std::path::PathBuf;
use std::sync::Arc;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦢 Gooseメモリーリコール処理のサンプル");
    
    // 1. 設定の読み込みとセッションの構築
    println!("1. セッションを構築中...");
    let config = load_config(None).await?;
    let session_file = PathBuf::from("/tmp/goose_memory_example.json");
    let mut session = build_session(
        &config,
        BuildSessionOptions {
            session_file: Some(session_file.clone()),
            resume: None,
            debug: true,
        },
    )
    .await?;
    println!("   セッションの構築完了");
    
    // 2. memory拡張機能の追加
    // 実際にはGooseのシステム拡張機能として自動的に追加されます
    println!("2. memory拡張機能のロードについて");
    println!("   実際のアプリケーションでは、memory拡張機能はシステム拡張機能として自動的に追加されます");
    println!("   内部処理では: add_builtin_extension(\"memory\")が呼ばれます");
    
    // 3. システムプロンプトの取得
    println!("3. システムプロンプトにメモリーが統合される仕組み");
    println!("   a. build_session()内でget_system_prompt()が呼ばれる");
    println!("   b. session::build_system_prompt()が実行され、ベースプロンプトと拡張機能情報が結合される");
    println!("   c. memory拡張機能により読み込まれたメモリーがシステムプロンプトに追加される");
    
    // 4. 「〇〇を思い出して」処理のシミュレーション
    println!("4. 「〇〇を思い出して」処理のフロー");
    println!("   フローのシミュレーションを行います");
    
    // ユーザーメッセージ例
    let user_message = "Gooseのメモリー実装について思い出してください";
    println!("   ユーザー: {}", user_message);
    
    // 5. メッセージ処理とツール呼び出しのシミュレーション
    println!("5. メッセージ処理の流れ:");
    println!("   a. ユーザーメッセージがsession.process_message()に渡される");
    println!("   b. LLMが「思い出して」というキーワードを認識する");
    println!("   c. LLMがretrieve_memoriesツールを呼び出す判断を行う");
    println!("   d. ツール形式のレスポンスが返ってくる");
    
    // ツール呼び出しのモック
    let tool_call = ToolCall {
        id: "call_123".to_string(),
        name: "memory__retrieve_memories".to_string(),
        arguments: json!({
            "category": "goose_memory",
            "is_global": true
        }),
    };
    println!("   ツール呼び出し: {:?}", tool_call);
    
    // ツール実行結果のモック
    let tool_response = ToolResponse {
        call_id: "call_123".to_string(),
        content: json!([
            "# Gooseのメモリー実装について\n",
            "- ローカルメモリーは.goose/memory/ディレクトリに保存される\n",
            "- グローバルメモリーは~/.config/goose/memory/ディレクトリに保存される\n",
            "- 各カテゴリは別々のMarkdownファイルとして保存される\n"
        ]),
    };
    println!("   ツール実行結果: {}", tool_response.content);
    
    // 6. LLMの最終応答用のモック
    println!("6. LLMの応答生成処理");
    println!("   a. ツール実行結果がLLMに渡される");
    println!("   b. LLMがツール結果を元に応答を生成");
    println!("   c. 生成された応答がユーザーに表示される");
    
    let final_response = "Gooseのメモリー実装について、以下のような情報があります：\n\n"
                        + "- ローカルメモリーは`.goose/memory/`ディレクトリに保存され、プロジェクト固有の情報を保持します。\n"
                        + "- グローバルメモリーは`~/.config/goose/memory/`ディレクトリに保存され、すべてのプロジェクトで共通して使用されます。\n"
                        + "- 各カテゴリは個別のMarkdownファイルとして保存され、構造化されたデータとして管理されます。\n\n"
                        + "これらのメモリーはセッション開始時に読み込まれ、システムプロンプトに統合されます。ユーザーが「思い出して」というキーワードを使用すると、retrieve_memoriesツールが呼び出され、関連するメモリーが取得されます。";
    
    println!("   最終応答: {}", final_response);
    
    // 7. まとめ
    println!("\n7. 全体まとめ");
    println!("   1) セッション構築時、memory拡張機能がロードされ、ローカルとグローバルのメモリーが読み込まれる");
    println!("   2) システムプロンプト内にメモリー一覧が統合される");
    println!("   3) ユーザーが「思い出して」と言うと、LLMがキーワードを認識する");
    println!("   4) LLMがretrieve_memoriesツールを呼び出す");
    println!("   5) ツールがカテゴリとスコープに基づいてメモリーを取得");
    println!("   6) 結果がLLMに返され、ユーザー向けに整形された応答が生成される");
    
    println!("\n🦢 サンプル実行終了");
    Ok(())
}