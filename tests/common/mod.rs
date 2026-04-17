//! Common test utilities for tq integration tests
//!
//! This module provides shared test infrastructure, particularly for
//! managing Teradata driver initialization in parallel test scenarios.

#[allow(dead_code)]
pub mod pty_harness;

use std::sync::{Mutex, OnceLock};

/// Global lock for tests that use the Teradata driver
///
/// The `teradatarustapi` library has a global state for driver loading.
/// When multiple integration tests run in parallel, concurrent calls to
/// `load_driver()` can cause race conditions in the underlying Go-based driver.
///
/// This lock serializes driver initialization across test threads while
/// allowing non-driver tests to run in parallel.
///
/// # Usage
///
/// ```ignore
/// #[test]
/// #[ignore]
/// fn test_live_query() {
///     common::with_driver(|| {
///         let client = create_test_client();
///         // ... test code that uses the driver ...
///     });
/// }
/// ```
static DRIVER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// Get or initialize the driver lock
fn driver_lock() -> &'static Mutex<()> {
    DRIVER_LOCK.get_or_init(|| Mutex::new(()))
}

/// Execute a closure with exclusive access to the Teradata driver
///
/// This function acquires a global lock before executing the closure,
/// ensuring that only one test thread is initializing or using the
/// driver at a time.
///
/// # Arguments
///
/// * `f` - A closure to execute with driver access
///
/// # Returns
///
/// The return value of the closure
///
/// # Panics
///
/// Panics if the mutex is poisoned (another test panicked while holding the lock)
pub fn with_driver<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let _guard = driver_lock()
        .lock()
        .expect("Driver lock poisoned - a test panicked while holding the lock");
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_driver_executes_closure() {
        let result = with_driver(|| 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_with_driver_returns_value() {
        let result = with_driver(|| "hello".to_string());
        assert_eq!(result, "hello");
    }
}
