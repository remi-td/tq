//! Tiered PTY test harness for interactive REPL tests (Sprint 66)
//!
//! This module provides an observable, tiered-timeout harness for interactive
//! REPL tests. It addresses two recurrent problems with the legacy
//! `spawn_tq_repl()` helper:
//!
//! 1. A single blanket timeout made it impossible to distinguish a slow
//!    connect from a slow query when a test failed with `ExpectTimeout`.
//! 2. The PTY buffer held on timeout was discarded, so failures produced no
//!    actionable diagnostics — just "ExpectTimeout" with no context.
//!
//! # Design
//!
//! - [`Stage`] names the three phases of a REPL interaction: connect/auth,
//!   prompt-ready, query-result. Each has its own timeout budget.
//! - [`Timeouts`] carries the three budgets. [`Timeouts::from_env()`] reads
//!   `TQ_TEST_CONNECT_TIMEOUT`, `TQ_TEST_PROMPT_TIMEOUT`,
//!   `TQ_TEST_QUERY_TIMEOUT` (all u64 seconds) and falls back to defaults
//!   (45 / 15 / 60 s).
//! - [`TqPty`] wraps an [`expectrl::Session`] and a rolling capture of bytes
//!   read so far. `expect_stage()` applies the stage's timeout, and on
//!   `ExpectTimeout` drains any remaining pending bytes, writes the last
//!   4 KiB to `tests/results/sprint-66/<test_name>.pty.log`, and returns a
//!   stage-specific [`PtyError`] variant.
//! - [`PtyError`] has four variants: one per stage plus an IO catch-all.
//!
//! # Unit testability
//!
//! The dump logic lives in the free function [`write_dump_for_test()`] so
//! unit tests can exercise it without a PTY or a live database.

use std::fs;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use expectrl::Session;

// ---------------------------------------------------------------------------
// Cursor Position Report (CPR) injection
// ---------------------------------------------------------------------------
//
// Some REPL screens (e.g. the alternate-screen `\sessions --watch` viewer)
// emit a Device Status Report cursor query (`ESC [ 6 n`, "DSR-CPR") and block
// waiting for the terminal to reply with the current cursor position
// (`ESC [ row ; col R`). A bare PTY has no terminal driver to answer, so the
// child stalls and the test times out, often surfacing as a "cursor position"
// error loop. The harness plays the terminal's role: whenever a chunk read
// from the child contains the DSR cursor query, it writes back a canned CPR
// reply so the child can proceed.

/// Device Status Report — cursor-position query emitted by the child.
const DSR_CURSOR_QUERY: &[u8] = b"\x1b[6n";

/// Canned Cursor Position Report reply (`ESC [ 1 ; 1 R`, row 1 col 1).
const CPR_REPLY: &[u8] = b"\x1b[1;1R";

/// Returns true if `bytes` contains the DSR cursor-position query sequence.
pub(crate) fn detect_cpr_query(bytes: &[u8]) -> bool {
    bytes
        .windows(DSR_CURSOR_QUERY.len())
        .any(|w| w == DSR_CURSOR_QUERY)
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Stage of a REPL interaction. Each stage has its own timeout budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// Connecting + authenticating to the database (slow: TLS + auth + catalog warm-up).
    Connect,
    /// Waiting for the REPL prompt after connection.
    Prompt,
    /// Waiting for a query result (can be slow against a cold endpoint).
    Query,
}

impl Stage {
    /// Short, human-readable label used in error messages and file names.
    pub fn label(self) -> &'static str {
        match self {
            Stage::Connect => "connect",
            Stage::Prompt => "prompt",
            Stage::Query => "query",
        }
    }
}

/// Tiered timeout budgets (seconds).
#[derive(Debug, Clone, Copy)]
pub struct Timeouts {
    pub connect: Duration,
    pub prompt: Duration,
    pub query: Duration,
}

impl Timeouts {
    /// Default budgets: connect=45 s, prompt=15 s, query=60 s.
    pub const DEFAULT_CONNECT_SECS: u64 = 45;
    pub const DEFAULT_PROMPT_SECS: u64 = 15;
    pub const DEFAULT_QUERY_SECS: u64 = 60;

