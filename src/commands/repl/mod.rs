//! REPL (Read-Eval-Print Loop) command implementation
//!
//! Provides an interactive SQL shell for Teradata databases.
//! Features:
//! - Multi-line SQL input with semicolon termination
//! - Multi-line command history - complete statements stored as single entries (Sprint 24)
//! - SQL syntax highlighting with customizable colors
//! - Persistent command history (saved to ~/.tq_history)
//! - Vim and Emacs keybinding modes
//! - Metacommands for session management (/quit, /help, /session, /ping, /describe, /logon)
//! - Result paging for large result sets
//! - Graceful Ctrl-C handling
//! - Context-aware tab completion (Sprint 7)

mod completer;
mod executor;
mod highlighter;
mod metacommands;
mod metadata_completer;
mod pager;
mod prompt;
mod sql_context;
mod state;
mod validator;

use crate::cli::{EditorMode, ReplArgs};
use crate::db::DatabaseClient;
use crate::error::Result;
use metadata_completer::{CompletionState, MetadataCompleter};
use nu_ansi_term::Color;
use reedline::{
    default_emacs_keybindings, default_vi_insert_keybindings, default_vi_normal_keybindings,
    ColumnarMenu, EditMode, Emacs, FileBackedHistory, KeyCode, KeyModifiers, Keybindings,
    MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal, Vi,
};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub use executor::{execute_sql, execute_sql_with_state, QueryTiming};
pub use highlighter::SqlHighlighter;
pub use metacommands::handle_metacommand;
pub use pager::{display_with_pager, should_page, PagerConfig};
pub use prompt::TqPrompt;
pub use sql_context::{analyze_context, CompletionContext, TableReference};
pub use state::ReplState;
pub use validator::SqlStatementValidator;

/// Execute the REPL command
///
/// Sprint 7: Updated to support metadata completion and /logon.
/// Sprint 20: Pre-loads database names BEFORE editor initialization to avoid
/// TTY conflicts during tab completion.
/// The client is now owned by a shared state that can be updated on reconnection.
pub fn execute<W: Write>(
    client: DatabaseClient,
    args: &ReplArgs,
    writer: &mut W,
    use_color: bool,
    _verbose: bool,
) -> Result<()> {
    // Create shared completion state (thread-safe for reedline)
    let database = client.config().database.clone();
    let completion_state = Arc::new(Mutex::new(CompletionState::new(client, database)));

    // Initialize REPL state
    let mut state = {
        let cs = completion_state.lock().unwrap();
        ReplState::new(cs.client().config().clone())
    };

    // Pre-load ALL metadata BEFORE editor initialization for faster tab completion.
    // This ensures tab completion is instant without needing database roundtrips.
    {
        let mut cs = completion_state.lock().unwrap();
        log::info!("Pre-loading metadata for tab completion...");

        // Load database names first
        if cs.ensure_databases_loaded() {
            log::info!("Database names loaded successfully");
        } else {
            log::warn!("Failed to pre-load database names; tab completion may be limited");
        }

        // Also pre-load tables in current database
        // This prevents queries during tab completion after "SELECT * FROM "
        if cs.ensure_tables_loaded() {
            log::info!("Table metadata loaded successfully");
        } else {
            log::warn!("Failed to pre-load table metadata; tab completion may be limited");
        }
    }

    // Show startup banner
    {
        let cs = completion_state.lock().unwrap();
        print_banner(&cs, args, writer)?;
    }

    // Initialize reedline editor with persistent history and editor mode
    // Note: Databases are already cached, so tab completion won't trigger queries
    let mut editor = create_editor(args, writer, Arc::clone(&completion_state))?;

    // Create prompt
    let prompt = TqPrompt::new();

    // Main REPL loop
    repl_loop(
        &mut editor,
        &completion_state,
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
///
/// Sprint 7: Now accepts shared completion state for metadata-aware completion.
/// Sprint 8: Fixed tab completion by adding ColumnarMenu and keybindings.
/// Sprint 24: Added SqlStatementValidator for multi-line history support.
fn create_editor(
    args: &ReplArgs,
    writer: &mut impl Write,
    completion_state: Arc<Mutex<CompletionState>>,
) -> Result<Reedline> {
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
                let _ = writeln!(
                    writer,
                    "History will be stored in memory only for this session."
                );
                log::warn!("Failed to load history file: {}", e);
            }
        }

        // Exclude metacommands from history
        editor = editor.with_history_exclusion_prefix(Some("/".to_string()));
    }

    // Sprint 24: Add validator for multi-line history support
    //
    // The SqlStatementValidator detects when SQL statements are complete (end with ';').
    // When the validator returns `Incomplete`, reedline:
    // - Does NOT save partial input to history
    // - Continues accepting input on new lines
    // - Accumulates all lines into a single buffer
    //
    // When `Complete` is returned, the ENTIRE buffer (including newlines) is saved
    // as one history entry. This achieves REQ-HIST-001 through REQ-HIST-007:
    // pressing UP arrow recalls complete multi-line statements, not individual lines.
    let statement_validator = validator::SqlStatementValidator::new();
    editor = editor.with_validator(Box::new(statement_validator));

    // Configure tab completion with metadata support (Sprint 7)
    // Sprint 8: Fixed by adding ColumnarMenu and proper keybindings
    let completer = MetadataCompleter::with_state(completion_state);
    editor = editor.with_completer(Box::new(completer));

    // Create a completion menu that shows suggestions
    // Sprint 20 Fix: Use ColumnarMenu instead of ListMenu.
    // ListMenu displays a pager banner "Page 1: records 0 - 0 total: 0" which appears
    // even when there are no completions, causing unwanted output during tab completion.
    // ColumnarMenu does not have this banner and provides a cleaner completion experience.
    let completion_menu = ColumnarMenu::default()
        .with_name("completion_menu")
        .with_columns(2) // Show completions in 2 columns for better readability
        .with_column_padding(4); // Add padding between columns
    editor = editor.with_menu(ReedlineMenu::EngineCompleter(Box::new(completion_menu)));

    // Configure editor mode with keybindings that include Tab completion
    let edit_mode: Box<dyn EditMode> = match args.editor_mode {
        EditorMode::Vi => {
            // Vi insert mode keybindings with Tab completion
            let mut insert_kb = default_vi_insert_keybindings();
            add_completion_keybinding(&mut insert_kb);
            // Vi normal mode keybindings
            let normal_kb = default_vi_normal_keybindings();
            Box::new(Vi::new(insert_kb, normal_kb))
        }
        EditorMode::Emacs => {
            let mut kb = default_emacs_keybindings();
            add_completion_keybinding(&mut kb);
            Box::new(Emacs::new(kb))
        }
    };
    editor = editor.with_edit_mode(edit_mode);

    // Configure syntax highlighting
    let highlighter = if args.no_syntax_highlight {
        highlighter::SqlHighlighter::disabled()
    } else {
        highlighter::SqlHighlighter::new()
    };
    editor = editor.with_highlighter(Box::new(highlighter));

    Ok(editor)
}

