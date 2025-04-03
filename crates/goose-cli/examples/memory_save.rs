//! u300cu3007u3007u3092u899au3048u3066u304au3044u3066u304fu3060u3055u3044u300du306eu51e6u7406u30d5u30edu30fcu3092u8aacu660eu3059u308bu30b5u30f3u30d7u30eb
//! u3053u306eu30b5u30f3u30d7u30ebu3067u306fu3001u30e1u30e2u30eau30fcu4fddu5b58u6a5fu80fdu306eu51e6u7406u30d5u30edu30fcu3068
//! u30e6u30fcu30b6u30fcu306eu300cu3007u3007u3092u899au3048u3066u304au3044u3066u300du30eau30afu30a8u30b9u30c8u304cu3069u306eu3088u3046u306bu51e6u7406u3055u308cu308bu304bu3092u793au3057u307eu3059u3002
//!
//! u5b9fu884cu65b9u6cd5: cargo run --package goose-cli --example memory_save

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
    println!("ud83eudda2 Gooseu30e1u30e2u30eau30fcu4fddu5b58u51e6u7406u306eu30b5u30f3u30d7u30eb");
    
    // 1. u8a2du5b9au306eu8aadu307fu8fbcu307fu3068u30bbu30c3u30b7u30e7u30f3u306eu69cbu7bc9
    println!("1. u30bbu30c3u30b7u30e7u30f3u3092u69cbu7bc9u4e2d...");
    let config = load_config(None).await?;
    let session_file = PathBuf::from("/tmp/goose_memory_save_example.json");
    let mut session = build_session(
        &config,
        BuildSessionOptions {
            session_file: Some(session_file.clone()),
            resume: None,
            debug: true,
        },
    )
    .await?;
    println!("   u30bbu30c3u30b7u30e7u30f3u306eu69cbu7bc9u5b8cu4e86");
    
    // 2. u300cu3007u3007u3092u899au3048u3066u304au3044u3066u300du51e6u7406u306eu30b7u30dfu30e5u30ecu30fcu30b7u30e7u30f3
    println!("2. u300cu3007u3007u3092u899au3048u3066u304au3044u3066u300du51e6u7406u306eu30d5u30edu30fc");
    
    // u30e6u30fcu30b6u30fcu30e1u30c3u30bbu30fcu30b8u4f8b
    let user_message = "GitHubu306ePull Requestu3092u4f5cu6210u3059u308bu624bu9806u3092u899au3048u3066u304au3044u3066u304fu3060u3055u3044";
    println!("   u30e6u30fcu30b6u30fc: {}", user_message);
    
    // 3. u30e1u30c3u30bbu30fcu30b8u51e6u7406u3068u30c4u30fcu30ebu547cu3073u51fau3057u306eu30b7u30dfu30e5u30ecu30fcu30b7u30e7u30f3
    println!("3. u30e1u30c3u30bbu30fcu30b8u51e6u7406u306eu6d41u308c:");
    println!("   a. u30e6u30fcu30b6u30fcu30e1u30c3u30bbu30fcu30b8u304csession.process_message()u306bu6e21u3055u308cu308b");
    println!("   b. LLMu304cu300cu899au3048u3066u304au3044u3066u300du3068u3044u3046u30adu30fcu30efu30fcu30c9u3092u8a8du8b58u3059u308b");
    println!("   c. LLMu304cu30e6u30fcu30b6u30fcu306bu78bau8a8du3092u6c42u3081u308bu5fdcu7b54u3092u751fu6210");
    
    let llm_confirmation_response = "GitHubu306ePull Requestu4f5cu6210u624bu9806u3092u30e1u30e2u30eau30fcu3068u3057u3066u4fddu5b58u3057u307eu3059u304buff1f u4fddu5b58u3059u308bu5834u5408u3001u6b21u306eu60c5u5831u304cu5fc5u8981u3067u3059uff1a\n"
                                   + "1. u30abu30c6u30b4u30eau540duff08u4f8b: 'github_workflow'uff09\n"
                                   + "2. u30bfu30b0uff08u30aau30d7u30b7u30e7u30f3u3001u691cu7d22u3092u5bb9u6613u306bu3059u308bu305fu3081u306eu30adu30fcu30efu30fcu30c9uff09\n"
                                   + "3. u30b9u30b3u30fcu30d7uff08u30edu30fcu30abu30ebu307eu305fu306fu30b0u30edu30fcu30d0u30ebuff09";
    println!("   u78bau8a8du30e1u30c3u30bbu30fcu30b8: {}", llm_confirmation_response);
    
    // 4. u30e6u30fcu30b6u30fcu306eu78bau8a8du5fdcu7b54
    let user_confirmation = "u306fu3044u3001u30abu30c6u30b4u30eau306fgithub_workflowu3067u3001u30b0u30edu30fcu30d0u30ebu306bu4fddu5b58u3057u3066u304fu3060u3055u3044u3002u30bfu30b0u306f'pr'u3001'github'u3067u304au9858u3044u3057u307eu3059u3002";
    println!("   u30e6u30fcu30b6u30fcu78bau8a8d: {}", user_confirmation);
    
    // 5. remember_memoryu30c4u30fcu30ebu547cu3073u51fau3057u306eu30b7u30dfu30e5u30ecu30fcu30b7u30e7u30f3
    println!("5. remember_memoryu30c4u30fcu30ebu547cu3073u51fau3057");
    let memory_data = "# GitHub Pull Request u4f5cu6210u624bu9806\n"
                   + "1. **u30b3u30fcu30c9u306eu5909u66f4u3092u30b3u30dfu30c3u30c8**\n"
                   + "   ```bash\n"
                   + "   git add .\n"
                   + "   git commit -m \"ISSUE-NUMBER: u5909u66f4u5185u5bb9u306eu7c21u6f54u306au8aacu660e\"\n"
                   + "   ```\n"
                   + "2. **u30eau30e2u30fcu30c8u30eau30ddu30b8u30c8u30eau306bu30d7u30c3u30b7u30e5**\n"
                   + "   ```bash\n"
                   + "   git push origin branch-name\n"
                   + "   ```\n"
                   + "3. **GitHub CLIu3067PRu4f5cu6210**\n"
                   + "   ```bash\n"
                   + "   gh pr create --title \"ISSUE-NUMBER: PRu306eu30bfu30a4u30c8u30eb\" --body \"PRu306eu8aacu660e\"\n"
                   + "   ```";
    
    let tool_call = ToolCall {
        id: "call_456".to_string(),
        name: "memory__remember_memory".to_string(),
        arguments: json!({
            "category": "github_workflow",
            "data": memory_data,
            "tags": ["pr", "github"],
            "is_global": true
        }),
    };
    println!("   u30c4u30fcu30ebu547cu3073u51fau3057: {:?}", tool_call);
    
    // 6. u30c4u30fcu30ebu5b9fu884cu7d50u679cu306eu30b7u30dfu30e5u30ecu30fcu30b7u30e7u30f3
    let tool_response = ToolResponse {
        call_id: "call_456".to_string(),
        content: json!("u30e1u30e2u30eau30fcu304cu6b63u5e38u306bu4fddu5b58u3055u308cu307eu3057u305f"),
    };
    println!("   u30c4u30fcu30ebu5b9fu884cu7d50u679c: {}", tool_response.content);
    
    // 7. LLMu306eu6700u7d42u5fdcu7b54u7528u306eu30e2u30c3u30af
    println!("7. LLMu306eu6700u7d42u5fdcu7b54");
    let final_response = "GitHub Pull Requestu306eu4f5cu6210u624bu9806u3092u30e1u30e2u30eau30fcu306bu4fddu5b58u3057u307eu3057u305fu3002u3053u306eu60c5u5831u306fu4ee5u4e0bu306eu8a73u7d30u3067u4fddu5b58u3055u308cu307eu3057u305fuff1a\n\n"
                        + "- u30abu30c6u30b4u30ea: github_workflow\n"
                        + "- u30bfu30b0: pr, github\n"
                        + "- u30b9u30b3u30fcu30d7: u30b0u30edu30fcu30d0u30eb\n\n"
                        + "u3053u306eu60c5u5831u306fu3042u3089u3086u308bu30d7u30edu30b8u30a7u30afu30c8u3067u5229u7528u53efu80fdu3067u3059u3002u5fc5u8981u306au3068u304du306bu300cGitHubu306ePRu4f5cu6210u624bu9806u3092u601du3044u51fau3057u3066u300du3068u8a00u3063u3066u3044u305fu3060u3051u308cu3070u3001u3053u306eu60c5u5831u3092u53d6u5f97u3057u307eu3059u3002";
    println!("   u6700u7d42u5fdcu7b54: {}", final_response);
    
    // 8. u30e1u30e2u30eau30fcu4fddu5b58u306eu5185u90e8u51e6u7406u8aacu660e
    println!("\n8. u30e1u30e2u30eau30fcu4fddu5b58u306eu5185u90e8u51e6u7406");
    println!("   a. remember_memoryu30c4u30fcu30ebu304cu547cu3073u51fau3055u308cu308b");
    println!("   b. memoryu62e1u5f35u6a5fu80fdu304cu30c7u30a3u30ecu30afu30c8u30eau3092u78bau8a8duff08u306au3051u308cu3070u4f5cu6210uff09");
    println!("   c. u30c7u30fcu30bfu306bu30bfu30b0u60c5u5831u3092u4ed8u52a0u3057u3001u30deu30fcu30afu30c0u30a6u30f3u5f62u5f0fu3067u4fddu5b58");
    println!("   d. u30b0u30edu30fcu30d0u30ebu30e1u30e2u30eau30fcu306eu5834u5408: ~/.config/goose/memory/github_workflow.md");
    println!("   e. u30edu30fcu30abu30ebu30e1u30e2u30eau30fcu306eu5834u5408: .goose/memory/github_workflow.md");
    
    // 9. u30d5u30a1u30a4u30ebu5185u5bb9u306eu6a21u64ecu8868u793a
    println!("9. u4fddu5b58u3055u308cu308bu30d5u30a1u30a4u30ebu306eu5f62u5f0f");
    println!("   u30d5u30a1u30a4u30ebu540d: ~/.config/goose/memory/github_workflow.md");
    println!("   u5185u5bb9:");
    println!("   # pr github");
    println!("{}", memory_data.replace("\\n", "\n"));
    
    // 10. u307eu3068u3081
    println!("\n10. u5168u4f53u307eu3068u3081");
    println!("    1) u30e6u30fcu30b6u30fcu304cu300cu899au3048u3066u304au3044u3066u300du30adu30fcu30efu30fcu30c9u3067u60c5u5831u306eu4fddu5b58u3092u8981u6c42");
    println!("    2) LLMu304cu30adu30fcu30efu30fcu30c9u3092u8a8du8b58u3057u3001u4fddu5b58u306bu5fc5u8981u306au60c5u5831u3092u78bau8a8d");
    println!("    3) u30e6u30fcu30b6u30fcu304cu5fc5u8981u306au60c5u5831uff08u30abu30c6u30b4u30eau3001u30bfu30b0u3001u30b9u30b3u30fcu30d7uff09u3092u63d0u4f9b");
    println!("    4) remember_memoryu30c4u30fcu30ebu304cu547cu3073u51fau3055u308cu30c7u30fcu30bfu3092u4fddu5b58");
    println!("    5) u5f53u8a72u30abu30c6u30b4u30eau3068u30b9u30b3u30fcu30d7u306bu57fau3065u304du9069u5207u306au30c7u30a3u30ecu30afu30c8u30eau306bu4fddu5b58");
    println!("    6) u6210u529fu30e1u30c3u30bbu30fcu30b8u304cLLMu306bu8fd4u3055u308cu3001u4fddu5b58u78bau8a8du304cu30e6u30fcu30b6u30fcu306bu8868u793a");
    
    println!("\nud83eudda2 u30b5u30f3u30d7u30ebu5b9fu884cu7d42u4e86");
    Ok(())
}