    /// Construct the built-in defaults.
    pub fn defaults() -> Self {
        Self {
            connect: Duration::from_secs(Self::DEFAULT_CONNECT_SECS),
            prompt: Duration::from_secs(Self::DEFAULT_PROMPT_SECS),
            query: Duration::from_secs(Self::DEFAULT_QUERY_SECS),
        }
    }

    /// Construct from environment, falling back to defaults for any unset
    /// or unparseable variable. Recognises:
    /// - `TQ_TEST_CONNECT_TIMEOUT` (u64 seconds)
    /// - `TQ_TEST_PROMPT_TIMEOUT` (u64 seconds)
    /// - `TQ_TEST_QUERY_TIMEOUT` (u64 seconds)
    pub fn from_env() -> Self {
        let mut t = Self::defaults();
        if let Some(secs) = parse_env_secs("TQ_TEST_CONNECT_TIMEOUT") {
            t.connect = Duration::from_secs(secs);
        }
        if let Some(secs) = parse_env_secs("TQ_TEST_PROMPT_TIMEOUT") {
            t.prompt = Duration::from_secs(secs);
        }
        if let Some(secs) = parse_env_secs("TQ_TEST_QUERY_TIMEOUT") {
            t.query = Duration::from_secs(secs);
        }
        t
    }

    /// Budget for a given stage.
    pub fn for_stage(&self, stage: Stage) -> Duration {
        match stage {
            Stage::Connect => self.connect,
            Stage::Prompt => self.prompt,
            Stage::Query => self.query,
        }
    }
}

/// Errors surfaced by the tiered PTY harness.
///
/// Three of the four variants are stage-specific timeout errors; each carries
/// the path to the dumped PTY tail log so the caller can reference it in a
/// `panic!` / `expect()` message.
#[derive(Debug)]
pub enum PtyError {
    /// Connect/auth stage exceeded its budget. `dump_path` holds the tail log.
    ConnectTimeout { dump_path: PathBuf },
    /// Prompt-ready stage exceeded its budget. `dump_path` holds the tail log.
    PromptTimeout { dump_path: PathBuf },
    /// Query-result stage exceeded its budget. `dump_path` holds the tail log.
    QueryTimeout { dump_path: PathBuf },
    /// Any non-timeout error (IO, process exit, unexpected expectrl error).
    Io(io::Error),
}

impl PtyError {
    /// Construct the timeout variant appropriate for the given stage.
    pub fn timeout_for(stage: Stage, dump_path: PathBuf) -> Self {
        match stage {
            Stage::Connect => PtyError::ConnectTimeout { dump_path },
            Stage::Prompt => PtyError::PromptTimeout { dump_path },
            Stage::Query => PtyError::QueryTimeout { dump_path },
        }
    }
}

impl std::fmt::Display for PtyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtyError::ConnectTimeout { dump_path } => write!(
                f,
                "connect stage timed out (see PTY tail: {})",
                dump_path.display()
            ),
            PtyError::PromptTimeout { dump_path } => write!(
                f,
                "prompt stage timed out (see PTY tail: {})",
                dump_path.display()
            ),
            PtyError::QueryTimeout { dump_path } => write!(
                f,
                "query stage timed out (see PTY tail: {})",
                dump_path.display()
            ),
            PtyError::Io(e) => write!(f, "PTY IO error: {}", e),
        }
    }
}

impl std::error::Error for PtyError {}

impl From<io::Error> for PtyError {
    fn from(e: io::Error) -> Self {
        PtyError::Io(e)
    }
}

/// A tiered PTY wrapper around an `expectrl::Session`.
///
/// Callers use [`TqPty::expect_stage`] instead of raw `session.expect(...)`
/// so each wait is bounded by the correct tier and any timeout produces a
/// diagnostic dump on disk.
pub struct TqPty {
    session: Session,
    test_name: String,
    timeouts: Timeouts,
    /// Rolling capture of bytes observed on stdout. Grows as `expect_stage`
    /// succeeds so the dump on a later timeout has context.
    captured: Vec<u8>,
}

impl TqPty {
    /// Wrap an existing session. The session's own expect timeout is
    /// reconfigured per-call by [`expect_stage`].
    pub fn new(session: Session, test_name: impl Into<String>, timeouts: Timeouts) -> Self {
        Self {
            session,
            test_name: test_name.into(),
            timeouts,
            captured: Vec::new(),
        }
    }

