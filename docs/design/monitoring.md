# Monitoring Thresholds, Severity and Colors — Design

## Overview

Monitoring commands (`sessions`, `resources`, `skew`, `locks`, `space`, `dbspace`) surface
numeric metrics whose interpretation depends on site policy. This document describes the
shared severity layer that classifies a metric into `Normal` / `Warning` / `Critical`, the
configuration that drives it, and the single rendering helper every monitoring command uses
so that coloring is implemented exactly once.

Two goals shape the design:

1. **One classifier, one painter.** Commands must not embed magic numbers or ANSI codes.
   They ask the shared layer for a `Severity` and hand a string to a painter.
2. **Structured formats stay clean.** ANSI escapes may only ever appear in the `table`
   renderer. `json`, `csv` and `markdown` are machine-consumed and must be byte-identical
   whether or not color is enabled.

---

## Functional Building Blocks

| Concern | Location |
|---------|----------|
| Config deserialization + defaults | `src/config.rs` (`MonitoringSettings`) |
| Validation of user-supplied thresholds | `src/config.rs` (`MonitoringSettings::validate`) |
| Severity classification | `src/commands/severity.rs` (`Severity`, `Thresholds`) |
| Color resolution and painting | `src/commands/severity.rs` (`SeverityStyler`) |
| Color mode decision (auto/always/never) | `src/cli.rs:1497` (`ColorChoice::should_use_color`) |
| Refresh interval resolution | `src/config.rs` + `src/main.rs` watch dispatch |

---

## Configuration Schema

New optional `[monitoring]` tree in `~/.tq/config.toml` and `.tq.toml`:

```toml
[monitoring.thresholds]
cpu_warning      = 70
cpu_critical     = 90
io_warning       = 80
io_critical      = 95
skew_warning     = 40
skew_critical    = 70
space_warning    = 80    # percent of MaxPerm consumed
space_critical   = 90
refresh_interval = 6     # seconds, default for watch mode

[monitoring.colors]
normal   = "green"
warning  = "yellow"
critical = "red"
```

### Types

Every key is optional; the whole `[monitoring]` table is optional. This is expressed with
`#[serde(default)]` on the structs plus a `Default` impl carrying the built-in values, which
matches how `OutputSettings` and `ReplSettings` are declared at `src/config.rs:83` and
`src/config.rs:107`.

```rust
// src/config.rs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitoringSettings {
    pub thresholds: MonitoringThresholds,
    pub colors: MonitoringColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitoringThresholds {
    pub cpu_warning: f64,
    pub cpu_critical: f64,
    pub io_warning: f64,
    pub io_critical: f64,
    pub skew_warning: f64,
    pub skew_critical: f64,
    pub space_warning: f64,
    pub space_critical: f64,
    pub refresh_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MonitoringColors {
    pub normal: String,
    pub warning: String,
    pub critical: String,
}
```

`MonitoringSettings` is added as a `pub monitoring: MonitoringSettings` field on `Config`
(`src/config.rs:25`). Because `Config::load()` seeds Figment with
`Serialized::defaults(Config::default())` (`src/config.rs:160`), partial user tables merge
over the defaults key-by-key — a config that sets only `cpu_warning` leaves the other eight
values at their built-in defaults with no extra code.

Defaults are the values shown above.

### Environment variables are not supported for this tree

`Config::load()` merges `Env::prefixed("TQ_").split("_")` (`src/config.rs:190`). Because the
threshold keys themselves contain underscores, `TQ_MONITORING_THRESHOLDS_CPU_WARNING` would
split into `monitoring.thresholds.cpu.warning` and fail to bind. Threshold configuration is
therefore file-only by design; this is documented in the configuration specification rather
than worked around, since introducing a second env provider with different splitting rules
would make precedence harder to reason about.

---

## Validation

Thresholds are validated, not merely parsed. Rules:

- Every percentage threshold must lie in `0.0..=100.0`.
- For each metric family, `warning < critical` (strictly, per REQ-MON-005 — equal values are
  rejected because they would make the `Warning` band empty).