/// Add Tab keybinding for completion menu
fn add_completion_keybinding(keybindings: &mut Keybindings) {
    // Tab key triggers the completion menu
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );
}

/// Resolve the history file path
///
/// Handles ~ expansion and environment variables
fn resolve_history_path(path: &std::path::Path) -> PathBuf {
    // Check for environment variable override first
    if let Ok(env_path) = std::env::var("TQ_HISTORY_FILE") {
        return expand_tilde(std::path::Path::new(&env_path));
    }

    expand_tilde(path)
}

/// Expand ~ to the user's home directory
fn expand_tilde(path: &std::path::Path) -> PathBuf {
    let path_str = path.to_string_lossy();

    if path_str.starts_with("~/") || path_str == "~" {
        if let Some(home) = dirs::home_dir() {
            if path_str == "~" {
                return home;
            }
            return home.join(&path_str[2..]);
        }
    }

    path.to_path_buf()
}

/// Home directory helper using the directories crate
mod dirs {
    use std::path::PathBuf;

    pub fn home_dir() -> Option<PathBuf> {
        directories::BaseDirs::new().map(|d| d.home_dir().to_path_buf())
    }
}

/// Print the startup banner with connection information
///
/// Sprint 20: Fixed logo display - 9-line lowercase "tq" ASCII art with 't' in Teradata orange
/// and 'q' in white/default color. Information messages displayed to the RIGHT of the logo.
fn print_banner<W: Write>(
    completion_state: &CompletionState,
    args: &ReplArgs,
    writer: &mut W,
) -> Result<()> {
    let config = completion_state.client().config();

    // Teradata orange: #F37021, xterm-256 color 202
    let orange = Color::Fixed(202);

    // Build the info lines to display to the right of the logo
    let mut info_lines: Vec<String> = Vec::new();
    info_lines.push(format!("Teradata Query Tool v{}", env!("CARGO_PKG_VERSION")));
    info_lines.push(format!("Connected to {}:{}", config.host, config.port));
    info_lines.push(format!("Database: {}", config.database));
    info_lines.push(format!("User: {}", config.user));
    if args.default_limit > 0 {
        info_lines.push(format!("Default row limit: {}", args.default_limit));
    }
    let editor_mode_str = match args.editor_mode {
        EditorMode::Emacs => "emacs",
        EditorMode::Vi => "vi",
    };
    info_lines.push(format!("Editor mode: {}", editor_mode_str));

    // LOWERCASE "tq" ASCII art logo - 9 lines
    // Sprint 20: User's exact ASCII art specification from branding-guidelines.md
    // 't' portion (left side) is in Teradata orange, 'q' portion (right side) is default color
    //
    // Full logo:
    //  __
    // /\ \__
    // \ \ ,_\    __
    //  \ \ \/  /'__`\
    //   \ \ \_/\ \L\ \
    //    \ \__\ \___, \
    //     \/__/\/___/\ \
    //               \ \_\
    //                \/_/
    //
    // The 't' portion (lines 1-7 left side) is in orange
    // The 'q' portion (lines 3-9 right side) is in default color

    let logo_t = [
        " __",
        "/\\ \\__",
        "\\ \\ ,_\\",
        " \\ \\ \\/",
        "  \\ \\ \\_",
        "   \\ \\__",
        "    \\/__",
        "        ",
        "        ",
    ];

    let logo_q = [
        "",
        "",
        "    __",
        "  /'__`\\",
        "/\\ \\L\\ \\",
        "\\ \\___, \\",
        "/\\/___/\\ \\",
        "      \\ \\_\\",
        "       \\/_/",
    ];

    writeln!(writer)?;

    // Print each line of the logo with info to the right
    for (i, (t_part, q_part)) in logo_t.iter().zip(logo_q.iter()).enumerate() {
        // Combine 't' (orange) and 'q' (default)
        let t_colored = orange.bold().paint(*t_part);

        // Get info line if available (offset by 0 to align with first logo line)
        let info = info_lines.get(i).map(|s| s.as_str()).unwrap_or("");

        // Calculate spacing between logo and info (align info at consistent column)
        let logo_width = t_part.len() + q_part.len();
        let target_width = 18; // Ensure consistent alignment
        let padding = if logo_width < target_width {
            " ".repeat(target_width - logo_width)
        } else {
            "   ".to_string()
        };

        writeln!(writer, "{}{}{}   {}", t_colored, q_part, padding, info)?;
    }

    writeln!(writer)?;
    writeln!(writer, "Type /help for commands, /quit to exit.")?;
    writeln!(writer)?;

    Ok(())
}

