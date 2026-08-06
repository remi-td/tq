//! Shared severity classification and coloring for monitoring commands
//!
//! Monitoring commands (`resources`, `skew`, `space`, `dbspace`, `watch`)
//! surface numeric metrics whose interpretation depends on site policy. This
//! module provides the single classifier and the single painter every one of
//! them uses, so that thresholds and ANSI codes are never embedded at a call
//! site.
//!
//! Two invariants shape the API:
//!
//! 1. **One classifier, one painter.** Commands ask [`Thresholds`] for a
//!    [`Severity`] and hand a rendered string to [`SeverityStyler::paint`].
//! 2. **Structured formats stay clean.** A [`SeverityStyler`] is only ever
//!    handed to the `table` and `markdown` renderers; `json` and `csv`
//!    renderers do not receive one, which makes it structurally impossible for
//!    escape sequences to leak into machine-readable output.

use crate::config::{MonitoringColors, MonitoringThresholds};
use nu_ansi_term::Color;

/// Three-level severity of a monitored metric
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Below the warning threshold
    Normal,
    /// At or above the warning threshold, below the critical threshold
    Warning,
    /// At or above the critical threshold
    Critical,
}

impl Severity {
    /// Lowercase name, used in structured output and tests
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Normal => "normal",
            Severity::Warning => "warning",
            Severity::Critical => "critical",
        }
    }
}

/// Classify `value` against a warning/critical pair.
///
/// Boundaries are inclusive (REQ-COLOR-001): a value exactly equal to the
/// warning threshold is [`Severity::Warning`], and a value exactly equal to the
/// critical threshold is [`Severity::Critical`].
pub fn classify(value: f64, warning: f64, critical: f64) -> Severity {
    if value >= critical {
        Severity::Critical
    } else if value >= warning {
        Severity::Warning
    } else {
        Severity::Normal
    }
}

/// Runtime view of the configured thresholds
///
/// One accessor per metric family, so call sites read declaratively and never
/// index configuration fields directly.
#[derive(Debug, Clone, Copy)]
pub struct Thresholds {
    cpu_warning: f64,
    cpu_critical: f64,
    io_warning: f64,
    io_critical: f64,
    skew_warning: f64,
    skew_critical: f64,
    space_warning: f64,
    space_critical: f64,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self::from_config(&MonitoringThresholds::default())
    }
}

impl Thresholds {
    /// Build from the deserialized configuration
    pub fn from_config(t: &MonitoringThresholds) -> Self {
        Self {
            cpu_warning: t.cpu_warning,
            cpu_critical: t.cpu_critical,
            io_warning: t.io_warning,
            io_critical: t.io_critical,
            skew_warning: t.skew_warning,
            skew_critical: t.skew_critical,
            space_warning: t.space_warning,
            space_critical: t.space_critical,
        }
    }

    /// Classify a CPU utilisation percentage
    pub fn cpu(&self, pct: f64) -> Severity {
        classify(pct, self.cpu_warning, self.cpu_critical)
    }

    /// Classify an I/O utilisation percentage
    pub fn io(&self, pct: f64) -> Severity {
        classify(pct, self.io_warning, self.io_critical)
    }

    /// Classify a skew percentage
    pub fn skew(&self, pct: f64) -> Severity {
        classify(pct, self.skew_warning, self.skew_critical)
    }

    /// Classify a space-consumption percentage (`PermUsed%`)
    pub fn space(&self, pct: f64) -> Severity {
        classify(pct, self.space_warning, self.space_critical)
    }
}

/// Parse an accepted color name into a [`Color`]
///
/// Accepts the eight ANSI-portable base names and their `bright_` variants,
/// case-insensitively. Returns `None` for any other name; configuration
/// validation rejects those before a styler is ever constructed.
pub fn parse_color(name: &str) -> Option<Color> {
    let lower = name.trim().to_ascii_lowercase();
    let (bright, base) = match lower.strip_prefix("bright_") {
        Some(rest) => (true, rest),
        None => (false, lower.as_str()),
    };

    Some(match (base, bright) {
        ("black", false) => Color::Black,
        ("red", false) => Color::Red,
        ("green", false) => Color::Green,
        ("yellow", false) => Color::Yellow,
        ("blue", false) => Color::Blue,
        ("magenta", false) => Color::Magenta,
        ("cyan", false) => Color::Cyan,
        ("white", false) => Color::White,
        ("black", true) => Color::DarkGray,
        ("red", true) => Color::LightRed,
        ("green", true) => Color::LightGreen,
        ("yellow", true) => Color::LightYellow,
        ("blue", true) => Color::LightBlue,
        ("magenta", true) => Color::LightMagenta,
        ("cyan", true) => Color::LightCyan,
        ("white", true) => Color::LightGray,
        _ => return None,
    })
}

/// Owns the resolved palette and the enable flag
///
/// Constructed once per command invocation so the color-mode decision and the
/// color-name lookup happen once, not per rendered cell.
#[derive(Debug, Clone)]
pub struct SeverityStyler {
    enabled: bool,
    normal: Option<Color>,
    warning: Option<Color>,
    critical: Option<Color>,
}

