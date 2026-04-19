//! Watch mode for monitoring commands
//!
//! Provides a shared auto-refresh loop used by `/sessions`, `/locks`, and
//! `/resources` when they are invoked with `--watch`. The display clears and
//! re-renders at a configurable interval. The user can press `q` or `Esc` to
//! stop and receive an exit snapshot, or `Ctrl-C` to stop silently.
//!
//! # Hardening (Sprint 65)
//!
//! - [`RawModeGuard`] is an RAII guard that enables raw mode + alternate
//!   screen + hidden cursor on construction and unconditionally restores
//!   them in its `Drop` impl. Terminal state survives panics.
//! - Per-tick errors from the render closure are caught and displayed as a
//!   red/bold error header within the frame; the last successful body is
//!   retained below so the user does not lose context.
//! - On `q` / `Esc`, after leaving the alternate screen, the last rendered
//!   frame is printed to stdout as a plain-text snapshot (mirroring the
//!   Sprint 63 pager exit snapshot pattern). `Ctrl-C` does NOT print the
//!   snapshot — matching pager convention.
//! - Interval is clamped to `[1, 3600]` seconds per
//!   REQ-REPL-SESSIONS-WATCH-002; default is 6 seconds.

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Minimum allowed watch interval, in seconds (REQ-REPL-SESSIONS-WATCH-002.2).
pub const MIN_INTERVAL_SECS: u64 = 1;

/// Maximum allowed watch interval, in seconds (REQ-REPL-SESSIONS-WATCH-002.3).
pub const MAX_INTERVAL_SECS: u64 = 3600;

/// Default watch interval when `--watch` is given without `--interval`
/// (REQ-REPL-SESSIONS-WATCH-002.1).
pub const DEFAULT_INTERVAL_SECS: u64 = 6;

/// How the watch loop terminated.
///
/// Used to decide whether to print an exit snapshot: `q` / `Esc` prints one,
/// `Ctrl-C` does not.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExitReason {
    /// User pressed `q`, `Q`, or `Esc` — print the exit snapshot.
    Quit,
    /// User pressed `Ctrl-C` — exit silently, no snapshot.
    Interrupt,
}

/// RAII guard for raw-mode + alternate-screen + hidden-cursor terminal state.
///
/// On construction, enables raw mode, enters the alternate screen, and hides
/// the cursor. On `Drop`, reverses all three unconditionally. The `Drop` impl
/// ignores errors because `Drop` cannot propagate them — this is best-effort
/// cleanup, and the only recourse on failure is to leave the terminal in a
/// degraded state (which is strictly better than no cleanup at all).
///
/// Using RAII here is critical: if any code in the watch loop panics, the
/// stack unwinds *through* the guard's `Drop`, so terminal state is always
/// restored before the panic escapes `run_watch`.
struct RawModeGuard;