- `refresh_interval` must lie in `2..=300`, matching the range already enforced by clap on
  `--interval`.
- Each color name must be one of the accepted values.

Validation runs in a single pass and reports **every** violation it finds, so a
misconfigured file is fixed in one edit rather than one `tq` invocation per mistake
(REQ-MON-010).

```rust
impl MonitoringSettings {
    pub fn validate(&self) -> Result<()>;
}
```

Failures produce `TqError::ConfigParseError` with a message naming the offending key and the
accepted range, for example:

```
Error: invalid configuration in [monitoring.thresholds]:
  cpu_warning (95) must not exceed cpu_critical (90)
```

### Where validation is invoked

Validation deliberately does **not** live inside `Config::load()`. `src/main.rs:62` treats a
`Config::load()` error as non-fatal and falls back to `Config::default()`:

```rust
let config = Config::load().unwrap_or_else(|e| { log::warn!(...); Config::default() });
```

A misconfigured threshold silently reverting to defaults would violate the requirement that
bad values yield a descriptive error. Validation is therefore an explicit call in `main`
immediately after the config is loaded, whose `Result` is propagated:

```rust
config.monitoring.validate()?;
```

This keeps the existing graceful-degradation behaviour for I/O and syntax problems while
making semantic threshold errors hard failures.

---

## Severity Model

```rust
// src/commands/severity.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity { Normal, Warning, Critical }
```

Classification is a single free function over a resolved threshold pair, so there is exactly
one place where the boundary semantics (inclusive at the threshold) are defined:

```rust
pub fn classify(value: f64, warning: f64, critical: f64) -> Severity {
    if value >= critical { Severity::Critical }
    else if value >= warning { Severity::Warning }
    else { Severity::Normal }
}
```

`Thresholds` is the runtime view of `MonitoringThresholds` and exposes one accessor per
metric family so call sites read declaratively and never index config fields directly:

```rust
pub struct Thresholds { /* copied from MonitoringThresholds */ }

impl Thresholds {
    pub fn cpu(&self, pct: f64)   -> Severity;
    pub fn io(&self, pct: f64)    -> Severity;
    pub fn skew(&self, pct: f64)  -> Severity;
    pub fn space(&self, pct: f64) -> Severity;
}
```

`Option<f64>` metrics (idle sessions, NULL skew) are handled by the caller: `None` means
"no measurement", which is rendered unstyled rather than as `Normal`. This is why the
accessors take `f64` and not `Option<f64>` — mapping absent data to a severity would be a
silent lie.

---

## Styler

`SeverityStyler` owns the resolved palette and the enable flag. Constructing it once per
command invocation means the color-mode decision and the color-name lookup each happen once,
not per cell.

```rust
pub struct SeverityStyler {
    enabled: bool,
    normal:   Option<nu_ansi_term::Color>,
    warning:  Option<nu_ansi_term::Color>,
    critical: Option<nu_ansi_term::Color>,
}

impl SeverityStyler {
    /// `use_color` comes from `ColorChoice::should_use_color()` — already accounts for
    /// --color auto/always/never, NO_COLOR and TTY detection.
    pub fn new(colors: &MonitoringColors, use_color: bool) -> Self;

    /// Returns `text` unchanged when disabled or when the severity has no mapped color.
    pub fn paint(&self, severity: Severity, text: &str) -> String;

    pub fn is_enabled(&self) -> bool;
}
```

### Color-name resolution

Names are parsed case-insensitively into `nu_ansi_term::Color` (the crate is already a
dependency, and is used by the REPL highlighter at `src/commands/repl/highlighter.rs`).
Accepted: `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` and their
`bright_*` variants — exactly the set enumerated by REQ-MON-004, no more. An unrecognised
name is a validation error surfaced by `MonitoringSettings::validate()` alongside the
numeric checks, so it is caught at startup rather than producing an uncolored surprise
mid-render.