impl SeverityStyler {
    /// Build a styler.
    ///
    /// `use_color` must already account for `--color auto/always/never`,
    /// `NO_COLOR` and TTY detection — it comes from
    /// `ColorChoice::should_use_color()`.
    pub fn new(colors: &MonitoringColors, use_color: bool) -> Self {
        Self {
            enabled: use_color,
            normal: parse_color(&colors.normal),
            warning: parse_color(&colors.warning),
            critical: parse_color(&colors.critical),
        }
    }

    /// A styler that never emits escape sequences
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            normal: None,
            warning: None,
            critical: None,
        }
    }

    /// Whether this styler will emit escape sequences
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Paint `text` according to `severity`.
    ///
    /// Returns `text` unchanged when color is disabled or the severity has no
    /// resolvable color, so call sites need no branch of their own.
    pub fn paint(&self, severity: Severity, text: &str) -> String {
        if !self.enabled {
            return text.to_string();
        }
        let color = match severity {
            Severity::Normal => self.normal,
            Severity::Warning => self.warning,
            Severity::Critical => self.critical,
        };
        match color {
            Some(c) => c.paint(text).to_string(),
            None => text.to_string(),
        }
    }

    /// Paint an optional metric.
    ///
    /// `None` means "no measurement" and is rendered unstyled: per REQ-COLOR-002
    /// a NULL metric never implies Warning or Critical.
    pub fn paint_optional(
        &self,
        severity: Option<Severity>,
        text: &str,
    ) -> String {
        match severity {
            Some(s) => self.paint(s, text),
            None => text.to_string(),
        }
    }
}

/// Everything a monitoring command needs to classify and color its output
///
/// Passed as a single borrowed struct rather than as additional parameters, so
/// command signatures do not trip `clippy::too_many_arguments` (the same reason
/// `FastloadOptions` exists).
#[derive(Debug, Clone)]
pub struct MonitoringContext {
    /// Resolved severity thresholds
    pub thresholds: Thresholds,
    /// Resolved palette and enable flag
    pub styler: SeverityStyler,
    /// Default watch-mode refresh interval in seconds, used when a command's
    /// own `--interval` flag is absent
    pub refresh_interval: u64,
}

impl Default for MonitoringContext {
    fn default() -> Self {
        Self {
            thresholds: Thresholds::default(),
            styler: SeverityStyler::disabled(),
            refresh_interval: MonitoringThresholds::default().refresh_interval,
        }
    }
}