impl RawModeGuard {
    /// Enter raw mode, alternate screen, and hide the cursor.
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        // If EnterAlternateScreen fails, raw mode is still enabled; disable it
        // before returning so we don't leak state on the error path.
        if let Err(e) = execute!(io::stdout(), EnterAlternateScreen, Hide) {
            let _ = disable_raw_mode();
            return Err(e);
        }
        Ok(Self)
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        // Best-effort cleanup. Drop cannot propagate errors; leaving the
        // terminal in a slightly-off state is better than a panic-on-drop.
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

/// Run a function in watch mode, refreshing at the given interval.
///
/// The `render` closure is called each refresh cycle. It receives a
/// `&mut Vec<u8>` writer and should write the command's plain-text output to
/// it (no ANSI escapes — watch mode itself adds chrome around the content).
///
/// # Exit behavior
///
/// - `q`, `Q`, or `Esc`: leaves the alternate screen, then writes a plain-text
///   snapshot of the last successful frame to stdout so the user has a
///   persistent copy (REQ-REPL-SESSIONS-WATCH-007).
/// - `Ctrl-C`: leaves the alternate screen without printing a snapshot
///   (REQ-REPL-SESSIONS-WATCH-005.3).
///
/// # Terminal safety
///
/// Raw mode and the alternate screen are restored by [`RawModeGuard::drop`]
/// on every exit path, including panics (REQ-REPL-SESSIONS-WATCH-006.5).
///
/// # Error handling
///
/// Errors returned by the `render` closure are caught and displayed as an
/// error header within the current frame; the loop continues
/// (REQ-REPL-SESSIONS-WATCH-008). I/O errors from writing to stdout propagate
/// as normal — those indicate a broken terminal that watch mode cannot
/// recover from.
pub fn run_watch<F>(interval_secs: u64, render: F) -> crate::error::Result<()>
where
    F: Fn(&mut Vec<u8>) -> crate::error::Result<()>,
{
    // Enter raw mode + alternate screen. Guard drops on every return path.
    let _guard = RawModeGuard::enter()?;

    let (reason, last_frame) = watch_loop(interval_secs, &render)?;

    // Explicitly drop the guard BEFORE writing the snapshot, so the snapshot
    // lands on the normal screen (in scrollback), not the alternate screen.
    drop(_guard);

    if reason == ExitReason::Quit {
        // Plain-text snapshot of the last frame on the primary screen.
        let mut stdout = io::stdout();
        stdout.write_all(&last_frame)?;
        stdout.flush()?;
    }

    Ok(())
}

/// Inner loop that drives the watch cycle.
///
/// Returns the reason the loop exited plus the final plain-text frame body
/// (header + render output + footer, no ANSI) so `run_watch` can print it
/// as an exit snapshot once the guard is dropped.
fn watch_loop<F>(
    interval_secs: u64,
    render: &F,
) -> crate::error::Result<(ExitReason, Vec<u8>)>
where
    F: Fn(&mut Vec<u8>) -> crate::error::Result<()>,
{
    let interval = Duration::from_secs(interval_secs);

    // Last successful render output. Retained so a transient query error
    // still shows the previous body below the error header.
    let mut last_body: Vec<u8> = Vec::new();
    // Plain-text snapshot of the most recently displayed frame.
    let mut last_snapshot: Vec<u8> = Vec::new();

    loop {
        // Try to render this tick. On error, we keep the loop alive and show
        // the error in the frame header.
        let mut fresh_body: Vec<u8> = Vec::new();
        let render_result = render(&mut fresh_body).map(|()| fresh_body);
        let outcome = handle_tick_result(render_result, last_body);
        last_body = outcome.body;
        let render_error = outcome.error_message;

        let timestamp = format_timestamp();

        // -- Draw to the alternate screen (with ANSI styling) --
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

        if let Some(err_msg) = &render_error {
            // Red + bold error header. Raw mode disables ONLCR, so use \r\n.
            write!(
                stdout,
                "\x1b[1;31mError at {}: {} - retrying in {}s\x1b[0m\r\n",
                timestamp, err_msg, interval_secs
            )?;
        } else {
            // Normal header (ANSI-free content; just formatted with \r\n).
            write!(
                stdout,
                "Updated {} - refreshing every {}s\r\n",
                timestamp, interval_secs
            )?;
        }
        write!(stdout, "\r\n")?;
        // Body: plain text from the render closure. It may contain \n, but
        // because we're in raw mode the terminal won't CR automatically.
        // Convert \n to \r\n for on-screen display only.
        for line in split_lines_preserve(&last_body) {
            stdout.write_all(line)?;
            write!(stdout, "\r\n")?;
        }
        // Footer / instruction line.
        write!(stdout, "\r\n")?;
        write!(
            stdout,
            "Press q, Esc, or Ctrl-C to stop (interval: {}s)\r\n",
            interval_secs
        )?;
        stdout.flush()?;

        // -- Build plain-text snapshot (no ANSI, \n endings) --
        last_snapshot.clear();
        if let Some(err_msg) = &render_error {
            writeln!(
                last_snapshot,
                "Error at {}: {} - retrying in {}s",
                timestamp, err_msg, interval_secs
            )?;
        } else {
            writeln!(
                last_snapshot,
                "Updated {} - refreshing every {}s",
                timestamp, interval_secs
            )?;
        }
        writeln!(last_snapshot)?;
        last_snapshot.extend_from_slice(&last_body);
        // Ensure snapshot ends with a trailing newline for a clean prompt.
        if !last_snapshot.ends_with(b"\n") {
            writeln!(last_snapshot)?;
        }

        // -- Poll for exit keys until the interval elapses --
        let start = Instant::now();
        while start.elapsed() < interval {
            let remaining = interval.saturating_sub(start.elapsed());
            let poll_timeout = remaining.min(Duration::from_millis(100));
            if event::poll(poll_timeout)? {
                if let Event::Key(key) = event::read()? {
                    if let Some(reason) = classify_key(&key) {
                        return Ok((reason, last_snapshot));
                    }
                }
            }
        }
    }
}

/// Outcome of a single watch-loop tick after the render closure runs.
///
/// Separates "what body to retain for future frames" from "what error header
/// to display this frame" so the tick-handling logic is pure and testable.
#[derive(Debug, Clone, PartialEq, Eq)]
struct TickOutcome {
    /// The body to retain as `last_body` for the next frame. On success this
    /// is the freshly rendered body; on error the previous `last_body` is
    /// returned unchanged so the user does not lose context.
    body: Vec<u8>,
    /// The formatted error header text for this frame, or `None` on success.
    /// Contains only the error message itself — no ANSI, no trailing newline,
    /// no `Error at ... - retrying in ...` framing (that's the caller's job).
    error_message: Option<String>,
}

/// Pure tick-result handler. Decides:
///   - what becomes the retained body going forward, and
///   - whether this frame needs an error header.
///
/// AC-3: on `Err`, the retained body is the unchanged `last_body`.
/// AC-4: on `Ok(new)`, the retained body is `new` (the freshly rendered body).
///
/// This function is pure (no I/O, no global state) and trivially unit-testable.
fn handle_tick_result(
    render_result: crate::error::Result<Vec<u8>>,
    last_body: Vec<u8>,
) -> TickOutcome {
    match render_result {
        Ok(new_body) => TickOutcome {
            body: new_body,
            error_message: None,
        },
        Err(e) => TickOutcome {
            body: last_body,
            error_message: Some(e.to_string()),
        },
    }
}

/// Split a byte buffer on `\n` but keep empty trailing lines out.
///
/// Returns a `Vec` of slices (no allocation per line for typical usage),
/// used only to convert `\n` to `\r\n` when writing to the alternate screen.
fn split_lines_preserve(buf: &[u8]) -> Vec<&[u8]> {
    let mut lines: Vec<&[u8]> = buf.split(|b| *b == b'\n').collect();
    // `split` produces a trailing empty slice after a terminating '\n';
    // drop it so we don't emit a spurious blank line.
    if matches!(lines.last(), Some(last) if last.is_empty()) {
        lines.pop();
    }
    lines
}

/// Classify a key event into an exit reason, or `None` if it should not
/// cause an exit.
fn classify_key(key: &crossterm::event::KeyEvent) -> Option<ExitReason> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(ExitReason::Quit),
        KeyCode::Char('c') | KeyCode::Char('C')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            Some(ExitReason::Interrupt)
        }
        _ => None,
    }
}