    /// Borrow the underlying session for operations the wrapper does not
    /// expose (e.g. `send_line`, `send`).
    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    /// Expect a needle within the given stage's timeout budget. On success,
    /// the consumed bytes are appended to the rolling capture. On
    /// `ExpectTimeout`, any still-pending bytes are drained, the last 4 KiB
    /// of the capture is written to
    /// `tests/results/sprint-66/<test_name>.pty.log`, and a stage-specific
    /// [`PtyError`] is returned.
    pub fn expect_stage<N>(&mut self, stage: Stage, needle: N) -> Result<Vec<u8>, PtyError>
    where
        N: expectrl::Needle,
    {
        // Manual, chunk-driven match loop. We poll `try_read` ourselves rather
        // than delegating to `session.expect()` so that EVERY chunk passes
        // through the CPR-answering path (`read_chunk_answering_cpr`). This is
        // essential: the REPL emits a Device Status Report cursor query
        // (`ESC [ 6 n`) and blocks waiting for a reply. If we relied on
        // `session.expect()`, those query bytes would be buffered internally
        // and we'd never get a chance to answer until after the whole stage
        // timed out — by which point crossterm has already entered its
        // "cursor position could not be read" retry loop.
        let deadline = Instant::now() + self.timeouts.for_stage(stage);
        // Index into `captured` from which the needle has not yet been checked.
        // The needle scans the rolling tail so a match spanning chunk
        // boundaries is still found.
        let scan_start = self.captured.len();
        let mut scratch = [0u8; 4096];
        loop {
            match self.session.try_read(&mut scratch) {
                Ok(0) => {
                    if Instant::now() >= deadline {
                        return self.timeout_with_dump(stage);
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Ok(n) => {
                    self.absorb_chunk(&scratch[..n]);
                    // Check the accumulated tail for a match.
                    if let Ok(matches) = needle.check(&self.captured[scan_start..], false) {
                        if let Some(m) = matches.into_iter().max_by_key(|m| m.end()) {
                            let abs_end = scan_start + m.end();
                            return Ok(self.captured[scan_start..abs_end].to_vec());
                        }
                    }
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    if Instant::now() >= deadline {
                        return self.timeout_with_dump(stage);
                    }
                    std::thread::sleep(Duration::from_millis(5));
                }
                Err(e) => return Err(PtyError::Io(e)),
            }
        }
    }

    /// Append a chunk to the rolling capture, answering any CPR cursor query
    /// it contains so the child does not stall waiting for a terminal reply.
    fn absorb_chunk(&mut self, chunk: &[u8]) {
        if detect_cpr_query(chunk) {
            let _ = self.session.write_all(CPR_REPLY);
            let _ = self.session.flush();
        }
        self.captured.extend_from_slice(chunk);
    }

    /// Build a stage-specific timeout error, draining any pending bytes and
    /// dumping the rolling tail for diagnostics first.
    fn timeout_with_dump<T>(&mut self, stage: Stage) -> Result<T, PtyError> {
        self.drain_pending();
        let dump_path = dump_path_for(&self.test_name);
        let _ = write_dump_for_test(&self.test_name, &self.captured);
        Err(PtyError::timeout_for(stage, dump_path))
    }

    /// Drain any bytes currently available on the session and append them
    /// to the rolling capture. Non-blocking.
    ///
    /// `expectrl::SyncSession::try_read` returns `Ok(0)` when no bytes are
    /// available *at this instant*, which is NOT necessarily EOF — output
    /// may still be in-flight from the child process. To avoid truncating
    /// the dump on transient empty reads, we retry with a short bounded
    /// window (~50 ms total) before giving up.
    fn drain_pending(&mut self) {
        let mut scratch = [0u8; 4096];
        let deadline = Instant::now() + Duration::from_millis(50);
        let mut empty_reads: u32 = 0;
        loop {
            match self.session.try_read(&mut scratch) {
                Ok(0) => {
                    if Instant::now() >= deadline {
                        break;
                    }
                    // Short sleep before retrying; bounded by the deadline.
                    std::thread::sleep(Duration::from_millis(5));
                    empty_reads += 1;
                    if empty_reads > 10 {
                        break;
                    }
                }
                Ok(n) => {
                    self.absorb_chunk(&scratch[..n]);
                    empty_reads = 0;
                }
                Err(_) => break,
            }
        }
    }

    /// Access the captured buffer (useful for diagnostic assertions).
    pub fn captured(&self) -> &[u8] {
        &self.captured
    }
}

// ---------------------------------------------------------------------------
// Free functions
// ---------------------------------------------------------------------------

/// Maximum number of bytes retained in the PTY tail log.
pub const DUMP_TAIL_BYTES: usize = 4096;

/// Directory in which PTY tail logs are written. Derived from the project
/// root via `CARGO_MANIFEST_DIR` so the path is stable regardless of the
/// current working directory at test time.
fn dump_dir() -> PathBuf {
    let root = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(root).join("tests/results/sprint-66")
}

/// Path of the tail log for a given test name.
pub fn dump_path_for(test_name: &str) -> PathBuf {
    dump_dir().join(format!("{}.pty.log", sanitize_test_name(test_name)))
}

/// Replace path-hostile characters in test names so the dump file is always
/// creatable.
fn sanitize_test_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            c if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' => c,
            _ => '_',
        })
        .collect()
}

