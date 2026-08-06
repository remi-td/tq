# Sprint 76 Planning

**Date:** 2026-08-06
**Type:** Feature Sprint
**Current Version:** 1.54.2 → target 1.55.0

---

## Reality Check Summary

- **Reviewed sprints:** 73, 74, 75
- **Patterns detected:**
  - *Healthy velocity* — 3 consecutive sprints shipped with 100% test pass rate, 0 clippy warnings, 0 regressions.
  - *No stuck issues* — no bug or feature recurred across sprints.
  - *No accumulating debt* — Sprint 74 explicitly retired the fastload delimiter debt; Sprint 75 reported none.
  - ⚠️ *Minor framework smell (non-blocking)* — the "Cost Metrics" tables in `sprint-74-review.md` and `sprint-75-review.md` are verbatim copies of Sprint 73's numbers (same session id, same 5,476,296 total). Metrics collection is reporting stale data rather than per-sprint data. Logged for Phase 6, not a crisis.
- **Decision:** **Feature Sprint**
- **Rationale:** Delivery is healthy and improving. No crisis warrants a maintenance sprint. Proceed with feature work; address the metrics-collection smell during Phase 6 (Framework Optimization).

---

## Objectives

1. **Perm space analysis (#54)** — Give DBAs a one-command view of space usage for any database or object, without hand-writing DBC queries.
2. **PMON alerting & threshold configuration (#23)** — Make monitoring thresholds, colors, and refresh rate configurable, and apply color-coded severity to monitoring output.
3. **Release v1.55.0** — Ship with full docs, tests, and a tagged release.

---

## Scope

### In Scope

#### Feature 1 — Space analysis commands (#54) — **P0**

Three invocation shapes:

| Command | Behaviour |
|---------|-----------|
| `tq space <database>` | One header row for the database's own space, followed by one row per object directly under it. |
| `tq space <database>.<object>` | Single row for that object. |
| `tq dbspace <database>` | Database-level space only. Errors clearly if the argument names a non-database or is qualified as `db.object`. |

Metrics returned:
- Object level: `CurrentPerm`, `PeakPerm`, perm skew %.
- Database level: additionally `MaxPerm` and % allocated used; plus **Spool** and **Temp** (current / allocated / peak / skew).

Sources: `DBC.DiskSpaceV` (database level), `DBC.TableSizeV` (object level). Skew formula per issue:
`100 - (AVG(CurrentPerm) / NULLIFZERO(MAX(CurrentPerm)) * 100)`.

All four output formats supported (table, json, csv, markdown), consistent with `tq skew`.

#### Feature 2 — Monitoring thresholds & colors (#23) — **P1**

Config additions in `~/.tq/config.toml` / `.tq.toml`:

```toml
[monitoring.thresholds]
cpu_warning = 70
cpu_critical = 90
io_warning = 80
io_critical = 95
skew_warning = 40
skew_critical = 70
space_warning = 80      # % of MaxPerm used
space_critical = 90
refresh_interval = 6    # seconds

[monitoring.colors]
normal = "green"
warning = "yellow"
critical = "red"
```

Behaviour:
- Thresholds resolve with sane built-in defaults; partial config is valid (missing keys fall back).
- Severity classification (`Normal` / `Warning` / `Critical`) applied to monitoring output: `resources`, `watch`, `skew`, and the new `space` commands.
- Colors honour the existing color mode (`auto` / `always` / `never`) — never emit ANSI when piped or when `--no-color`/`NO_COLOR` applies.
- `refresh_interval` becomes the default for `tq watch` when `--interval` is not given.
- Invalid threshold values (out of range, warning > critical) produce a clear config error rather than a panic.

Covers user stories US-8.1 through US-8.5.

#### Feature 3 — Release
- Bump to `1.55.0`, update `docs/roadmap/status.md`, tag and push release.

### Out of Scope

- #21 / #22 — PMON graphical resource and session displays (sparklines, bar charts). Larger UI work; deferred to a dedicated sprint.
- #49 — Custom severity *levels* (beyond the three-level normal/warning/critical model).
- Alert *delivery* (desktop notifications, webhooks, exit codes on breach). This sprint delivers visual indication only.
- Historical space trending / growth projection.

---

## Acceptance Criteria

- [ ] `tq space <db>` returns a database header row plus one row per contained object.
- [ ] `tq space <db>.<obj>` returns exactly one row for that object.
- [ ] `tq dbspace <db>` returns database-level perm/spool/temp metrics.
- [ ] `tq dbspace <db>.<obj>` fails with a clear, actionable error message.
- [ ] Unknown database/object produces the project's standard not-found error (with spelling suggestion where the existing helpers provide one).
- [ ] Skew percentages match the SQL in issue #54 and are `NULL`-safe (no divide-by-zero).
- [ ] All space commands support `--format table|json|csv|markdown`.
- [ ] `[monitoring.thresholds]` and `[monitoring.colors]` parse from config; every key is optional and defaults apply.
- [ ] Warning/critical thresholds drive color output in `resources`, `watch`, `skew`, and `space`.
- [ ] `NO_COLOR` / `--color never` / piped output emit zero ANSI escapes.
- [ ] `tq watch` uses `refresh_interval` from config when `--interval` is absent; the CLI flag still wins.
- [ ] Misconfigured thresholds (warning > critical, negative, > 100) yield a descriptive config error.
- [ ] All SQL validated against the live Teradata database from `.env` — no fabricated DBC objects or columns.
- [ ] `cargo test` 100% pass (with execution output captured in the quality report).
- [ ] `cargo clippy -- -D warnings` clean; `scripts/ci-check.sh` passes before push.
- [ ] Specifications, design docs, and help text updated.
- [ ] Version bumped to 1.55.0, committed, pushed, and tagged.

---

## GitHub Issues

### Selected for Sprint
- **#54** — [FEATURE] Add perm space analysis (enhancement) — P0
- **#23** — [FEATURE] PMON: Alerting and Threshold Configuration (enhancement, priority-medium) — P1

### Deferred
- **#21** — PMON: Graphical Resource Displays — larger UI effort; needs its own sprint.
- **#22** — PMON: Graphical Session Displays — same, and depends on #21.
- **#49** — Set custom severity levels — superseded in part by #23; re-triage after this sprint lands.

---

## Dependencies

- Live Teradata database reachable via `.env` (`TQ_LOGON`) for SQL validation and integration tests.
- `DBC.DiskSpaceV` and `DBC.TableSizeV` access rights for the test user.
- #23 builds on the existing `resources` / `watch` / `skew` commands and the current color-mode plumbing — no external blockers.

---

## Phase 2 Design Decisions (Coordinator)

Design surfaced seven findings requiring coordinator rulings. All decisions are binding for Phase 3.

| # | Finding | Decision |
|---|---------|----------|
| 1 | `--interval` is declared `default_value = "6"` on `resources`, `locks`, `sessions` (`src/cli.rs` ~744, ~807, ~1288), so "omitted" is indistinguishable from "explicitly 6" and config could never win. | **In scope.** Change to `Option<u64>`, resolve precedence at the `main.rs` dispatch site. Required for the `refresh_interval` acceptance criterion. |
| 2 | `monitoring_utils::extract_integer` returns `None` for `Value::String`; the driver returns every `SUM(BIGINT)` as a quoted string, so all space metrics would silently read as zero. | **Add** `extract_i64_lenient` / `extract_f64_lenient` alongside the existing extractors. Do **not** change the existing contract or its test. Regression test required feeding `Value::String`. |
| 3 | `Config::load()` is called with `unwrap_or_else(→ default)` (`src/main.rs:62`), so threshold validation errors would degrade to defaults with a log warning only. | Validation is an **explicit `config.monitoring.validate()?`** call in `main`, making semantic threshold errors fatal while I/O and syntax errors still degrade gracefully. |
| 4 | `[monitoring]` cannot be set via environment variables — `Env::prefixed("TQ_").split("_")` mis-splits keys that themselves contain underscores. | **Accept as file-only.** Do not add a second env provider; that would make precedence harder to reason about. Document the limitation in `docs/specifications/configuration.md`. |
| 5 | `tq skew` prints four interpretation bands (`good`/`moderate`/`high`/`severe`) but the severity model has only two thresholds. | **Preserve all four words.** Severity/color is a separate axis driven by `skew_warning`/`skew_critical`; the existing interpretation text is unchanged. No user-visible regression, no fourth threshold pair. |
| 6 | An empty result set is ambiguous between "database holds no space" and "database name typo". | **Approved:** probe `DBC.DatabasesV` / `DBC.TablesV` on the empty-result path only, so the common case still costs one round trip. |
| 7 | Adding thresholds + palette to command signatures trips clippy `too_many_arguments`. | **Approved:** pass a single borrowed `MonitoringContext<'a>`, following the `FastloadOptions` precedent. |

### Verified DBC schema (confirmed live via `HELP VIEW`, not assumed)

- `DBC.DiskSpaceV`: `MaxPerm, MaxSpool, MaxTemp, CurrentPerm, CurrentSpool, CurrentTemp, PeakPerm, PeakSpool, PeakTemp, AllocatedPerm, AllocatedSpool, AllocatedTemp, PermSkew, SpoolSkew, TempSkew, …`
- `DBC.TableSizeV`: `Vproc, DataBaseName, AccountName, TableName, CurrentPerm, PeakPerm` — note the capital **B** in `DataBaseName`, and **no `MaxPerm`**.
- `DiskSpaceV.PermSkew`/`SpoolSkew`/`TempSkew` are the *configured permissible skew limit*, **not** measured skew. Do not use them; compute skew with the `AVG`/`MAX` formula from issue #54.
- `DBKind` is `'U'` for `demo_user` and `DBC`, so `dbspace` must accept both `'D'` and `'U'`.
- `TableSizeV` includes stored procedures (`TableKind='P'`); no kind filter applied.

### Descoped in Phase 2

- **Spelling suggestions on unknown database/object.** No fuzzy-match helper exists anywhere in `src/`. The acceptance criterion reduces to emitting the project's standard not-found error. Tests TC109-I06/I07 assert the standard message only.

### Test tooling (approved, built in Phase 3 by quality-validator)

1. ANSI-escape detection helpers (`contains_ansi` / `assert_no_ansi` / `assert_has_ansi`) — pure `std`, no new crate.
2. Promote the private `create_user_config` TOML-fixture helper out of `tests/integration_project_config_edge_cases.rs` into a shared test helper.

These are test-only and small; quality-validator owns `tests/` and builds them directly rather than round-tripping through the architect.

### Test plan accepted

`tests/strategy/sprint-76-strategy.md` — **59 numbered tests** (37 unit, 22 integration):
- TC109 Space Analysis: 27 (13 unit + 14 integration)
- TC110 Monitoring Thresholds & Colors: 32 (24 unit + 8 integration)

Phase 3 validation will verify every numbered test exists and executes.

---

## Session Budget

Two features, both additive and well-specified (#54 ships with working SQL). Estimated to complete Phases 0–6 in a single session. If Phase 3 runs long, #23's scope reduces to config parsing + `space`/`skew` coloring, deferring `watch`/`resources` coloring.
