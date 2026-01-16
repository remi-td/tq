---
name: rust-coder
description: Guides Claude in writing idiomatic, efficient, well-structured Rust code using proper data modeling, traits, impl organization, macros, and build-speed best practices based on official Rust style guide principles.
---

# Rust Coder

## Core Design Philosophy

**Purpose**: Rust's style conventions reduce cognitive burden, facilitate team communication, and improve code comprehension through pattern matching. Consistent formatting allows developers to focus on substance rather than style debates.

**Guiding Principles** (in priority order):
1. **Readability** - Code must be scan-able, avoid misleading formatting, work in plain-text contexts (diffs, grep, error messages), and support accessibility tools
2. **Aesthetics** - Maintain visual harmony and consistency with broader programming conventions
3. **Specifics** - Optimize for version control (clean diffs), prevent rightward drift, minimize vertical space
4. **Application** - Rules should be manually applicable, tool-friendly (rustfmt), internally consistent, and simple

## Instructions

### 1. Fully understand the user request
Determine whether the task involves designing data structures, implementing traits, writing macros, modeling domain logic, or organizing modules.
Identify key constraints such as mutability needs, ownership flow, async context, interior mutability, or concurrency boundaries.

### 2. Plan data structures with precision
- Choose between `struct`, `enum`, or `newtype` based on domain needs
- Consider ownership of each field:
  - Use `&str` vs `String`, slices vs vectors, `Arc<T>` when sharing, or `Cow<'a, T>` for flexible ownership
- Model invariants explicitly using types (e.g., `NonZeroU32`, `Duration`, custom enums)
- **Prefer `enum` for state machines** instead of boolean flags or loosely related fields
- Use **expression-oriented programming**: prefer returning values from conditional expressions rather than statement-based approaches with separate assignments

### 3. Follow official naming conventions
- **`UpperCamelCase`** - Types, traits, enum variants
- **`snake_case`** - Functions, methods, fields, local variables, modules, macros
- **`SCREAMING_SNAKE_CASE`** - Constants and immutable statics
- When conflicting with keywords, use raw identifiers (`r#crate`) or append underscore (`crate_`)
- Avoid intentional misspellings

### 4. Organize modules and items correctly
**Item ordering at module level**:
1. `extern crate` statements (alphabetically sorted)
2. `use` statements and module declarations (version-sorted; `self`/`super` first, globs last)
3. Other items (functions, structs, enums, traits, impls)

**Module structure best practices**:
- Organize code into modules reflecting ownership and domain boundaries
- Use `pub(crate)` instead of `pub` when possible; expose only what needs exposing
- Minimize use of `#[path]` annotations; prefer default filesystem-based resolution
- Keep APIs small and expressive; avoid leaking internal types
- Use meaningful file and module names aligned with functionality

### 5. Write idiomatic Rust implementations
- Place `impl` blocks immediately below the struct/enum they modify
- Group related methods: constructors, getters, mutation methods, domain logic, helpers
- Provide clear constructors (`new`, `with_capacity`, builders) where appropriate
- Use trait implementations (`Display`, `Debug`, `From`, `Into`, `TryFrom`) to simplify conversions
- **Prefer returning `Result<T, E>` instead of panicking**
- Keep functions short to help lifetime inference and clarity
- Use four-space indentation (never tabs)
- Target 100-character line limit
- Use block indentation over visual alignment
- Add trailing commas in multi-line constructs

### 6. Format code elements properly

**Blocks**:
- Empty blocks as `{}`
- Single-line blocks only in expression position: `{ expr }`
- Newlines after `{` and before `}` for multi-line blocks
- Keywords (`unsafe`, `async`) on same line as opening brace

**Function signatures**:
- Format: `[pub] [unsafe] [extern ["ABI"]] fn name(args) -> ReturnType { body }`
- Multi-line: break after `(`, block-indent each argument, trailing comma before `)`

**Structs**:
- Opening brace on same line as struct name
- Each field block-indented with trailing comma
- If field type exceeds margin, break to indented line

**Enums**:
- Each variant on own line, block-indented
- Small struct variants may be single-line with spaces around braces
- If any variant is multi-line, format all struct variants as multi-line

**Traits & Impls**:
- Empty on one line: `trait Foo {}`
- Otherwise break after opening brace
- Prefer inline trait bounds; if breaking, each bound on own line before `+`

**Closures**:
- No space before first `|` (unless preceded by `move`)
- Space between `||` and expression body
- Omit braces when possible; add for return types, statements, comments, or multi-line control flow

**Method chains**:
- Single line if small
- Multi-line: each element on own line with break before `.`, block-indent subsequent lines

### 7. Apply rigorous documentation and code-style best practices
- Use triple-slash (`///`) doc comments for public structs, enums, fields, and methods
- Use inner doc comments (`//!`) for module-level documentation explaining design or architecture
- Include examples in docs where valuable, especially for public APIs
- Run `cargo fmt` and `cargo clippy --all-targets --all-features` to maintain consistency
- Reserve blank lines between logically separate methods and sections

### 8. Use macros effectively but responsibly
- Apply `derive` macros (`Debug`, `Clone`, `Serialize`, `Deserialize`, etc.) to reduce boilerplate
- Create small, focused declarative macros to eliminate repetitive patterns
- For procedural macros, enforce clear boundaries and predictable generated code
- Document macro behavior and provide usage examples

### 9. Optimize build speed when relevant
- On Linux, configure `.cargo/config.toml` to use the `mold` linker when appropriate
- Use `sccache` to cache compiled artifacts during development
- Minimize unnecessary dependencies and feature flags
- Prefer `cargo check` during rapid iteration over `cargo build`
- Split crates into lightweight workspaces to avoid monolithic rebuilds
- Use `cargo profile` settings for tuned dev/release defaults

### 10. Provide explanations and alternatives
For every code design, explain why a certain pattern is chosen and propose alternatives when relevant:
- Builder pattern vs simple constructor
- Enum-based state machine vs multiple booleans
- Shared ownership via `Arc<T>` vs message passing channels
- Slice-based APIs for performance vs owned collections for convenience
- Deriving traits vs manual implementations for custom logic
- Expression-oriented vs statement-based approaches

### 11. Maintain clarity, safety, and idiomatic style at all times
Prioritize predictable ownership flow, correct lifetimes, and ergonomic APIs that reflect common Rust patterns. Code should be readable in plain-text contexts without syntax highlighting, produce clean diffs for version control, and follow the principle that "humans comprehend information through pattern matching."