/// Write the last [`DUMP_TAIL_BYTES`] bytes of `buffer` to the PTY tail log
/// for `test_name`. Creates the directory on demand.
///
/// Exposed as a free function so unit tests can exercise the dump without a
/// live PTY.
pub fn write_dump_for_test(test_name: &str, buffer: &[u8]) -> io::Result<PathBuf> {
    let path = dump_path_for(test_name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tail = if buffer.len() > DUMP_TAIL_BYTES {
        &buffer[buffer.len() - DUMP_TAIL_BYTES..]
    } else {
        buffer
    };
    fs::write(&path, tail)?;
    Ok(path)
}

fn parse_env_secs(key: &str) -> Option<u64> {
    std::env::var(key).ok().and_then(|v| v.trim().parse().ok())
}

// ---------------------------------------------------------------------------
// Unit tests (TC-66-U01..U03)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Guard for env-mutating tests. Rust runs `#[test]` functions on
    // multiple threads; env vars are process-global. Serialising env tests
    // with a mutex prevents cross-test pollution.
    fn env_lock() -> &'static std::sync::Mutex<()> {
        static LOCK: std::sync::OnceLock<std::sync::Mutex<()>> = std::sync::OnceLock::new();
        LOCK.get_or_init(|| std::sync::Mutex::new(()))
    }

    /// TC-66-U01 — `write_dump_for_test` writes the tail of the buffer,
    /// truncates to `DUMP_TAIL_BYTES`, and creates the directory if missing.
    #[test]
    fn write_dump_writes_tail_and_creates_file() {
        // Small buffer — written in full.
        let small = b"hello world";
        let path = write_dump_for_test("tc_66_u01_small", small).expect("small dump");
        assert!(path.exists(), "dump file should exist: {}", path.display());
        let written = fs::read(&path).expect("read back small dump");
        assert_eq!(written, small, "small buffer written verbatim");

        // Oversized buffer — tail only.
        let big: Vec<u8> = (0..8000u32).map(|i| (i % 256) as u8).collect();
        let path = write_dump_for_test("tc_66_u01_big", &big).expect("big dump");
        let written = fs::read(&path).expect("read back big dump");
        assert_eq!(
            written.len(),
            DUMP_TAIL_BYTES,
            "oversized dump truncated to last {DUMP_TAIL_BYTES} bytes"
        );
        assert_eq!(
            written.as_slice(),
            &big[big.len() - DUMP_TAIL_BYTES..],
            "dump is the TAIL of the buffer, not the head"
        );

        // Dump directory exists at the expected location.
        assert!(dump_dir().is_dir(), "dump dir exists under project root");
    }

    /// TC-66-U02 — each `PtyError` stage variant is constructible with the
    /// correct constructor and its `Display` impl names the stage plus the
    /// dump path. Exercises `PtyError::timeout_for` for exhaustive coverage.
    #[test]
    fn pty_error_variants_identify_stage_and_dump_path() {
        let path = PathBuf::from("/tmp/nowhere.pty.log");

        let conn = PtyError::timeout_for(Stage::Connect, path.clone());
        assert!(matches!(conn, PtyError::ConnectTimeout { .. }));
        let s = conn.to_string();
        assert!(s.contains("connect"), "connect in message: {s}");
        assert!(s.contains("nowhere.pty.log"), "dump path in message: {s}");

        let prompt = PtyError::timeout_for(Stage::Prompt, path.clone());
        assert!(matches!(prompt, PtyError::PromptTimeout { .. }));
        assert!(prompt.to_string().contains("prompt"));

        let query = PtyError::timeout_for(Stage::Query, path.clone());
        assert!(matches!(query, PtyError::QueryTimeout { .. }));
        assert!(query.to_string().contains("query"));

        // Stage labels are stable identifiers.
        assert_eq!(Stage::Connect.label(), "connect");
        assert_eq!(Stage::Prompt.label(), "prompt");
        assert_eq!(Stage::Query.label(), "query");

        // IO variant formats the inner error.
        let io_err = PtyError::Io(io::Error::other("boom"));
        assert!(io_err.to_string().contains("boom"));
    }

    /// TC107 — `detect_cpr_query` recognises the DSR cursor-position query
    /// (`ESC [ 6 n`) anywhere within a chunk, and ignores chunks without it.
    #[test]
    fn cpr_detection_returns_true_for_dsr_query_bytes() {
        // Exact sequence.
        assert!(detect_cpr_query(DSR_CURSOR_QUERY));
        // Embedded in surrounding output.
        assert!(detect_cpr_query(b"some output\x1b[6nmore output"));
        // At the very end of a chunk.
        assert!(detect_cpr_query(b"trailing\x1b[6n"));

        // Negative cases: no query present.
        assert!(!detect_cpr_query(b""));
        assert!(!detect_cpr_query(b"no escape sequences here"));
        // A different CSI sequence must not match.
        assert!(!detect_cpr_query(b"\x1b[1;1R"));
        // Partial / truncated query must not match.
        assert!(!detect_cpr_query(b"\x1b[6"));
    }

    /// TC-66-U03 — `Timeouts::from_env` parses the three override variables
    /// and falls back to defaults when they are absent or unparseable.
    #[test]
    fn timeouts_from_env_parses_overrides_and_falls_back() {
        let _g = env_lock().lock().unwrap_or_else(|p| p.into_inner());

        // Clean slate.
        for k in [
            "TQ_TEST_CONNECT_TIMEOUT",
            "TQ_TEST_PROMPT_TIMEOUT",
            "TQ_TEST_QUERY_TIMEOUT",
        ] {
            std::env::remove_var(k);
        }

        // Defaults when unset.
        let t = Timeouts::from_env();
        assert_eq!(t.connect, Duration::from_secs(Timeouts::DEFAULT_CONNECT_SECS));
        assert_eq!(t.prompt, Duration::from_secs(Timeouts::DEFAULT_PROMPT_SECS));
        assert_eq!(t.query, Duration::from_secs(Timeouts::DEFAULT_QUERY_SECS));

        // Overrides take effect.
        std::env::set_var("TQ_TEST_CONNECT_TIMEOUT", "90");
        std::env::set_var("TQ_TEST_PROMPT_TIMEOUT", "22");
        std::env::set_var("TQ_TEST_QUERY_TIMEOUT", "120");
        let t = Timeouts::from_env();
        assert_eq!(t.connect, Duration::from_secs(90));
        assert_eq!(t.prompt, Duration::from_secs(22));
        assert_eq!(t.query, Duration::from_secs(120));

        // Unparseable values fall back to defaults silently — tests should
        // never blow up on a malformed env var.
        std::env::set_var("TQ_TEST_CONNECT_TIMEOUT", "not-a-number");
        let t = Timeouts::from_env();
        assert_eq!(
            t.connect,
            Duration::from_secs(Timeouts::DEFAULT_CONNECT_SECS),
            "unparseable override falls back to default"
        );

        // for_stage dispatches correctly.
        let t = Timeouts {
            connect: Duration::from_secs(1),
            prompt: Duration::from_secs(2),
            query: Duration::from_secs(3),
        };
        assert_eq!(t.for_stage(Stage::Connect), Duration::from_secs(1));
        assert_eq!(t.for_stage(Stage::Prompt), Duration::from_secs(2));
        assert_eq!(t.for_stage(Stage::Query), Duration::from_secs(3));

        // Restore clean state for other tests.
        for k in [
            "TQ_TEST_CONNECT_TIMEOUT",
            "TQ_TEST_PROMPT_TIMEOUT",
            "TQ_TEST_QUERY_TIMEOUT",
        ] {
            std::env::remove_var(k);
        }
    }
}
