use console::style;
use goose::agents::extension::ExtensionError;
use goose::agents::AgentFactory;
use goose::config::{Config, ExtensionManager};
use goose::session;
use goose::session::Identifier;
use mcp_client::transport::Error as McpClientError;
use std::process;

use super::output;
use super::Session;

pub async fn build_session(
    identifier: Option<Identifier>,
    resume: bool,
    extensions: Vec<String>,
    builtins: Vec<String>,
    debug: bool,
) -> Session {
    println!("DEBUG: build_session start");

    // Load config and get provider/model
    println!("DEBUG: Loading config");
    let config = Config::global();
    println!("DEBUG: Config loaded");

    println!("DEBUG: Getting provider name");
    let provider_name: String = config
        .get_param("GOOSE_PROVIDER")
        .expect("No provider configured. Run 'goose configure' first");
    println!("DEBUG: Provider name: {}", provider_name);

    println!("DEBUG: Getting model name");
    let model: String = config
        .get_param("GOOSE_MODEL")
        .expect("No model configured. Run 'goose configure' first");
    println!("DEBUG: Model name: {}", model);

    println!("DEBUG: Creating model config");
    let model_config = goose::model::ModelConfig::new(model.clone());
    println!("DEBUG: Creating provider");
    let provider =
        goose::providers::create(&provider_name, model_config).expect("Failed to create provider");
    println!("DEBUG: Provider created");

    // Create the agent
    println!("DEBUG: Creating agent");
    let mut agent = AgentFactory::create(&AgentFactory::configured_version(), provider)
        .expect("Failed to create agent");
    println!("DEBUG: Agent created");

    // Handle session file resolution and resuming
    println!("DEBUG: Handling session file resolution");
    let session_file = if resume {
        if let Some(identifier) = identifier {
            let session_file = session::get_path(identifier);
            if !session_file.exists() {
                output::render_error(&format!(
                    "Cannot resume session {} - no such session exists",
                    style(session_file.display()).cyan()
                ));
                process::exit(1);
            }

            session_file
        } else {
            // Try to resume most recent session
            match session::get_most_recent_session() {
                Ok(file) => file,
                Err(_) => {
                    output::render_error("Cannot resume - no previous sessions found");
                    process::exit(1);
                }
            }
        }
    } else {
        // Create new session with provided name/path or generated name
        let id = match identifier {
            Some(identifier) => identifier,
            None => Identifier::Name(session::generate_session_id()),
        };

        // Just get the path - file will be created when needed
        session::get_path(id)
    };
    println!("DEBUG: Session file resolved: {:?}", session_file);

    if resume {
        println!("DEBUG: Handling session resume");
        // Read the session metadata
        let metadata = session::read_metadata(&session_file).unwrap_or_else(|e| {
            output::render_error(&format!("Failed to read session metadata: {}", e));
            process::exit(1);
        });

        let current_workdir =
            std::env::current_dir().expect("Failed to get current working directory");
        if current_workdir != metadata.working_dir {
            // Ask user if they want to change the working directory
            let change_workdir = cliclack::confirm(format!("{} The working directory of this session was set to {}. It does not match the current working directory. Would you like to change it?", style("WARNING:").yellow(), style(metadata.working_dir.display()).cyan()))
            .initial_value(true)
            .interact().expect("Failed to get user input");

            if change_workdir {
                std::env::set_current_dir(metadata.working_dir).unwrap();
            }
        }
    }

    // Setup extensions for the agent
    println!("DEBUG: Setting up extensions");
    // Extensions need to be added after the session is created because we change directory when resuming a session
    for extension in ExtensionManager::get_all().expect("should load extensions") {
        println!("DEBUG: Loading extension: {}", extension.config.name());
        if extension.enabled {
            let config = extension.config.clone();
            println!("DEBUG: Adding extension: {}", config.name());
            agent
                .add_extension(config.clone())
                .await
                .unwrap_or_else(|e| {
                    let err = match e {
                        ExtensionError::Transport(McpClientError::StdioProcessError(inner)) => {
                            inner
                        }
                        _ => e.to_string(),
                    };
                    println!("Failed to start extension: {}, {:?}", config.name(), err);
                    println!(
                        "Please check extension configuration for {}.",
                        config.name()
                    );
                    process::exit(1);
                });
            println!("DEBUG: Extension added: {}", config.name());
        }
    }
    println!("DEBUG: Extensions setup completed");

    // Create new session
    println!("DEBUG: Creating session");
    let mut session = Session::new(agent, session_file.clone(), debug);
    println!("DEBUG: Session created");

    // Add extensions if provided
    println!("DEBUG: Adding provided extensions");
    for extension_str in extensions {
        println!("DEBUG: Adding extension: {}", extension_str);
        if let Err(e) = session.add_extension(extension_str.clone()).await {
            eprintln!("Failed to start extension: {}", e);
            process::exit(1);
        }
        println!("DEBUG: Extension added: {}", extension_str);
    }
    println!("DEBUG: Provided extensions added");

    // Add builtin extensions
    println!("DEBUG: Adding builtin extensions");
    for builtin in builtins {
        println!("DEBUG: Adding builtin: {}", builtin);
        if let Err(e) = session.add_builtin(builtin.clone()).await {
            eprintln!("Failed to start builtin extension: {}", e);
            process::exit(1);
        }
        println!("DEBUG: Builtin added: {}", builtin);
    }
    println!("DEBUG: Builtin extensions added");

    // Add CLI-specific system prompt extension
    println!("DEBUG: Extending system prompt");
    session
        .agent
        .extend_system_prompt(super::prompt::get_cli_prompt())
        .await;
    println!("DEBUG: System prompt extended");

    // Only override system prompt if a system override exists
    println!("DEBUG: Checking for system prompt override");
    let system_prompt_file: Option<String> = config.get_param("GOOSE_SYSTEM_PROMPT_FILE_PATH").ok();
    if let Some(ref path) = system_prompt_file {
        println!("DEBUG: Reading system prompt override from: {}", path);
        let override_prompt =
            std::fs::read_to_string(path).expect("Failed to read system prompt file");
        session.agent.override_system_prompt(override_prompt).await;
        println!("DEBUG: System prompt overridden");
    } else {
        println!("DEBUG: No system prompt override found");
    }

    println!("DEBUG: Displaying session info");
    output::display_session_info(resume, &provider_name, &model, &session_file);
    println!("DEBUG: Session build completed");
    session
}
