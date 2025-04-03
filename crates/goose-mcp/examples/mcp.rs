// An example script to run an MCP server
use anyhow::Result;
use serde_json::json;
use goose_mcp::MemoryRouter;
use mcp_server::Router;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> Result<()> {
    // Set up file appender for logging
    let file_appender = RollingFileAppender::new(Rotation::DAILY, "logs", "goose-mcp-example.log");

    // Initialize the tracing subscriber with file and stdout logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .with_writer(file_appender)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tracing::info!("Starting MCP server");

    // Create an instance of our memory router
    let memory_router = MemoryRouter::new();
    println!("Router created with instructions: {}", memory_router.get_instructions());
    println!("Available tools: {}", serde_json::to_string_pretty(&memory_router.list_tools())?);
    println!("Available resources: {:?}", memory_router.list_resources());
    println!("Available prompts: {:?}", memory_router.list_prompts());
    
    // Retrieve memories using await since call_tool returns a Future
    let memories_result = memory_router.call_tool("retrieve_memories",  json!({
        "category": "github_workflow",
        "is_global": true,
    })).await;
    let pretty_json = serde_json::to_string_pretty(memories_result?.get(0).unwrap())?;
    println!("Memories: {}", pretty_json);
    
    // Note: The following code can be uncommented to run a full MCP server
    // instead of just demonstrating the memory router functionality
    //
    // // Create a router service wrapping our memory router
    // // let router_service = mcp_server::router::RouterService(memory_router);
    // // 
    // // // Create and run the server
    // // let server = mcp_server::Server::new(router_service);
    // // let transport = mcp_server::ByteTransport::new(tokio::io::stdin(), tokio::io::stdout());
    // //
    // // tracing::info!("Server initialized and ready to handle requests");
    // // server.run(transport).await?;
    
    Ok(())
}