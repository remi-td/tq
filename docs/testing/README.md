# Testing Documentation

This directory contains the testing methodology, approach, and guidelines for the tq project.

## Purpose

While `docs/specifications/` defines **WHAT** features should do and `docs/design/` explains **HOW** they're implemented, `docs/testing/` describes **HOW WE VALIDATE** that implementations meet specifications.

## Structure

| Document | Purpose |
|----------|---------|
| `philosophy.md` | Core testing principles and quality philosophy |
| `approach.md` | Testing strategy, test types, design patterns |
| `execution.md` | Running tests, best practices, debugging |
| `tools.md` | Testing infrastructure, tools, and utilities |

## Relationship to Other Documentation

```
docs/
├── specifications/   # WHAT features should do
├── design/          # HOW features are implemented
├── testing/         # HOW we validate implementation
│   ├── philosophy.md
│   ├── approach.md
│   ├── execution.md
│   └── tools.md
├── sprints/         # Historical record of deliveries
└── roadmap/         # Status and future plans

tests/               # Actual test implementation and results
├── cases/           # Test case definitions
├── interactive_tests.rs  # Interactive REPL tests
├── strategy/        # Per-sprint test strategies
├── results/         # Test execution results
└── tools/          # Test utilities and helpers
```

## Key Principles

### Test What Users See

Tests must validate user experience, not just implementation mechanics. A feature that technically works but delivers wrong output has failed.

### The Testing Contract

> "If a feature is specified, it has a test. If a test exists, it passes. If it passes, the spec is accurate."

This ensures no untested features ship and test failures indicate real problems.

### Coverage Philosophy

tq uses two distinct coverage metrics:
- **Automated Coverage** (~40%): Unit tests measured by cargo-tarpaulin
- **Total Coverage** (~85%): Includes interactive REPL tests

The seemingly "low" automated coverage is expected because REPL features require interactive testing with live databases.

## For Test Designers

When creating tests:
1. Read `philosophy.md` to understand testing principles
2. Review `approach.md` for test design patterns
3. Consult `tools.md` for available testing utilities
4. Follow execution guidelines in `execution.md`

## For Developers

When implementing features:
1. Read specifications to understand requirements
2. Read design docs to understand implementation approach
3. Read testing docs to understand validation approach
4. Write code that can be effectively tested

## For Sprint Coordinators

When planning sprints:
1. Ensure test strategy aligns with feature requirements
2. Verify test coverage for all deliverables
3. Allocate time for both automated and interactive testing
4. Document test approach in `tests/strategy/`

## Testing vs Sprint Execution

**Important distinction**:
- **Testing documentation** (`docs/testing/`) - Timeless methodology
- **Sprint execution** (`tests/strategy/`, `tests/results/`) - Per-sprint implementation

Testing docs explain HOW to test. Sprint artifacts track WHAT was tested and WHEN.

## Quick Reference

### Running All Tests
```bash
cargo test
```

### Running Unit Tests Only
```bash
cargo test --lib
```

### Running Interactive Tests
```bash
cargo test --test interactive_tests -- --ignored --test-threads=1
```

### Measuring Coverage
```bash
cargo tarpaulin --out Html --output-dir coverage
```

See `execution.md` for comprehensive testing commands and options.
