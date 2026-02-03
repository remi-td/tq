//! Test Tools Module for Dimensional Testing Infrastructure
//!
//! Sprint 30: This module provides automated utilities for validating
//! terminal width constraints and visual output correctness.
//!
//! # Components
//!
//! - [`visual_validator`] - Dimensional assertion utilities for validating
//!   that table output respects terminal width constraints
//!
//! - [`terminal_simulator`] - Terminal width simulation for configurable
//!   testing environments
//!
//! # Purpose
//!
//! These tools address the Sprint 29 gap where tests validated implementation
//! artifacts (state changes, indicators) rather than user-observable outcomes
//! (readable output within terminal dimensions).
//!
//! # Example
//!
//! ```ignore
//! use tests::tools::{visual_validator, terminal_simulator};
//!
//! // Validate output fits terminal
//! let output = render_table(&data);
//! visual_validator::assert_no_overflow(&output, 80);
//!
//! // Simulate specific terminal dimensions
//! let sim = terminal_simulator::TerminalSimulator::new(120, 40);
//! sim.validate_output(&output).expect("output should fit");
//! ```

pub mod terminal_simulator;
pub mod visual_validator;
