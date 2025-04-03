//! セッション構築とエクステンション管理のサンプル
//! このサンプルでは、以下の機能を説明します：
//! - build_session関数の使用方法
//! - add_extension関数によるエクステンションの追加
//! - get_system_prompt関数によるシステムプロンプトの取得
//! - メモリーの統合方法
//! 
//! 実行方法: cargo run --package goose-cli --example session_builder

use goose::{
    agent::Agent,
    config::{Config, ProviderConfig},
    core::{types::Message, AgentResponse},
    models::provider_config::ModelParams,
    models::provider_config::Provider,
    providers,
    session::BuildSessionOptions,
};
use goose_cli::{
    config::load_config,
    session::{build_session, Session},
};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🦢 Gooseセッションビルダーの例");
    
    // 1. 設定の読み込み
    println!("1. 設定を読み込み中...");
    let config = load_config(None).await?;
    println!("   設定読み込み完了: プロバイダー={:?}", config.provider);
    
    // 2. セッションの構築
    println!("2. セッションを構築中...");
    let session_file = PathBuf::from("/tmp/goose_session_example.json");
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
    
    // 3. 組み込みエクステンションの追加
    println!("3. 組み込みエクステンションを追加中...");
    session.add_builtin("developer".to_string()).await?;
    println!("   'developer'エクステンションを追加しました");
    
    // 4. 外部エクステンションの追加（ここではダミーとして）
    println!("4. 外部エクステンションの追加方法（実行はしません）");
    println!("   実行例: session.add_extension(\"path/to/extension\").await?;");
    
    // 5. システムプロンプトの取得と表示
    println!("5. システムプロンプトを取得中...");
    let system_prompt = session.get_system_prompt()?;
    println!("   システムプロンプト取得完了。長さ: {} 文字", system_prompt.len());
    println!("   プロンプトの冒頭部分: {}", &system_prompt[..std::cmp::min(100, system_prompt.len())]);
    
    // 6. メモリーの統合過程を説明
    println!("6. メモリー統合の流れ");
    println!("   a. session.build_session()内で拡張機能とシステムプロンプトの設定が行われる");
    println!("   b. load_extensions()で.goose/memory/と~/.config/goose/memory/からメモリーを読み込む");
    println!("   c. memory拡張機能が初期化され、読み込まれたメモリーがシステムプロンプトに統合される");
    println!("   d. ユーザーが「〇〇を思い出して」と言うと、memory拡張機能のretrieve_memories()が呼ばれる");
    
    // 7. メッセージ処理の例
    println!("7. メッセージ処理の例");
    let response = session
        .process_message("Gooseのセッション管理について教えてください".to_string())
        .await?;
    
    match response {
        AgentResponse::Message(content) => {
            println!("   AIの応答: {}", content);
        }
        AgentResponse::ToolRequest(request) => {
            println!("   ツールリクエスト: {:?}", request);
        }
        _ => {
            println!("   その他の応答: {:?}", response);
        }
    }
    
    // 8. セッション保存の説明
    println!("8. セッション保存の仕組み");
    println!("   セッション終了時やセッション中断時に、メッセージ履歴が自動的に保存されます");
    println!("   保存先: {:?}", session_file);
    println!("   次回起動時にresume=trueでセッションを再開できます");
    
    // 9. カスタムクライアントモデルの使用例
    println!("9. カスタムクライアントの設定例");
    let custom_config = create_custom_config();
    println!("   カスタム設定: {:?}", custom_config);
    
    println!("\n🦢 サンプル実行終了");
    Ok(())
}

// カスタム設定の作成例
fn create_custom_config() -> Config {
    let provider_config = ProviderConfig {
        provider: Provider::OpenAI,
        model: Some("gpt-4o".to_string()),
        params: Some(ModelParams::default()),
        ..Default::default()
    };
    
    Config {
        provider: provider_config,
        ..Default::default()
    }
}