//! REPL (Read-Eval-Print Loop) command implementation
//!
//! Provides an interactive SQL shell for Teradata databases.
//! Features:
//! - Multi-line SQL input with semicolon termination
//! - Persistent command history (saved to ~/.tq_history)
//! - Vim and Emacs keybinding modes
//! - Metacommands for session management (/quit, /help, /session, /ping, /describe)
//! - Graceful Ctrl-C handling

mod executor;
mod metacommands;
mod prompt;
mod state;

use crate::cli::{EditorMode, ReplArgs};
use crate::db::DatabaseClient;
use crate::error::Result;
use reedline::{EditMode, Emacs, FileBackedHistory, Reedline, Signal, Vi};
use std::io::Write;
use std::path::PathBuf;

pub use executor::execute_sql;
pub use metacommands::handle_metacommand;
pub use prompt::TqPrompt;
pub use state::ReplState;

/// Execute the REPL command
pub fn execute<W: Write>(
    client: &DatabaseClient,
    args: &ReplArgs,
    writer: &mut W,
    use_color: bool,
    _verbose: bool,
) -> Result<()> {
    // Initialize state
    let mut state = ReplState::new(client.config().clone());

    // Show startup banner
    print_banner(client, args, writer)?;

    // Initialize reedline editor with persistent history and editor mode
    let mut editor = create_editor(args, writer)?;

    // Create prompt
    let prompt = TqPrompt::new();

    // Main REPL loop
    repl_loop(
        &mut editor,
        client,
        &mut state,
        &prompt,
        writer,
        use_color,
        args.default_limit,
    )?;

    // Clean exit
    writeln!(writer, "Goodbye!")?;
    Ok(())
}

/// Create and configure the reedline editor
fn create_editor(args: &ReplArgs, writer: &mut impl Write) -> Result<Reedline> {
    let mut editor = Reedline::create();

    // Configure persistent history if enabled
    if !args.no_history {
        let history_path = resolve_history_path(&args.history_file);

        match FileBackedHistory::with_file(10000, history_path.clone()) {
            Ok(history) => {
                editor = editor.with_history(Box::new(history));
                log::debug!("History file loaded: {}", history_path.display());
            }
            Err(e) => {
                // Warn but continue without persistent history
                let _ = writeln!(
                    writer,
                    "Warning: Cannot load history from {}: {}",
                    history_path.display(),
                    e
                );
                let _ = writeln!(writer, "History will be stored in memory only for this session.");
                log::warn!("Failed to load history file: {}", e);
            }
        }

        // Exclude metacommands from history
        editor = editor.with_history_exclusion_prefix(Some("/".to_string()));
    }

    // Configure editor mode (Vim or Emacs keybindings)
    let edit_mode: Box<dyn EditMode> = match args.editor_mode {
        EditorMode::Vi => Box::new(Vi::default()),
        EditorMode::Emacs => Box::new(Emacs::default()),
    };
    editor = editor.with_edit_mode(edit_mode);

    Ok(editor)
}

/// Resolve the history file path
///
/// Handles ~ expansion and environment variables
fn resolve_history_path(path: &PathBuf) -> PathBuf {
    // Check for environment variable override first
    if let Ok(env_path) = std::env::var("TQ_HISTORY_FILE") {
        return expand_tilde(&PathBuf::from(env_path));
    }

    expand_tilde(path)
}

/// Expand ~ to the user's home directory
fn expand_tilde(path: &PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();

    if path_str.starts_with("~/") || path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            if path_str == "~" {
                return home;
            }
            return home.join(&path_str[2..]);
        }
    }

    path.clone()
}

/// Home directory helper using the directories crate
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
    }
}

/// Print the startup banner with connection information
fn print_banner<W: Write>(
    client: &DatabaseClient,
    args: &ReplArgs,
    writer: &mut W,
) -> Result<()> {
    let config = client.config();

    writeln!(writer)?;
    writeln!(
        writer,
        "Connected to {}:{}",
        config.host, config.port
    )?;
    writeln!(writer, "Database: {}", config.database)?;
    writeln!(writer, "User: {}", config.user)?;
    writeln!(writer, "Logon Mechanism: {}", config.logmech)?;
    if args.default_limit > 0 {
        writeln!(writer, "Default row limit: {}", args.default_limit)?;
    }
    // Show editor mode
    let editor_mode_str = match args.editor_mode {
        EditorMode::Emacs => "emacs",
        EditorMode::Vi => "vi",
    };
    writeln!(writer, "Editor mode: {}", editor_mode_str)?;
    writeln!(writer)?;
    writeln!(writer, "Type /help for commands, /quit to exit.")?;
    writeln!(writer)?;

    Ok(())
}

/// Main REPL loop
fn repl_loop<W: Write>(
    editor: &mut Reedline,
    client: &DatabaseClient,
    state: &mut ReplState,
    prompt: &TqPrompt,
    writer: &mut W,
    use_color: bool,
    default_limit: usize,
) -> Result<()> {
    loop {
        // Get the appropriate prompt based on state
        let current_prompt = prompt.for_state(state);

        match editor.read_line(&current_prompt) {
            Ok(Signal::Success(line)) => {
                let trimmed = line.trim();

                // Empty line - just continue
                if trimmed.is_empty() {
                    continue;
                }

                // Check for metacommand
                if trimmed.starts_with('/') || trimmed.starts_with('\\') {
                    match handle_metacommand(trimmed, state, client, writer) {
                        Ok(should_continue) => {
                            if !should_continue {
                                break; // /quit was issued
                            }
                        }
                        Err(e) => {
                            writeln!(writer, "Error: {}", e)?;
                        }
                    }
                    continue;
                }

                // Accumulate SQL input
                state.append_input(&line);

                // Check if statement is complete (ends with semicolon)
                if state.input_buffer().trim_end().ends_with(';') {
                    let sql = state.take_input();

                    // Execute the SQL with default limit for SELECT queries
                    match execute_sql(client, &sql, writer, use_color, default_limit) {
                        Ok(row_count) => {
                            state.record_query(row_count);
                        }
                        Err(e) => {
                            // Print error but don't exit REPL
                            writeln!(writer, "\nError: {}", e)?;
                        }
                    }
                    writeln!(writer)?;
                }
            }

            Ok(Signal::CtrlC) => {
                if state.has_input() {
                    // Clear current input buffer
                    state.clear_input();
                    writeln!(writer, "^C")?;
                } else {
                    // Hint to use /quit
                    writeln!(writer, "\nUse /quit or Ctrl-D to exit.")?;
                }
            }

            Ok(Signal::CtrlD) => {
                if !state.has_input() {
                    // Exit on Ctrl-D with empty buffer
                    break;
                }
                // Ignore Ctrl-D when there's input in buffer
            }

            Err(e) => {
                // Handle read error (e.g., terminal issues)
                writeln!(writer, "Error reading input: {}", e)?;
                // Continue the loop - don't exit on transient errors
            }
        }
    }

    Ok(())
}
