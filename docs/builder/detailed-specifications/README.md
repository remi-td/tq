# Detailed Specifications Index

**Last Updated:** 2026-01-18

This directory contains detailed specifications extracted from the comprehensive tq specifications document. Each file is a self-contained specification covering a specific aspect of the tq tool.

---

## Core Design Specifications

### [cli-interface.md](cli-interface.md)
**Command-Line Interface Design**

Comprehensive CLI design including:
- Command structure and global options
- `ping`, `query`, and `repl` commands
- Input/output behavior and terminal detection
- Flag design guidelines
- Help text and version information

**Sections:** 8 major sections | **Size:** ~11 KB

---

### [repl-mode.md](repl-mode.md)
**REPL Mode (Interactive Mode) Specifications**

Complete interactive mode specifications including:
- Sprint 4 Phase 2 foundation features
- User interface and prompt design
- Input handling (multi-line SQL, history, line editing)
- Metacommands (`/describe`, `/ping`, `/session`, etc.)
- Editor modes (Emacs and Vi/Vim keybindings)
- Result display and special features

**Sections:** 9 major sections | **Size:** ~27 KB

---

### [batch-mode.md](batch-mode.md)
**Batch Mode Specifications**

Non-interactive execution mode for scripts and automation:
- Execution modes (inline, file, stdin)
- Output destinations and error handling
- Scripting integration examples
- Performance considerations for large datasets
- Transaction control and variable substitution (future)

**Sections:** 7 major sections | **Size:** ~3.5 KB

---

## Configuration and Data Management

### [configuration.md](configuration.md)
**Configuration and Credential Management**

Configuration system and security:
- Configuration hierarchy (defaults → system → user → project → env → CLI)
- Configuration file format (TOML)
- Environment variables
- Connection profiles
- Credential management (password files, keyring, prompts)
- SSL/TLS configuration (future)

**Sections:** 6 major sections | **Size:** ~5.2 KB

---

### [output-formats.md](output-formats.md)
**Output Format Specifications**

Detailed output format specifications:
- Format selection priority and types
- Table format (ASCII, simple, compact, markdown)
- JSON format (array, JSONL, wrapped)
- CSV format (RFC 4180, Excel-compatible, TSV)
- Type mapping and comparison matrix

**Sections:** 5 major sections | **Size:** ~6.6 KB

---

## Quality and Security

### [error-handling.md](error-handling.md)
**Error Handling and User Feedback**

Comprehensive error handling strategy:
- Error categories (user, connection, auth, query, permission, system)
- Error message structure and best practices
- Progress indicators (spinner, progress bar, multi-progress)
- Warnings and verbose output
- Logging (future)

**Sections:** 6 major sections | **Size:** ~6.4 KB

---

### [security.md](security.md)
**Security Requirements**

Security best practices and requirements:
- Credential security (never log, prevent leaks, file permissions)
- SQL injection prevention
- Connection security (TLS/SSL, timeouts)
- Data privacy (redaction, audit logging)
- Supply chain security
- Security hardening principles

**Sections:** 6 major sections | **Size:** ~3.8 KB

---

### [performance.md](performance.md)
**Performance Considerations**

Performance targets and optimization strategies:
- Startup performance (< 100ms target)
- Query execution and connection pooling
- Memory usage targets and strategies
- Large result set handling (streaming)
- Network performance
- Build optimization
- Performance monitoring

**Sections:** 7 major sections | **Size:** ~3.6 KB

---

## Additional Resources

### [interactive-mode-mvp.md](interactive-mode-mvp.md)
**Interactive Mode MVP (Phase 1)**

Original MVP specifications for interactive mode Phase 1 (completed).

**Size:** ~11 KB

---

### [user-personas.md](user-personas.md)
**User Personas and Use Cases**

Detailed user personas and common use cases.

**Size:** ~11 KB

---

## Document Metadata

| Specification | Version | Owner | Status |
|--------------|---------|-------|--------|
| cli-interface.md | 1.1.0 | cli-ux-designer | Active |
| repl-mode.md | 1.1.0 | cli-ux-designer | Active - Phase 2 (Sprint 4) |
| batch-mode.md | 1.1.0 | cli-ux-designer | Active |
| configuration.md | 1.1.0 | cli-ux-designer | Active |
| output-formats.md | 1.1.0 | cli-ux-designer | Active |
| error-handling.md | 1.1.0 | cli-ux-designer | Active |
| security.md | 1.1.0 | cli-ux-designer | Active |
| performance.md | 1.1.0 | cli-ux-designer | Active |
| interactive-mode-mvp.md | 1.0.0 | cli-ux-designer | Completed |
| user-personas.md | 1.0.0 | cli-ux-designer | Active |

---

## Usage Guidelines

1. **Authority**: These detailed specifications are authoritative for their respective domains
2. **Updates**: Changes must be reviewed and approved by the specification owner
3. **Cross-references**: Specifications may reference each other but should be self-contained
4. **Implementation**: Developers should consult relevant specifications before coding

---

## Master Specifications

For higher-level architectural decisions and overall project direction, see:
- `docs/builder/specifications.md` - Master specifications
- `docs/builder/rust-architecture.md` - Rust architecture for tq
- `docs/builder/rust-cli-design-general.md` - General Rust CLI design guidelines
- `docs/builder/testing-guidelines.md` - Testing methodology and best practices

---
