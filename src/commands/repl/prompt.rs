//! REPL prompt handling
//!
//! Provides customized prompts for the tq interactive shell.
//! The prompt changes based on the current state (normal vs multi-line input).
//!
//! Sprint 13: Prompts now use Teradata orange (#F37021) per branding guidelines.

use super::state::ReplState;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};
use std::borrow::Cow;

/// Custom prompt for the tq REPL
///
/// Sprint 13: Prompts colored in Teradata orange per branding-guidelines.md
#[derive(Clone)]
pub struct TqPrompt {
    /// Base prompt text (with ANSI color codes)
    normal_prompt: String,
    /// Continuation prompt for multi-line input (with ANSI color codes)
    continuation_prompt: String,
}

impl TqPrompt {
    /// Create a new TqPrompt with Teradata orange colored prompts
    ///
    /// Sprint 13: Per branding-guidelines.md, prompts should be in Teradata orange (#F37021)
    /// Uses xterm-256 color 202 for better terminal compatibility
    pub fn new() -> Self {
        // Teradata orange: xterm-256 color 202 (closest match to #F37021)
        // ANSI 256-color escape: \x1b[38;5;Nm where N is the color index
        // Reset escape: \x1b[0m
        let orange_start = "\x1b[38;5;202m";
        let reset = "\x1b[0m";

        Self {
            normal_prompt: format!("{}tq> {}", orange_start, reset),
            continuation_prompt: format!("{}...> {}", orange_start, reset),
        }
    }

    /// Create a prompt instance for the current state
    pub fn for_state(&self, state: &ReplState) -> StatefulPrompt {
        StatefulPrompt {
            prompt: self.clone(),
            is_multiline: state.is_multiline(),
        }
    }
}

impl Default for TqPrompt {
    fn default() -> Self {
        Self::new()
    }
}

/// A prompt that carries state information
pub struct StatefulPrompt {
    prompt: TqPrompt,
    is_multiline: bool,
}

impl Prompt for StatefulPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        if self.is_multiline {
            Cow::Borrowed(&self.prompt.continuation_prompt)
        } else {
            Cow::Borrowed(&self.prompt.normal_prompt)
        }
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("...> ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!(
            "({}reverse-i-search)`{}': ",
            prefix, history_search.term
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::LogonMechanism;
    use crate::db::ConnectionConfig;
    use std::time::Duration;

    fn create_test_config() -> ConnectionConfig {
        ConnectionConfig {
            host: "testhost".to_string(),
            port: 1025,
            database: "testdb".to_string(),
            user: "testuser".to_string(),
            password: None,
            logmech: LogonMechanism::Td2,
            timeout: Duration::from_secs(30),
            query_timeout: None,
        }
    }

    #[test]
    fn test_normal_prompt() {
        let prompt = TqPrompt::new();
        let config = create_test_config();
        let state = ReplState::new(config);

        let stateful = prompt.for_state(&state);
        // Sprint 13: Prompt now includes Teradata orange ANSI color codes (xterm-256 color 202)
        let rendered = stateful.render_prompt_left();
        assert!(rendered.contains("tq> "), "Prompt should contain 'tq> '");
        assert!(
            rendered.contains("\x1b[38;5;202m"),
            "Prompt should have Teradata orange color (xterm-256 202)"
        );
    }

    #[test]
    fn test_multiline_prompt() {
        let prompt = TqPrompt::new();
        let config = create_test_config();
        let mut state = ReplState::new(config);
        state.append_input("SELECT");

        let stateful = prompt.for_state(&state);
        // Sprint 13: Continuation prompt now includes Teradata orange ANSI color codes (xterm-256 color 202)
        let rendered = stateful.render_prompt_left();
        assert!(rendered.contains("...> "), "Prompt should contain '...> '");
        assert!(
            rendered.contains("\x1b[38;5;202m"),
            "Prompt should have Teradata orange color (xterm-256 202)"
        );
    }
}