/// Determine whether a key event should exit watch mode (any exit type).
///
/// Retained for backwards-compatible unit tests.
#[cfg(test)]
fn should_quit(key: &crossterm::event::KeyEvent) -> bool {
    classify_key(key).is_some()
}

/// Produce a `HH:MM:SS` timestamp for the status footer.
fn format_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// Parse a `--watch` / `--interval` pair from a slice of REPL arguments.
///
/// Supports the following patterns:
///
/// - `--watch`              -> returns `Some(default)` where `default` is the
///   caller-supplied fallback (typically 6).
/// - `--watch 5`            -> returns `Some(5)` (shorthand).
/// - `--interval 5`         -> returns `Some(5)`.
/// - `--watch --interval 5` -> returns `Some(5)`.
///
/// Values are clamped to `[MIN_INTERVAL_SECS, MAX_INTERVAL_SECS]`
/// (REQ-REPL-SESSIONS-WATCH-002). Non-numeric arguments after `--watch` fall
/// back to the default.
///
/// Returns `None` if `--watch` is not present in the args.
pub fn parse_watch_args(args: &[&str], default_interval: u64) -> Option<u64> {
    let has_watch = args.contains(&"--watch");
    if !has_watch {
        return None;
    }

    // Look for an explicit --interval value first.
    for (i, arg) in args.iter().enumerate() {
        if *arg == "--interval" {
            if let Some(next) = args.get(i + 1) {
                if let Ok(n) = next.parse::<u64>() {
                    return Some(n.clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS));
                }
            }
        }
    }

    // Fall back to positional value after --watch (e.g. `--watch 5`).
    for (i, arg) in args.iter().enumerate() {
        if *arg == "--watch" {
            if let Some(next) = args.get(i + 1) {
                if let Ok(n) = next.parse::<u64>() {
                    return Some(n.clamp(MIN_INTERVAL_SECS, MAX_INTERVAL_SECS));
                }
            }
        }
    }

    Some(default_interval)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- parse_watch_args tests --

    #[test]
    fn no_watch_flag_returns_none() {
        let args: Vec<&str> = vec!["--physical"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), None);
    }

    #[test]
    fn watch_alone_uses_default() {
        let args = vec!["--watch"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(6));
    }

    #[test]
    fn watch_with_positional_interval() {
        let args = vec!["--watch", "10"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(10));
    }

    #[test]
    fn watch_with_explicit_interval() {
        let args = vec!["--watch", "--interval", "15"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(15));
    }

    #[test]
    fn interval_without_watch_returns_none() {
        let args = vec!["--interval", "5"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), None);
    }

    #[test]
    fn interval_takes_precedence_over_positional() {
        let args = vec!["--watch", "10", "--interval", "20"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(20));
    }

    #[test]
    fn interval_one_is_allowed() {
        // REQ-REPL-SESSIONS-WATCH-002.2: minimum is 1.
        let args = vec!["--watch", "1"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(1));
    }

    #[test]
    fn interval_zero_is_clamped_to_minimum() {
        let args = vec!["--watch", "0"];
        assert_eq!(
            parse_watch_args(&args, DEFAULT_INTERVAL_SECS),
            Some(MIN_INTERVAL_SECS)
        );
    }

    #[test]
    fn interval_at_max_boundary_is_allowed() {
        let args = vec!["--watch", "3600"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(3600));
    }

    #[test]
    fn interval_above_max_is_clamped() {
        // REQ-REPL-SESSIONS-WATCH-002.3: maximum is 3600.
        let args = vec!["--watch", "999999"];
        assert_eq!(
            parse_watch_args(&args, DEFAULT_INTERVAL_SECS),
            Some(MAX_INTERVAL_SECS)
        );
    }

    #[test]
    fn non_numeric_positional_ignored() {
        let args = vec!["--watch", "--physical"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(6));
    }

    #[test]
    fn negative_interval_is_ignored() {
        // `-5` is not a valid u64; fall back to default.
        let args = vec!["--watch", "--interval", "-5"];
        assert_eq!(parse_watch_args(&args, DEFAULT_INTERVAL_SECS), Some(6));
    }

    #[test]
    fn default_interval_constant_matches_spec() {
        // REQ-REPL-SESSIONS-WATCH-002.1.
        assert_eq!(DEFAULT_INTERVAL_SECS, 6);
    }

    // -- classify_key / should_quit tests --

    #[test]
    fn q_key_quits() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert_eq!(classify_key(&key), Some(ExitReason::Quit));
        assert!(should_quit(&key));
    }

    #[test]
    fn uppercase_q_quits() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE);
        assert_eq!(classify_key(&key), Some(ExitReason::Quit));
    }

    #[test]
    fn esc_quits() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert_eq!(classify_key(&key), Some(ExitReason::Quit));
    }

    #[test]
    fn ctrl_c_interrupts() {
        // REQ-REPL-SESSIONS-WATCH-005.3: Ctrl-C exits *without* snapshot.
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(classify_key(&key), Some(ExitReason::Interrupt));
        assert!(should_quit(&key));
    }

    #[test]
    fn ctrl_uppercase_c_interrupts() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('C'), KeyModifiers::CONTROL);
        assert_eq!(classify_key(&key), Some(ExitReason::Interrupt));
    }

    #[test]
    fn plain_c_does_not_quit() {
        // 'c' without Ctrl is not an exit key.
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
        assert_eq!(classify_key(&key), None);
    }

    #[test]
    fn regular_key_does_not_quit() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert_eq!(classify_key(&key), None);
    }

    #[test]
    fn ctrl_d_does_not_quit() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('d'), KeyModifiers::CONTROL);
        assert_eq!(classify_key(&key), None);
    }

    // -- format_timestamp --

    #[test]
    fn timestamp_has_expected_format() {
        let ts = format_timestamp();
        // Should be HH:MM:SS -- 8 characters.
        assert_eq!(ts.len(), 8, "timestamp '{}' is not HH:MM:SS", ts);
        assert_eq!(&ts[2..3], ":");
        assert_eq!(&ts[5..6], ":");
    }

    // -- split_lines_preserve --

    #[test]
    fn split_lines_basic() {
        let input = b"hello\nworld\n";
        let lines = split_lines_preserve(input);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], b"hello");
        assert_eq!(lines[1], b"world");
    }

    #[test]
    fn split_lines_no_trailing_newline() {
        let input = b"hello\nworld";
        let lines = split_lines_preserve(input);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], b"hello");
        assert_eq!(lines[1], b"world");
    }

    #[test]
    fn split_lines_empty() {
        let input = b"";
        let lines = split_lines_preserve(input);
        assert_eq!(lines.len(), 0);
    }

    #[test]
    fn split_lines_internal_blank_line_preserved() {
        let input = b"a\n\nb\n";
        let lines = split_lines_preserve(input);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], b"a");
        assert_eq!(lines[1], b"");
        assert_eq!(lines[2], b"b");
    }

    // -- render closure integration --

    #[test]
    fn render_closure_writes_to_buffer() {
        let render = |buf: &mut Vec<u8>| -> crate::error::Result<()> {
            writeln!(buf, "hello")?;
            Ok(())
        };

        let mut buf = Vec::new();
        render(&mut buf).unwrap();
        assert_eq!(String::from_utf8_lossy(&buf), "hello\n");
    }

    // -- exit-reason semantics documented by enum value --

    #[test]
    fn exit_reason_variants_distinct() {
        assert_ne!(ExitReason::Quit, ExitReason::Interrupt);
    }

    // ---------------------------------------------------------------------------
    // TC096: Sprint 65 spec-named interval boundary tests
    // Named to match test case document tests/cases/TC096.md for traceability.
    // These tests assert the spec requirements (AC-1, AC-2, AC-3, AC-9) using
    // the same names as the test case document.
    // ---------------------------------------------------------------------------

    /// TC096-A: AC-9 — no --watch flag means no watch mode activated
    #[test]
    fn watch_parse_no_flag_returns_none() {
        assert_eq!(parse_watch_args(&["--physical"], 6), None);
    }

    /// TC096-B: AC-1 — --watch alone uses the caller-supplied default of 6 s
    #[test]
    fn watch_parse_default_interval_is_6s() {
        assert_eq!(parse_watch_args(&["--watch"], 6), Some(6));
    }

    /// TC096-C: AC-2 — --interval 10 returns exactly 10 s
    #[test]
    fn watch_parse_explicit_interval_10() {
        assert_eq!(
            parse_watch_args(&["--watch", "--interval", "10"], 6),
            Some(10)
        );
    }

    /// TC096-D: AC-3 — minimum is 1 s (spec says 1, not 2)
    #[test]
    fn watch_parse_minimum_boundary_1s_accepted() {
        assert_eq!(
            parse_watch_args(&["--watch", "--interval", "1"], 6),
            Some(1)
        );
    }

    /// TC096-E: AC-3 — interval of 0 must be clamped to at least 1 s
    #[test]
    fn watch_parse_zero_interval_clamped_to_minimum() {
        let result = parse_watch_args(&["--watch", "--interval", "0"], 6);
        match result {
            Some(v) => assert!(v >= 1, "interval 0 must produce at least 1 s, got {}", v),
            None => panic!("--watch is present; result must be Some(_)"),
        }
    }

    /// TC096-F: AC-3 — maximum is 3600 s (spec says 3600, not 300)
    #[test]
    fn watch_parse_maximum_boundary_3600s_accepted() {
        assert_eq!(
            parse_watch_args(&["--watch", "--interval", "3600"], 6),
            Some(3600)
        );
    }

    /// TC096-G: AC-3 — values above 3600 must be clamped to 3600
    #[test]
    fn watch_parse_above_max_clamped_to_3600() {
        let result = parse_watch_args(&["--watch", "--interval", "3601"], 6);
        match result {
            Some(v) => assert!(
                v <= 3600,
                "--interval 3601 must be clamped to at most 3600, got {}",
                v
            ),
            None => panic!("--watch is present; result must be Some(_)"),
        }
    }

    /// TC096-H: AC-9 — --interval alone (no --watch) must not activate watch mode
    #[test]
    fn watch_parse_interval_only_no_watch_returns_none() {
        assert_eq!(parse_watch_args(&["--interval", "5"], 6), None);
    }

    // ---------------------------------------------------------------------------
    // Sprint 67: handle_tick_result (Sprint 65 P2 follow-up)
    //
    // Feature 2 ACs:
    //   AC-1: pure function, returns both display and retention data.
    //   AC-2: byte-identical behaviour (covered by using the same function
    //         from the live `watch_loop`, no behaviour divergence possible).
    //   AC-3: test_handle_tick_result_error_retains_last_body
    //   AC-4: test_handle_tick_result_success_replaces_body
    // ---------------------------------------------------------------------------

    /// AC-3: On Err(...) the retained body is exactly `last_body` and the
    /// error message is populated with the error's string representation.
    #[test]
    fn test_handle_tick_result_error_retains_last_body() {
        let last_body = b"previous frame content\n".to_vec();
        let render_result: crate::error::Result<Vec<u8>> =
            Err(crate::error::TqError::QueryExecution(
                "simulated tick failure".into(),
            ));

        let outcome = handle_tick_result(render_result, last_body.clone());

        assert_eq!(
            outcome.body, last_body,
            "On error, retained body must equal last_body unchanged"
        );
        let err_msg = outcome
            .error_message
            .expect("Error path must populate error_message");
        assert!(
            err_msg.contains("simulated tick failure"),
            "Error message should contain the error text, got: {:?}",
            err_msg
        );
    }

    /// AC-4: On Ok(new) the retained body becomes `new` and there is no error.
    #[test]
    fn test_handle_tick_result_success_replaces_body() {
        let last_body = b"previous frame content\n".to_vec();
        let new_body = b"fresh frame content\n".to_vec();
        let render_result: crate::error::Result<Vec<u8>> = Ok(new_body.clone());

        let outcome = handle_tick_result(render_result, last_body);

        assert_eq!(
            outcome.body, new_body,
            "On success, retained body must equal the freshly rendered body"
        );
        assert!(
            outcome.error_message.is_none(),
            "Success path must not populate error_message"
        );
    }

    /// AC-3 corollary: the last_body is preserved even when it is empty
    /// (first tick fails before any successful render).
    #[test]
    fn test_handle_tick_result_error_with_empty_last_body() {
        let last_body: Vec<u8> = Vec::new();
        let render_result: crate::error::Result<Vec<u8>> =
            Err(crate::error::TqError::QueryExecution(
                "first tick failed".into(),
            ));

        let outcome = handle_tick_result(render_result, last_body);

        assert!(
            outcome.body.is_empty(),
            "Empty last_body stays empty on error"
        );
        assert!(
            outcome.error_message.is_some(),
            "Error message must be populated"
        );
    }

    /// AC-4 corollary: ownership transfer — `handle_tick_result` takes ownership
    /// of its inputs and returns a TickOutcome owning the body. Asserting we
    /// can mutate outcome.body without touching anything else.
    #[test]
    fn test_handle_tick_result_ownership() {
        let last_body = b"old".to_vec();
        let new_body = b"new".to_vec();
        let render_result: crate::error::Result<Vec<u8>> = Ok(new_body);
        let mut outcome = handle_tick_result(render_result, last_body);
        outcome.body.extend_from_slice(b"-more");
        assert_eq!(outcome.body, b"new-more");
    }
}
