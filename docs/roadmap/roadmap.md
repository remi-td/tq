# Product Roadmap

**Last Updated:** 2026-01-23
**Current Phase:** v1.x - Core Stability

---

## Overview

This document outlines the high-level product direction for `tq`. It describes major milestones and themes, providing strategic context for sprint planning.

---

## Release History

### v1.0 - MVP Foundation (Complete)
**Theme:** Essential database client functionality

**Delivered:**
- Core query execution (`tq query`)
- Connection testing (`tq ping`)
- Multiple output formats (table, JSON, CSV)
- Authentication support (TD2, LDAP, Kerberos)
- Secure credential handling

**Key Achievement:** Viable alternative to basic Teradata CLI tools

---

### v1.2 - Interactive Mode Foundation (Complete)
**Theme:** REPL basics for exploratory analysis

**Delivered:**
- Interactive REPL mode (`tq repl`)
- Multi-line SQL editing
- Command history (in-memory)
- Basic metacommands (`/help`, `/quit`, `/session`)

**Key Achievement:** Interactive exploration capability

---

### v1.3 - Enhanced REPL (Complete)
**Theme:** Rich interactive experience

**Delivered:**
- SQL syntax highlighting
- Result paging (horizontal and vertical scrolling)
- Persistent command history
- Vi/Emacs keybindings
- Schema inspection (`/describe`, `/ping`)

**Key Achievement:** Comparable to `psql` for PostgreSQL

---

### v1.4-1.5 - Advanced REPL (Complete)
**Theme:** Productivity and intelligence

**Delivered:**
- Tab completion (keywords, tables, columns)
- Context-aware completion
- Multi-line completion support
- Dynamic connection switching (`/logon`)
- Pager control (`/pager`, `/colors`)

**Key Achievement:** Intelligent, productivity-focused tool

---

### v1.6 - Quality & Features (Complete)
**Theme:** Batch mode, exports, and branding

**Delivered:**
- Batch mode foundation (file input, stdin, multiple statements)
- Export to clipboard and files
- Professional branding (logo, colors)
- Enhanced error messages
- Export syntax simplification

**Key Achievement:** Complete tool for interactive and scripted use

---

### v1.7 - Configuration & Help (Complete)
**Theme:** Usability and configuration management

**Delivered:**
- User configuration files (`~/.tq/config.toml`)
- Connection profiles
- Help system (`tq help config`, `tq help credentials`)
- Profile listing (`tq profiles`)
- Enhanced security (password file permissions)

**Key Achievement:** Enterprise-ready configuration management

---

## Future Phases

### v1.x - Core Stability (Current)
**Theme:** Bug fixes, stability, and polish

**Focus:**
- Fix critical user-facing bugs
- Improve test coverage
- Refine UX based on feedback
- Maintain 100% test pass rate
- Zero technical debt tolerance

**Target:** Production-ready stability

---

### v2.0 - Advanced Features
**Theme:** Power user features and automation

**Planned:**
- Transaction control (`--atomic` flag)
- Streaming large result sets
- Variable substitution in batch mode
- Project-level config files (`.tq.toml`)
- Additional schema commands
- Query result caching

**Target:** Enterprise automation and advanced workflows

---

### v2.5 - Security & Integration
**Theme:** Enterprise security and ecosystem integration

**Planned:**
- Keyring integration (OS-native credential storage)
- SSO/SAML authentication support
- Audit logging
- Config validation and security scanning
- Additional export formats (Parquet, HTML)

**Target:** Enterprise security compliance

---

### v3.0 - Intelligence & Collaboration
**Theme:** AI-assisted SQL and team collaboration

**Considered:**
- Query performance analysis (EXPLAIN integration)
- SQL auto-formatting
- Autocorrect suggestions
- Query templates and snippets
- Collaborative features (query sharing)
- Multi-connection management

**Target:** Team productivity and intelligence

---

## Guiding Principles

Throughout all phases, `tq` adheres to these principles:

1. **UNIX Philosophy**: Do one thing well, compose with other tools
2. **Zero Configuration**: Works out of the box for common cases
3. **Fast**: Minimal startup time, efficient execution
4. **Secure**: Credential security is paramount
5. **Reliable**: Predictable behavior, clear error messages
6. **Cross-platform**: Identical experience on Linux, macOS, Windows

---

## Success Metrics

| Metric | v1.0 Target | v2.0 Target | v3.0 Target |
|--------|-------------|-------------|-------------|
| DBA Adoption | 30% | 70% | 90% |
| Daily Active Users | 100 | 500 | 2000 |
| Query Volume | 1K/day | 10K/day | 100K/day |
| Test Coverage | 40% | 80% | 95% |
| Startup Time | <100ms | <50ms | <25ms |
| Bug Report Rate | <5/month | <2/month | <1/month |

---

## Version Strategy

- **v1.x**: Patch releases for bug fixes and minor features
- **v2.0**: Major release for breaking changes and significant features
- **v3.0**: Next-generation features requiring architectural changes

**Semantic Versioning:**
- **Major (v2.0)**: Breaking changes, major features
- **Minor (v1.7)**: New features, no breaking changes
- **Patch (v1.7.1)**: Bug fixes only

---

## Decision Framework

During sprint planning (Phase 0), the sprint coordinator decides:

1. **Feature Sprint**: Implement features from backlog
2. **Maintenance Sprint**: Address technical debt, fix bugs

**Criteria for Maintenance Sprint:**
- >3 P0 bugs reported
- Test pass rate <95%
- User trust crisis
- Framework issues blocking progress

---

## Related Documents

- **[Status Dashboard](status.md)** - Current implementation status
- **[Backlog](backlog.md)** - Prioritized feature backlog
- **[Specifications](../specifications/)** - Detailed feature specifications
- **[Sprint History](../sprints/)** - Historical sprint reviews and retrospectives

---

## Roadmap Updates

This roadmap is reviewed quarterly and updated based on:
- User feedback and feature requests
- Market trends and competitive analysis
- Sprint retrospectives and lessons learned
- Strategic business goals

**Next Review:** 2026-04-01
