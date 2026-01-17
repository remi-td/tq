//! REPL prompt handling
//!
//! Provides customized prompts for the tq interactive shell.
//! The prompt changes based on the current state (normal vs multi-line input).

use super::state::ReplState;
use reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptHistorySearchStatus};
use std::borrow::Cow;

/// Custom prompt for the tq REPL
#[derive(Clone)]
pub struct TqPrompt {
    /// Base prompt text
    normal_prompt: String,
    /// Continuation prompt for multi-line input
    continuation_prompt: String,
}

impl TqPrompt {
    /// Create a new TqPrompt with default values
    pub fn new() -> Self {
        Self {
            normal_prompt: "tq> ".to_string(),
            continuation_prompt: "...> ".to_string(),
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
    fn render_prompt_left(&self) -> Cow<str> {
        if self.is_multiline {
            Cow::Borrowed(&self.prompt.continuation_prompt)
        } else {
            Cow::Borrowed(&self.prompt.normal_prompt)
        }
    }

    fn render_prompt_right(&self) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _edit_mode: PromptEditMode) -> Cow<str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<str> {
        Cow::Borrowed("...> ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        history_search: PromptHistorySearch,
    ) -> Cow<str> {
        let prefix = match history_search.status {
            PromptHistorySearchStatus::Passing => "",
            PromptHistorySearchStatus::Failing => "failing ",
        };
        Cow::Owned(format!("({}reverse-i-search)`{}': ", prefix, history_search.term))
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
        }
    }

    #[test]
    fn test_normal_prompt() {
        let prompt = TqPrompt::new();
        let config = create_test_config();
        let state = ReplState::new(config);

        let stateful = prompt.for_state(&state);
        assert_eq!(stateful.render_prompt_left(), "tq> ");
    }

    #[test]
    fn test_multiline_prompt() {
        let prompt = TqPrompt::new();
        let config = create_test_config();
        let mut state = ReplState::new(config);
        state.append_input("SELECT");

        let stateful = prompt.for_state(&state);
        assert_eq!(stateful.render_prompt_left(), "...> ");
    }
}
