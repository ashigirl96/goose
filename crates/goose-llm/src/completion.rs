use anyhow::Result;
use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

use goose::message::Message;
use goose::model::ModelConfig;
use goose::providers::create;
use goose::providers::errors::ProviderError;

use std::time::Instant;

use crate::prompt_template;
use crate::{CompletionResponse, Extension, RuntimeMetrics};

/// Public API for the Goose LLM completion function
pub async fn completion(
    provider: &str,
    model_config: ModelConfig,
    system_preamble: &str,
    messages: &[Message],
    extensions: &[Extension],
) -> Result<CompletionResponse, ProviderError> {
    let start_total = Instant::now();
    let provider = create(provider, model_config).unwrap();
    let system_prompt = construct_system_prompt(system_preamble, extensions);
    // println!("\nSystem prompt: {}\n", system_prompt);

    let tools = extensions
        .iter()
        .flat_map(|ext| ext.get_prefixed_tools())
        .collect::<Vec<_>>();

    let start_provider = Instant::now();
    let (response, usage) = provider.complete(&system_prompt, messages, &tools).await?;
    let total_time_ms_provider = start_provider.elapsed().as_millis();
    let total_time_ms = start_total.elapsed().as_millis();

    let tokens_per_second = usage.usage.total_tokens.and_then(|toks| {
        if total_time_ms_provider > 0 {
            Some(toks as f64 / (total_time_ms_provider as f64 / 1000.0))
        } else {
            None
        }
    });

    let runtime_metrics =
        RuntimeMetrics::new(total_time_ms, total_time_ms_provider, tokens_per_second);

    let result = CompletionResponse::new(response.clone(), usage.clone(), runtime_metrics);

    Ok(result)
}

fn construct_system_prompt(system_preamble: &str, extensions: &[Extension]) -> String {
    let mut context: HashMap<&str, Value> = HashMap::new();

    context.insert(
        "system_preamble",
        Value::String(system_preamble.to_string()),
    );
    context.insert("extensions", serde_json::to_value(extensions).unwrap());

    let current_date_time = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    context.insert("current_date_time", Value::String(current_date_time));

    prompt_template::render_global_file("system.md", &context).expect("Prompt should render")
}