Name validity is asserted in two places for a reason. `config::is_valid_color_name` answers
"may this string appear in a config file?" without pulling a rendering crate into the
configuration layer; `severity::parse_color` performs the actual `&str -> Color` resolution.
`SeverityStyler` stores `Option<Color>` and degrades an unresolvable name to "unstyled"
rather than panicking, so a hypothetical divergence between the two is a cosmetic
degradation, never a crash.

### Honouring color mode

`SeverityStyler::new` receives the already-computed `use_color` boolean rather than
re-deriving it. `main` computes it once at `src/main.rs:110` and threads it into every
command; `--color never`, `NO_COLOR`, and a non-TTY stdout are all resolved there by
`ColorChoice::should_use_color()` (`src/cli.rs:1497`). When `enabled` is false, `paint`
returns the input string verbatim — no allocation of escape sequences, and no branch needed
at the call sites.

### Structured formats

`display_table` and `display_markdown` receive the `MonitoringContext`; `display_json` and
`display_csv` do not take one at all. Because the styler is unreachable from the JSON and CSV
renderers, ANSI codes cannot leak into machine-readable output — a `--format json --color
always` invocation still produces clean JSON, and this is enforced by the type system rather
than by a runtime check.

Markdown is colored (REQ-COLOR-007): it is frequently read directly in a terminal, and when
it is written to a file the color-mode resolution has already disabled the styler, so no
escape reaches the file.

---

## Refresh Interval Resolution

`refresh_interval` becomes the default for watch mode. Today `--interval` is declared with
`default_value = "6"` on three arg structs (`src/cli.rs:744`, `src/cli.rs:807`,
`src/cli.rs:1288`), which makes "user asked for 6" indistinguishable from "user said
nothing" — so config could never win.

The fix is to drop the clap default and make the field optional:

```rust
#[arg(long, value_name = "SECONDS", requires = "watch",
      value_parser = clap::value_parser!(u64).range(2..=300))]
pub interval: Option<u64>,
```

Resolution happens at the dispatch site in `src/main.rs`, where both the args and the config
are in scope:

```rust
commands::watch::run_watch(args.interval.unwrap_or(refresh_interval), ...)
```

Precedence is therefore CLI flag > config > built-in default (6), consistent with the rest of
the configuration hierarchy documented in `docs/design/configuration.md`. `run_watch` keeps
its `u64` parameter and is unaffected.

The REPL reaches the same value through `MonitoringContext::refresh_interval`, which is
carried on the context rather than looked up from config a second time. `/resources --watch`
passes it as the default to `parse_watch_args`, which previously hard-coded `6`.

---

## Integration Into Existing Commands

The `use_color: bool` parameter is already threaded into every monitoring command's
`execute` signature — `skew::execute` (`src/commands/skew.rs:151`) currently ignores it as
`_use_color`. Adding severity requires each command to additionally receive the resolved
`Thresholds` and `MonitoringColors`.

Rather than growing every signature by two more parameters (which invites
`clippy::too_many_arguments`, the lint that motivated `FastloadOptions` in
`src/commands/fastload.rs`), a single borrowed context struct is passed:

```rust
// src/commands/severity.rs
pub struct MonitoringContext {
    pub thresholds: Thresholds,
    pub styler: SeverityStyler,
    pub refresh_interval: u64,
}
```

The context owns its data rather than borrowing `&'a MonitoringColors`: `SeverityStyler`
already resolves color *names* into `Option<Color>` at construction time, so nothing needs
to read the raw configuration again afterwards. Dropping the lifetime keeps the struct
storable on `ReplState` (see below) without infecting it with a lifetime parameter. It is
still passed by reference (`&MonitoringContext`) at every call site, which is what avoids
`clippy::too_many_arguments`.

`main` builds one `MonitoringContext` after config validation and passes `&ctx` to
`skew`, `resources`, `space` and `dbspace`. The existing `use_color` parameter is retained
where commands use it for non-severity purposes; where its only use was severity
(`skew::execute`, `resources::execute`, both of which took it as `_use_color`), it is
subsumed by the context and removed.

### Reaching the context from the REPL

