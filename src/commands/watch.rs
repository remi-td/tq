//! Watch mode for monitoring commands
//!
//! Provides a shared auto-refresh loop that any monitoring command can use.
//! The display clears and re-renders at a configurable interval. The user
//! can press `q`, `Esc`, or `Ctrl-C` to stop.

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Run a function in watch mode, refreshing at the given interval.
///
/// The `render` closure is called each refresh cycle. It receives a `&mut Vec<u8>`
/// writer and should write the command's output to it. The watch function handles
/// clearing the screen, writing the rendered content, and appending a status footer.
///
/// Returns `Ok(())` when the user presses `q`, `Esc`, or `Ctrl-C`.
///
/// # Terminal safety
///
/// Raw mode is always disabled before returning, even on error.
pub fn run_watch<F>(interval_secs: u64, render: F) -> crate::error::Result<()>
where
    F: Fn(&mut Vec<u8>) -> crate::error::Result<()>,
{
    terminal::enable_raw_mode()?;

    let result = watch_loop(interval_secs, &render);

    // Always restore terminal state, even if the loop returned an error.
    let _ = terminal::disable_raw_mode();

    result
}

/// Inner loop that drives the watch cycle.
///
/// Separated from `run_watch` so that the raw-mode guard lives in the outer
/// function and `disable_raw_mode` is guaranteed to run.
fn watch_loop<F>(interval_secs: u64, render: &F) -> crate::error::Result<()>
where
    F: Fn(&mut Vec<u8>) -> crate::error::Result<()>,
{
    let interval = Duration::from_secs(interval_secs);

    loop {
        // Clear screen and move cursor to top-left.
        let mut stdout = io::stdout();
        execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

        // Render the command output into a buffer.
        let mut buf = Vec::new();
        render(&mut buf)?;
        stdout.write_all(&buf)?;

        // Append the status footer.
        let timestamp = format_timestamp();
        writeln!(stdout)?;
        writeln!(
            stdout,
            "Last updated: {} | Refreshing every {}s | Press q or Ctrl-C to stop",
            timestamp, interval_secs
        )?;
        stdout.flush()?;

        // Wait for the interval, polling for key presses every 100 ms.
        let start = Instant::now();
        while start.elapsed() < interval {
            let remaining = interval.saturating_sub(start.elapsed());
            let poll_timeout = remaining.min(Duration::from_millis(100));
            if event::poll(poll_timeout)? {
                if let Event::Key(key) = event::read()? {
                    if should_quit(&key) {
                        return Ok(());
                    }
                }
            }
        }
    }
}

/// Determine whether a key event should exit watch mode.
fn should_quit(key: &crossterm::event::KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
        || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

/// Produce a `HH:MM:SS` timestamp for the status footer.
fn format_timestamp() -> String {
    chrono::Local::now().format("%H:%M:%S").to_string()
}

/// Parse a `--watch` / `--interval` pair from a slice of REPL arguments.
///
/// Supports the following patterns:
///
/// - `--watch`             -> returns `Some(default)` where `default` is the
///   caller-supplied fallback (typically 6).
/// - `--watch 5`           -> returns `Some(5)` (shorthand).
/// - `--interval 5`        -> returns `Some(5)`.
/// - `--watch --interval 5`-> returns `Some(5)`.
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
                    let clamped = n.clamp(2, 300);
                    return Some(clamped);
                }
            }
        }
    }

    // Fall back to positional value after --watch (e.g. `--watch 5`).
    for (i, arg) in args.iter().enumerate() {
        if *arg == "--watch" {
            if let Some(next) = args.get(i + 1) {
                if let Ok(n) = next.parse::<u64>() {
                    let clamped = n.clamp(2, 300);
                    return Some(clamped);
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
        assert_eq!(parse_watch_args(&args, 6), None);
    }

    #[test]
    fn watch_alone_uses_default() {
        let args = vec!["--watch"];
        assert_eq!(parse_watch_args(&args, 6), Some(6));
    }

    #[test]
    fn watch_with_positional_interval() {
        let args = vec!["--watch", "10"];
        assert_eq!(parse_watch_args(&args, 6), Some(10));
    }

    #[test]
    fn watch_with_explicit_interval() {
        let args = vec!["--watch", "--interval", "15"];
        assert_eq!(parse_watch_args(&args, 6), Some(15));
    }

    #[test]
    fn interval_without_watch_returns_none() {
        let args = vec!["--interval", "5"];
        assert_eq!(parse_watch_args(&args, 6), None);
    }

    #[test]
    fn interval_takes_precedence_over_positional() {
        let args = vec!["--watch", "10", "--interval", "20"];
        assert_eq!(parse_watch_args(&args, 6), Some(20));
    }

    #[test]
    fn interval_clamped_below_minimum() {
        let args = vec!["--watch", "1"];
        assert_eq!(parse_watch_args(&args, 6), Some(2));
    }

    #[test]
    fn interval_clamped_above_maximum() {
        let args = vec!["--watch", "999"];
        assert_eq!(parse_watch_args(&args, 6), Some(300));
    }

    #[test]
    fn non_numeric_positional_ignored() {
        let args = vec!["--watch", "--physical"];
        assert_eq!(parse_watch_args(&args, 6), Some(6));
    }

    // -- should_quit tests --

    #[test]
    fn q_key_quits() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE);
        assert!(should_quit(&key));
    }

    #[test]
    fn uppercase_q_quits() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('Q'), KeyModifiers::NONE);
        assert!(should_quit(&key));
    }

    #[test]
    fn esc_quits() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
        assert!(should_quit(&key));
    }

    #[test]
    fn ctrl_c_quits() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert!(should_quit(&key));
    }

    #[test]
    fn regular_key_does_not_quit() {
        let key = crossterm::event::KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        assert!(!should_quit(&key));
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
}