/// Main REPL loop
///
/// Sprint 7: Updated to use shared completion state for metacommand handling.
/// Sprint 24: Simplified to leverage SqlStatementValidator for multi-line history.
///
/// With the validator in place, reedline handles multi-line accumulation:
/// - When validator returns `Incomplete`, reedline continues accepting input
/// - When validator returns `Complete`, reedline returns the full buffer
/// - Multi-line statements are saved as single history entries (REQ-HIST-001-007)
fn repl_loop<W: Write>(
    editor: &mut Reedline,
    completion_state: &Arc<Mutex<CompletionState>>,
    state: &mut ReplState,
    prompt: &TqPrompt,
    writer: &mut W,
    _use_color: bool,
    default_limit: usize,
) -> Result<()> {
    loop {
        // Sprint 9 Bug 2 Fix: Update accumulated buffer in completion state for multi-line context
        // Note: With Sprint 24 validator, reedline handles accumulation internally.
        // The state.input_buffer() is mainly used for prompt display now.
        if let Ok(mut state_lock) = completion_state.lock() {
            state_lock.set_accumulated_buffer(state.input_buffer().to_string());
        }

        // Get the appropriate prompt based on state
        let current_prompt = prompt.for_state(state);

        match editor.read_line(&current_prompt) {
            Ok(Signal::Success(buffer)) => {
                // Sprint 24: With the validator, `buffer` contains the complete input.
                // For SQL statements, this is the full multi-line statement (including newlines).
                // For metacommands, this is the single line.
                let trimmed = buffer.trim();

                // Empty buffer - just continue
                if trimmed.is_empty() {
                    // Clear any accumulated state (shouldn't happen with validator, but be safe)
                    state.clear_input();
                    continue;
                }

                // Check for metacommand
                if trimmed.starts_with('/') || trimmed.starts_with('\\') {
                    // Clear accumulated input state for metacommand
                    state.clear_input();

                    // Lock completion state for metacommand handling
                    let mut cs = completion_state.lock().unwrap();
                    match metacommands::handle_metacommand_with_state(
                        trimmed, state, &mut cs, writer,
                    ) {
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

                // Sprint 24: With validator, `buffer` IS the complete SQL statement.
                // The validator returns `Complete` only when statement ends with ';'.
                // We no longer need to manually accumulate or check for semicolon.
                let sql = buffer;

                // Clear the state's input buffer since we're executing
                state.clear_input();

                // Lock completion state to access client
                let cs = completion_state.lock().unwrap();
                let client = cs.client();

                // Execute the SQL with default limit for SELECT queries (Sprint 6: uses state colors)
                match execute_sql_with_state(client, state, &sql, writer, default_limit) {
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

            Ok(Signal::CtrlC) => {
                // Ctrl-C during input - reedline clears the buffer automatically
                // with validator, but we still show feedback to user
                writeln!(writer, "^C")?;
                state.clear_input();
            }

            Ok(Signal::CtrlD) => {
                // Ctrl-D exits when input is empty (validator returns Complete for empty input)
                break;
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
