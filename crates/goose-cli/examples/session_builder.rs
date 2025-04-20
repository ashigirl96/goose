use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use goose::config::Config;

use goose_cli::commands::configure::handle_configure;
use goose_cli::commands::mcp::run_server;
use goose_cli::logging::setup_logging;
use goose_cli::session;
use goose_cli::session::{build_session, SessionBuilderConfig};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, display_name = "", about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Args)]
#[group(required = false, multiple = false)]
struct Identifier {
    #[arg(
        short,
        long,
        value_name = "NAME",
        help = "Name for the chat session (e.g., 'project-x')",
        long_help = "Specify a name for your chat session. When used with --resume, will resume this specific session if it exists."
    )]
    name: Option<String>,

    #[arg(
        short,
        long,
        value_name = "PATH",
        help = "Path for the chat session (e.g., './playground.jsonl')",
        long_help = "Specify a path for your chat session. When used with --resume, will resume this specific session if it exists."
    )]
    path: Option<PathBuf>,
}

fn extract_identifier(identifier: Identifier) -> session::Identifier {
    if let Some(name) = identifier.name {
        session::Identifier::Name(name)
    } else if let Some(path) = identifier.path {
        session::Identifier::Path(path)
    } else {
        unreachable!()
    }
}

#[derive(Subcommand)]
enum SessionCommand {
    #[command(about = "List all available sessions")]
    List {
        #[arg(short, long, help = "List all available sessions")]
        verbose: bool,

        #[arg(
            short,
            long,
            help = "Output format (text, json)",
            default_value = "text"
        )]
        format: String,
    },
}

#[derive(Subcommand)]
enum Command {
    /// Manage system prompts and behaviors
    #[command(about = "Run one of the mcp servers bundled with goose")]
    Mcp { name: String },

    /// Start or resume interactive chat sessions
    #[command(
        about = "Start or resume interactive chat sessions",
        visible_alias = "s"
    )]
    Session {
        #[command(subcommand)]
        command: Option<SessionCommand>,
        /// Identifier for the chat session
        #[command(flatten)]
        identifier: Option<Identifier>,

        /// Resume a previous session
        #[arg(
            short,
            long,
            help = "Resume a previous session (last used or specified by --name)",
            long_help = "Continue from a previous chat session. If --name or --path is provided, resumes that specific session. Otherwise resumes the last used session."
        )]
        resume: bool,

        /// Enable debug output mode
        #[arg(
            long,
            help = "Enable debug output mode with full content and no truncation",
            long_help = "When enabled, shows complete tool responses without truncation and full paths."
        )]
        debug: bool,

        /// Add stdio extensions with environment variables and commands
        #[arg(
            long = "with-extension",
            value_name = "COMMAND",
            help = "Add stdio extensions (can be specified multiple times)",
            long_help = "Add stdio extensions from full commands with environment variables. Can be specified multiple times. Format: 'ENV1=val1 ENV2=val2 command args...'",
            action = clap::ArgAction::Append
        )]
        extension: Vec<String>,

        /// Add builtin extensions by name
        #[arg(
            long = "with-builtin",
            value_name = "NAME",
            help = "Add builtin extensions by name (e.g., 'developer' or multiple: 'developer,github')",
            long_help = "Add one or more builtin extensions that are bundled with goose by specifying their names, comma-separated",
            value_delimiter = ','
        )]
        builtin: Vec<String>,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum CliProviderVariant {
    OpenAi,
    Databricks,
    Ollama,
}

pub async fn cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Mcp { name }) => {
            println!("Running mcp server: {}", name);
            let _ = run_server(&name).await;
        }
        None => {
            println!("No command provided. Use --help for usage information.");
            if !Config::global().exists() {
                let _ = handle_configure().await;
                return Ok(());
            } else {
                // Run session command by default
                let mut session = build_session(SessionBuilderConfig {
                    identifier: None,
                    resume: false,
                    extensions: vec![],
                    remote_extensions: vec![],
                    builtins: vec![],
                    extensions_override: None,
                    additional_system_prompt: None,
                    debug: false,
                }).await;
                setup_logging(
                    session.session_file().file_stem().and_then(|s| s.to_str()),
                    None,
                )?;
                let _ = session.interactive(None).await;
                return Ok(());
            }
        }
        _ => {
            panic!("Unknown command");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    cli().await
}