REPL metacommand handlers receive `state: &mut ReplState` and `writer`, not a context.
Threading a new parameter through `handle_metacommand` and `handle_metacommand_with_state`
would ripple into every caller and test for the benefit of three arms, so the context is
stored as `ReplState::monitoring` instead, set once in `repl::execute`. Handlers read
`&state.monitoring`. This also gives `/resources --watch` access to `refresh_interval`
without a second config load.

### Per-command mapping

| Command | Metric | Threshold family |
|---------|--------|------------------|
| `skew` | CPU skew %, I/O skew % | `skew` |
| `resources` | `AvgCPUBusy`, `PeakCPUBusy` | `cpu` |
| `resources` | `AvgIO*`, `PeakIO*` normalized to a share of the busiest observed value | `io` |
| `resources` | CPU / I/O skew summary footer | `skew` |
| `space` / `dbspace` | perm/spool/temp skew % | `skew` |
| `space` / `dbspace` | `PermUsed%` | `space` |

`resources` reports I/O as an absolute count or KB figure, not a percentage, so there is no
scale to compare a threshold against directly. It is normalized against the largest value
observed in the same sample (`io_scale`), which makes the `io_warning` / `io_critical` pair
mean "this VPROC is carrying at least N% of the busiest VPROC's I/O". When nothing has been
observed the scale is `None` and the cells carry no severity, rather than defaulting to
`Normal` and implying a measurement that does not exist.

### Severity and the skew interpretation ladder are separate axes

`tq skew` already prints a four-word textual interpretation (`good` / `moderate` / `high` /
`severe`) driven by fixed bands at 10/30/60. That vocabulary is **preserved verbatim**; it
describes the shape of the distribution and is a stable part of the command's output.

Severity is a *different* axis: it answers "does this cross the threshold this site cares
about?", is driven by `skew_warning` / `skew_critical`, and is expressed purely as color.
Collapsing the four words into three levels would have been a user-visible regression for a
purely internal tidiness gain, and would have made the words silently change meaning when a
DBA edited a threshold. The two therefore coexist: the numeric skew cell is colored, the
interpretation word is not.

Concretely, `skew_cell()` in `src/commands/skew.rs` paints the percentage, while
`format_skew_with_hint()` and the `interpretation` match are untouched.

---

## Testing Strategy

Unit tests live beside the code in `src/commands/severity.rs` and `src/config.rs`:

- `classify` boundary behaviour: exactly at `warning`, exactly at `critical`, just below each.
- `Thresholds` accessors dispatch to the right pair.
- `SeverityStyler::paint` returns input unchanged when `enabled == false` — asserted by
  checking the result contains no `\x1b`.
- `SeverityStyler::paint` emits an escape when enabled, for each severity.
- Color-name parsing: valid names, case-insensitivity, `none`, unknown name rejected.
- `MonitoringSettings::validate`: warning > critical rejected per family; negative rejected;
  above 100 rejected; `refresh_interval` outside `2..=300` rejected; all-defaults accepted.
- Partial TOML merge: a config supplying only `cpu_warning` leaves the remaining keys at
  their defaults.
- Interval precedence: `Some(30)` wins over a configured 10; `None` yields the configured 10;
  `None` with default config yields 6.

Renderer tests assert that `display_json`, `display_csv` and `display_markdown` output
contains no `\x1b` even when a styler is enabled elsewhere in the process.

---

## Code Linkage

| Element | Location |
|---------|----------|
| `Config` struct | `src/config.rs:25` |
| `Config::load` (Figment merge order) | `src/config.rs:147` |
| `should_use_color` helper | `src/config.rs:449` |
| `ColorChoice::should_use_color` | `src/cli.rs:1497` |
| `use_color` computation | `src/main.rs:110` |
| Config load + fallback | `src/main.rs:62` |
| `run_watch` | `src/commands/watch.rs:119` |
| `parse_interval` | `src/commands/watch.rs:343` |
| `skew::execute` (`_use_color`) | `src/commands/skew.rs:151` |
| Skew interpretation ladder | `src/commands/skew.rs:246` |
| `nu-ansi-term` dependency | `Cargo.toml:61` |
