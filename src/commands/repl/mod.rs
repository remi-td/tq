//! REPL (Read-Eval-Print Loop) command implementation
//!
//! Provides an interactive SQL shell for Teradata databases.
//! Features:
//! - Multi-line SQL input with semicolon termination
//! - In-memory command history with arrow key navigation
//! - Metacommands for session management (/quit, /help, /session)
//! - Graceful Ctrl-C handling

mod executor;
mod metacommands;
mod prompt;
mod state;

use crate::cli::ReplArgs;
use crate::db::DatabaseClient;
use crate::error::Result;
use reedline::{Reedline, Signal};
use std::io::Write;

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
    print_banner(client, writer)?;

    // Initialize reedline editor
    let mut editor = create_editor(args)?;

    // Create prompt
    let prompt = TqPrompt::new();

    // Main REPL loop
    repl_loop(&mut editor, client, &mut state, &prompt, writer, use_color)?;

    // Clean exit
    writeln!(writer, "Goodbye!")?;
    Ok(())
}

/// Create and configure the reedline editor
fn create_editor(args: &ReplArgs) -> Result<Reedline> {
    let mut editor = Reedline::create();

    // Configure history if enabled
    if !args.no_history {
        // For MVP, we use in-memory history only
        // Persistent history will be added in Phase 2
        editor = editor.with_history_exclusion_prefix(Some("/".to_string()));
    }

    Ok(editor)
}

/// Print the startup banner with connection information
fn print_banner<W: Write>(client: &DatabaseClient, writer: &mut W) -> Result<()> {
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

                    // Execute the SQL
                    match execute_sql(client, &sql, writer, use_color) {
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
