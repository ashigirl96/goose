---
sidebar_position: 3
---

# MCP Architecture & Tool Selection

This document describes the Message Control Protocol (MCP) architecture in Goose and how the system processes user instructions to determine which tools to invoke.

## Overview

The Message Control Protocol (MCP) is a fundamental component of Goose that enables communication between the main application and various extensions. MCP provides a standardized way to:

1. Define the capabilities of extensions through tools
2. Process user instructions to select the appropriate extension and tool
3. Execute tool calls and return results to the conversation flow

## MCP Communication Flow

The MCP architecture follows a client-server model:

1. **MCP Servers**: Each extension runs as an MCP server, providing tools and capabilities
2. **MCP Clients**: The main Goose application maintains client connections to each extension
3. **JSON-RPC Protocol**: Communication between clients and servers uses JSON-RPC messages

```
User Input → Goose Core → LLM → Tool Selection → MCP Client → MCP Server (Extension) → Tool Execution
```

## Tool Selection Process

When a user gives an instruction to Goose, the system follows a two-stage process to determine which extension and tool to use:

### Stage 1: LLM-Based Tool Selection

1. **Tool Information Collection**:
   - Goose collects all available tools from all active extensions via `capabilities.get_prefixed_tools()`
   - Each tool is prefixed with its extension name (e.g., `developer__shell`)
   - Tools include name, description, and parameter schema

2. **LLM Processing**:
   - The user's instruction, system prompt, and tool information are passed to the LLM via `provider().complete(...)`
   - The LLM analyzes the instruction and determines the most appropriate tool to use
   - The provider-specific format converters (e.g., `google.rs`, `openai.rs`) handle the API-specific formats

3. **Response Parsing**:
   - The LLM's response is processed by provider-specific parsers (e.g., `response_to_message()`)
   - If the LLM decides to use a tool, it includes a tool call in its response
   - This is converted to a `MessageContent::ToolRequest` with the tool name and arguments

Example code for converting an LLM's tool call (from Google AI):

```rust
if let Some(function_call) = part.get("functionCall") {
    let id = generate_random_id();
    let name = function_call["name"].as_str().unwrap_or_default().to_string();
    
    if !is_valid_function_name(&name) {
        // Handle invalid function name
    } else {
        if let Some(params) = function_call.get("args") {
            content.push(MessageContent::tool_request(
                id,
                Ok(ToolCall::new(&name, params.clone())),
            ));
        }
    }
}
```

### Stage 2: MCP Client/Tool Resolution

Once the LLM identifies a tool to use, Goose must determine which extension handles that tool:

1. **Extension Identification**:
   - The `dispatch_tool_call` method in `capabilities.rs` processes the tool call
   - It uses the tool name prefix to identify which extension should handle the call
   - The method `get_client_for_tool` searches for a matching MCP client:

```rust
fn get_client_for_tool(&self, prefixed_name: &str) -> Option<(&str, McpClientBox)> {
    self.clients
        .iter()
        .find(|(key, _)| prefixed_name.starts_with(*key))
        .map(|(name, client)| (name.as_str(), Arc::clone(client)))
}
```

2. **Tool Name Extraction**:
   - Once the extension is identified, the extension-specific tool name is extracted:

```rust
let tool_name = tool_call
    .name
    .strip_prefix(client_name)
    .and_then(|s| s.strip_prefix("__"))
    .ok_or_else(|| ToolError::NotFound(tool_call.name.clone()))?;
```

3. **Tool Execution**:
   - The identified MCP client is used to call the specific tool
   - Parameters from the LLM are passed to the tool

```rust
let client_guard = client.lock().await;
client_guard
    .call_tool(tool_name, tool_call.clone().arguments)
    .await
    .map(|result| result.content)
    .map_err(|e| ToolError::ExecutionError(e.to_string()))
```

## MCP Server Implementations

MCP servers can be implemented in three ways:

1. **Built-in**: Extensions embedded within the Goose binary
   - Run as separate processes, but use the same executable
   - Communication via standard I/O

2. **Stdio**: External extensions communicating via standard I/O
   - Can be implemented in any language
   - Run as child processes

3. **Server-Sent Events (SSE)**: RESTful HTTP endpoints
   - Remote extensions that can run on different machines
   - Communicate via HTTP streaming

## Example Flow

Here's an example of how the entire process works when a user asks Goose to list files:

1. User types: "List all files in the current directory"
2. LLM analyzes this request and decides to use the `developer__shell` tool
3. LLM response includes a tool call: `{name: "developer__shell", args: {command: "ls -la"}}`
4. Goose processes this response and extracts the tool request
5. `dispatch_tool_call` identifies:
   - Extension: `developer`
   - Tool: `shell`
6. The `developer` MCP client is used to call the `shell` tool with `ls -la` argument
7. The shell tool executes the command and returns the result
8. Results are added to the conversation as a `ToolResponse`
9. Conversation continues with the tool results included

## Benefits of MCP Architecture

1. **Modularity**: Extensions can be developed and deployed independently
2. **Flexibility**: Extensions can be implemented in any language
3. **Security**: Extensions run as separate processes with controlled communication
4. **Consistency**: Standardized protocol for all tool interactions
5. **Discoverability**: LLM can discover and use tools based on descriptions

This architecture allows Goose to dynamically adapt to user requests while maintaining a clean separation between the core system and its extensions.