impl MonitoringContext {
    /// Build from configuration and the already-resolved color mode
    pub fn new(
        thresholds: &MonitoringThresholds,
        colors: &MonitoringColors,
        use_color: bool,
    ) -> Self {
        Self {
            thresholds: Thresholds::from_config(thresholds),
            styler: SeverityStyler::new(colors, use_color),
            refresh_interval: thresholds.refresh_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // classify boundary behaviour (REQ-COLOR-001)
    // =========================================================================

    #[test]
    fn test_classify_below_warning_is_normal() {
        assert_eq!(classify(69.9, 70.0, 90.0), Severity::Normal);
    }

    #[test]
    fn test_classify_exactly_warning_is_warning() {
        assert_eq!(classify(70.0, 70.0, 90.0), Severity::Warning);
    }

    #[test]
    fn test_classify_between_is_warning() {
        assert_eq!(classify(80.0, 70.0, 90.0), Severity::Warning);
    }

    #[test]
    fn test_classify_exactly_critical_is_critical() {
        assert_eq!(classify(90.0, 70.0, 90.0), Severity::Critical);
    }

    #[test]
    fn test_classify_above_critical_is_critical() {
        assert_eq!(classify(120.0, 70.0, 90.0), Severity::Critical);
    }

    #[test]
    fn test_classify_zero_is_normal() {
        assert_eq!(classify(0.0, 70.0, 90.0), Severity::Normal);
    }

    // =========================================================================
    // Thresholds accessors dispatch to the right pair
    // =========================================================================

    #[test]
    fn test_thresholds_defaults_dispatch_independently() {
        let t = Thresholds::default();
        // 75 is Warning for cpu (70/90) but Normal for io (80/95)
        assert_eq!(t.cpu(75.0), Severity::Warning);
        assert_eq!(t.io(75.0), Severity::Normal);
        // 75 is Critical for skew (40/70) and Normal for space (80/90)
        assert_eq!(t.skew(75.0), Severity::Critical);
        assert_eq!(t.space(75.0), Severity::Normal);
    }

    #[test]
    fn test_thresholds_from_config_is_used() {
        let cfg = MonitoringThresholds {
            skew_warning: 10.0,
            skew_critical: 20.0,
            ..MonitoringThresholds::default()
        };
        let t = Thresholds::from_config(&cfg);
        assert_eq!(t.skew(9.0), Severity::Normal);
        assert_eq!(t.skew(10.0), Severity::Warning);
        assert_eq!(t.skew(20.0), Severity::Critical);
    }

    #[test]
    fn test_space_thresholds() {
        let t = Thresholds::default();
        assert_eq!(t.space(79.9), Severity::Normal);
        assert_eq!(t.space(80.0), Severity::Warning);
        assert_eq!(t.space(90.0), Severity::Critical);
    }

    // =========================================================================
    // Color-name parsing
    // =========================================================================

    #[test]
    fn test_parse_color_base_names() {
        for name in [
            "black", "red", "green", "yellow", "blue", "magenta", "cyan", "white",
        ] {
            assert!(parse_color(name).is_some(), "{name} should parse");
        }
    }

    #[test]
    fn test_parse_color_bright_variants() {
        assert_eq!(parse_color("bright_red"), Some(Color::LightRed));
        assert_eq!(parse_color("bright_green"), Some(Color::LightGreen));
        assert_eq!(parse_color("bright_yellow"), Some(Color::LightYellow));
    }

    #[test]
    fn test_parse_color_case_insensitive() {
        assert_eq!(parse_color("YELLOW"), Some(Color::Yellow));
        assert_eq!(parse_color("  Bright_Red "), Some(Color::LightRed));
    }

    #[test]
    fn test_parse_color_unknown_rejected() {
        assert!(parse_color("orange").is_none());
        assert!(parse_color("#ff0000").is_none());
        assert!(parse_color("").is_none());
        assert!(parse_color("bright_orange").is_none());
    }

    // =========================================================================
    // SeverityStyler
    // =========================================================================

    #[test]
    fn test_styler_disabled_emits_no_escapes() {
        let styler = SeverityStyler::new(&MonitoringColors::default(), false);
        assert!(!styler.is_enabled());
        for sev in [Severity::Normal, Severity::Warning, Severity::Critical] {
            let painted = styler.paint(sev, "42.0");
            assert_eq!(painted, "42.0");
            assert!(!painted.contains('\x1b'));
        }
    }

    #[test]
    fn test_styler_enabled_emits_escapes_for_each_severity() {
        let styler = SeverityStyler::new(&MonitoringColors::default(), true);
        assert!(styler.is_enabled());
        for sev in [Severity::Normal, Severity::Warning, Severity::Critical] {
            let painted = styler.paint(sev, "42.0");
            assert!(painted.contains('\x1b'), "{sev:?} should be painted");
            assert!(painted.contains("42.0"));
        }
    }

    #[test]
    fn test_styler_distinct_colors_per_severity() {
        let styler = SeverityStyler::new(&MonitoringColors::default(), true);
        let normal = styler.paint(Severity::Normal, "x");
        let warning = styler.paint(Severity::Warning, "x");
        let critical = styler.paint(Severity::Critical, "x");
        assert_ne!(normal, warning);
        assert_ne!(warning, critical);
        assert_ne!(normal, critical);
    }

    #[test]
    fn test_styler_unknown_color_falls_back_to_plain() {
        // Validation rejects this configuration; the styler still degrades
        // gracefully rather than panicking.
        let colors = MonitoringColors {
            normal: "orange".to_string(),
            ..MonitoringColors::default()
        };
        let styler = SeverityStyler::new(&colors, true);
        assert_eq!(styler.paint(Severity::Normal, "x"), "x");
        assert!(styler.paint(Severity::Warning, "x").contains('\x1b'));
    }

    #[test]
    fn test_styler_paint_optional_none_is_unstyled() {
        let styler = SeverityStyler::new(&MonitoringColors::default(), true);
        let painted = styler.paint_optional(None, "[--]");
        assert_eq!(painted, "[--]");
        assert!(!painted.contains('\x1b'));
    }

    #[test]
    fn test_styler_paint_optional_some_is_styled() {
        let styler = SeverityStyler::new(&MonitoringColors::default(), true);
        let painted = styler.paint_optional(Some(Severity::Critical), "99.0");
        assert!(painted.contains('\x1b'));
    }

    #[test]
    fn test_styler_disabled_constructor() {
        let styler = SeverityStyler::disabled();
        assert!(!styler.is_enabled());
        assert_eq!(styler.paint(Severity::Critical, "boom"), "boom");
    }

    // =========================================================================
    // MonitoringContext
    // =========================================================================

    #[test]
    fn test_context_default_is_uncolored_with_default_thresholds() {
        let ctx = MonitoringContext::default();
        assert!(!ctx.styler.is_enabled());
        assert_eq!(ctx.thresholds.cpu(95.0), Severity::Critical);
        assert_eq!(ctx.refresh_interval, 6);
    }

    #[test]
    fn test_context_new_wires_config_through() {
        let thresholds = MonitoringThresholds {
            space_warning: 10.0,
            space_critical: 20.0,
            ..MonitoringThresholds::default()
        };
        let ctx = MonitoringContext::new(&thresholds, &MonitoringColors::default(), true);
        assert_eq!(ctx.thresholds.space(15.0), Severity::Warning);
        assert!(ctx.styler.is_enabled());
        assert_eq!(ctx.refresh_interval, 6);
    }

    #[test]
    fn test_severity_as_str() {
        assert_eq!(Severity::Normal.as_str(), "normal");
        assert_eq!(Severity::Warning.as_str(), "warning");
        assert_eq!(Severity::Critical.as_str(), "critical");
    }
}
