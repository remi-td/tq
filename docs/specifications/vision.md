# tq (Teradata Query) - Project Vision

**Document Type:** Vision and Principles
**Last Updated:** 2026-01-23

---

## Overview

`tq` is a best-in-class, lightweight command-line client for Teradata databases, designed to be fast, intuitive, and composable. It follows UNIX philosophy while providing a rich interactive experience comparable to `psql` for PostgreSQL.

## Goals

- **Simplicity**: Zero-configuration for basic use cases
- **Composability**: Works seamlessly in scripts and pipelines
- **Performance**: Fast startup, efficient query execution, minimal memory footprint
- **Security**: Secure credential handling, no password leaks
- **Cross-platform**: Works identically on Linux, macOS, and Windows
- **Self-contained**: Single static binary with no runtime dependencies

## Design Principles

1. **Convention over Configuration**: Sensible defaults for 80% of use cases
2. **Progressive Disclosure**: Simple things easy, complex things possible
3. **Fail Fast**: Clear error messages with actionable suggestions
4. **Respect UNIX Conventions**: `-h/--help`, `-V/--version`, stdin/stdout, exit codes
5. **Terminal Context Awareness**: Human output for TTY, machine output for pipes

## Architecture

- **Execution Model**: One-shot execution (connect → query → disconnect)
- **Language**: Rust (for performance and safety)
- **Driver**: Teradata teradatarustapi native driver
- **Authentication**: TD2, LDAP, Kerberos (KRB5), TDNEGO

## Target Users

- **Database Administrators**: Quick health checks and diagnostics
- **Data Analysts**: Interactive exploration and reporting
- **DevOps Engineers**: Automated monitoring and CI/CD integration
- **Data Engineers**: Pipeline integration and data extraction

See [User Personas](user-personas.md) for detailed user profiles and use cases.

## Related Documentation

- **Specifications**: Pure feature requirements (this directory)
- **Roadmap**: Implementation status and backlog (`../roadmap/`)
- **Sprint History**: Historical planning and reviews (`../sprints/`)
