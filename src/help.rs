//! Extended help content for tq
//!
//! This module provides detailed help text for configuration and credential topics.
//! Help content is embedded at compile time from separate text files for maintainability.

/// Get help content for configuration
///
/// Returns detailed help about configuration file format, profiles, and settings.
pub fn config_help() -> &'static str {
    include_str!("help/config.txt")
}

/// Get help content for credentials
///
/// Returns detailed help about password management and security best practices.
pub fn credentials_help() -> &'static str {
    include_str!("help/credentials.txt")
}

/// Get help content for variable substitution
///
/// Returns detailed help about YAML parameter files and `{{variable}}` syntax.
pub fn params_help() -> &'static str {
    include_str!("help/params.txt")
}

/// Get general help when no topic is specified
///
/// Returns a list of available help topics.
pub fn general_help() -> &'static str {
    "Available help topics:\n\n  \
     tq help config       Configuration file format and usage\n  \
     tq help credentials  Password and credential management\n  \
     tq help params       Variable substitution syntax and YAML parameter files\n\n\
     For command help, use:\n  \
     tq <command> --help\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_help_not_empty() {
        let help = config_help();
        assert!(!help.is_empty());
        assert!(help.contains("CONFIGURATION FILE"));
        assert!(help.contains("profiles"));
    }

    /// Sprint 36: Verify config help includes project config section
    #[test]
    fn test_config_help_includes_project_config() {
        let help = config_help();
        assert!(help.contains("PROJECT CONFIGURATION"));
        assert!(help.contains(".tq.toml"));
        assert!(help.contains(".tq.toml.example"));
    }

    /// Sprint 36: Verify config help shows 5-level precedence
    #[test]
    fn test_config_help_five_level_precedence() {
        let help = config_help();
        assert!(help.contains("1. Built-in defaults"));
        assert!(help.contains("2. User config file"));
        assert!(help.contains("3. Project config file"));
        assert!(help.contains("4. Environment variables"));
        assert!(help.contains("5. Command-line arguments"));
    }

    #[test]
    fn test_credentials_help_not_empty() {
        let help = credentials_help();
        assert!(!help.is_empty());
        assert!(help.contains("PASSWORD"));
        assert!(help.contains("0600"));
    }

    #[test]
    fn test_general_help_lists_topics() {
        let help = general_help();
        assert!(help.contains("config"));
        assert!(help.contains("credentials"));
        assert!(help.contains("params"));
    }

    #[test]
    fn test_params_help_not_empty() {
        let help = params_help();
        assert!(!help.is_empty());
        assert!(help.contains("Variable Substitution"));
        assert!(help.contains("{{"));
        assert!(help.contains("$ENV"));
    }

    #[test]
    fn test_params_help_includes_repl_section() {
        let help = params_help();
        assert!(help.contains("REPL Usage"));
        assert!(help.contains("/params load"));
        assert!(help.contains("/params unload"));
        assert!(help.contains("/params show"));
    }
}
