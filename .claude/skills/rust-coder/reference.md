# Rust Coding Skill Reference

## Official Style Guide Principles

**Priority Order**:
1. **Readability** - Scan-ability, avoiding misleading formatting, accessibility, plain-text context support
2. **Aesthetics** - Visual harmony and consistency
3. **Specifics** - Version control compatibility, preventing rightward drift, minimizing vertical space
4. **Application** - Manual ease of use, tooling support, internal consistency, simplicity

## Naming Conventions

| Element | Convention | Examples |
|---------|-----------|----------|
| Types, Traits, Enum Variants | `UpperCamelCase` | `Rectangle`, `Display`, `Some` |
| Functions, Methods, Variables | `snake_case` | `calculate_area`, `user_name` |
| Modules | `snake_case` | `http_server`, `parse_utils` |
| Constants, Static Variables | `SCREAMING_SNAKE_CASE` | `MAX_SIZE`, `DEFAULT_PORT` |
| Macros | `snake_case!` | `vec!`, `println!` |
| Lifetime Parameters | `'lowercase` | `'a`, `'static` |

**Keyword conflicts**: Use raw identifiers (`r#type`) or append underscore (`type_`), but avoid misspellings.

## Module-Level Item Ordering

Items at module scope must appear in this order:
1. `extern crate` statements (alphabetically sorted)
2. `use` statements (version-sorted, `self`/`super` first, globs last)
3. Module declarations (`mod`)
4. Other items (functions, structs, enums, traits, impls, constants, statics)

**Note**: Don't automatically move `#[macro_use]` annotated module declarations, as this may change semantics.

## Formatting Basics

- **Indentation**: Four spaces (never tabs)
- **Line length**: Target 100 characters
- **Alignment**: Use block indentation, not visual alignment
- **Trailing commas**: Required in multi-line constructs (except in match arms with blocks)
- **Blank lines**: Single blank lines between top-level items; use sparingly within items

## Structs

```rust
// Single-line (if it fits)
struct Point { x: f64, y: f64 }

// Multi-line
struct Rectangle {
    /// Width of the rectangle in pixels
    width: u32,
    /// Height of the rectangle in pixels
    height: u32,
}

// Tuple struct
struct Color(u8, u8, u8);

// Unit struct
struct Marker;
```

**Rules**:
- Opening brace on same line as struct name
- Each field block-indented with trailing comma
- If field type exceeds margin, break to indented line
- Add `///` doc comments for public types and fields

## Enums

```rust
// Single-line variants
enum Direction {
    North,
    South,
    East,
    West,
}

// With data
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(u8, u8, u8),
}

// Small struct variants (single-line if small)
enum SmallData {
    Pair { a: i32, b: i32 },
}

// Multi-line struct variants (if any is multi-line, format all as multi-line)
enum LargeData {
    Complex {
        field_one: String,
        field_two: Vec<u8>,
        field_three: Option<i32>,
    },
}
```

**Rules**:
- Each variant on own line, block-indented
- Small struct variants may be single-line with spaces around braces
- If any struct variant is multi-line, all struct variants use multi-line formatting

## Impl Blocks

```rust
impl Rectangle {
    /// Creates a new Rectangle with the given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Returns the area of the rectangle
    pub fn area(&self) -> u32 {
        self.width * self.height
    }
}

// Empty trait impl
impl Default for Point {}

// Trait impl
impl Display for Rectangle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.width, self.height)
    }
}
```

**Rules**:
- Place immediately below struct/enum definition
- Empty impls on one line: `impl Foo {}`
- Group methods logically: constructors, getters, mutations, domain logic
- Blank lines between methods

## Functions

```rust
// Simple function
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Multi-line signature
fn long_function_name(
    first_parameter: String,
    second_parameter: Vec<u8>,
    third_parameter: Option<i32>,
) -> Result<ProcessedData, Error> {
    // Implementation
}

// Generic function
fn process<T: Display + Clone>(value: T) -> String {
    format!("{}", value)
}

// With where clause (if complex bounds)
fn complex_function<T, U>(t: T, u: U) -> Result<Output, Error>
where
    T: Display + Clone + Send,
    U: Debug + Sync,
{
    // Implementation
}
```

**Rules**:
- Format: `[pub] [unsafe] [extern ["ABI"]] fn name(args) -> ReturnType { body }`
- Multi-line: break after `(`, block-indent arguments, trailing comma before `)`
- Prefer inline trait bounds; use `where` clause only for complex bounds

## Traits

```rust
// Simple trait
trait Drawable {
    fn draw(&self);
}

// With associated types
trait Iterator {
    type Item;

    fn next(&mut self) -> Option<Self::Item>;
}

// With default implementations
trait Greet {
    fn greet(&self) -> String {
        String::from("Hello!")
    }
}

// Trait bounds (inline)
fn print_all<T: Display + Debug>(items: Vec<T>) { }

// Trait bounds (where clause for complex bounds)
fn process<T, U>(t: T, u: U)
where
    T: Display + Clone,
    U: Debug + Default,
{
    // Implementation
}
```

## Blocks and Control Flow

```rust
// Empty block
let x = {};

// Single-line block (expression position only)
let y = { 42 };

// If-else
if condition {
    do_something();
} else if other_condition {
    do_other_thing();
} else {
    default_action();
}

// Single-line if in expression context (if small)
let value = if flag { 1 } else { 0 };

// Match
match value {
    Some(x) => process(x),
    None => handle_none(),
}

// Multi-line match
match complex_value {
    Pattern::Variant { field1, field2 } => {
        process(field1);
        process(field2);
    }
    Pattern::Other(x) if x > 10 => special_handling(x),
    _ => default_case(),
}

// For loop
for item in collection {
    process(item);
}

// While loop
while condition {
    do_work();
}
```

**Rules**:
- Newlines after `{` and before `}` unless single-line in expression position
- Keywords (`unsafe`, `async`) on same line as opening brace
- Opening brace on new line if control line breaks
- Avoid extraneous parentheses unless clarifying precedence

## Closures

```rust
// Simple closure
let add = |a, b| a + b;

// With move keyword
let owned = move |x| process(x);

// Multi-line with braces
let complex = |input| {
    let processed = transform(input);
    validate(processed)
};

// With return type
let typed = |x: i32| -> i32 { x * 2 };
```

**Rules**:
- No space before first `|` (unless preceded by `move`)
- Space between `||` and expression body
- Omit braces when possible
- Add braces for: return types, statements, comments, multi-line control flow

## Method Chains

```rust
// Single-line if small
let result = data.iter().map(|x| x * 2).collect();

// Multi-line
let result = data
    .iter()
    .filter(|x| x.is_valid())
    .map(|x| x.transform())
    .collect::<Vec<_>>();
```

**Rules**:
- Single line if small
- Multi-line: each element on own line, break before `.`, block-indent subsequent lines

## Macros

```rust
// Declarative macro
macro_rules! vec_of_strings {
    ($($x:expr),*) => {
        vec![$($x.to_string()),*]
    };
}

// Derive macros
#[derive(Debug, Clone, PartialEq, Eq)]
struct Data {
    value: String,
}

// Custom derive with attributes
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    user_name: String,
}
```

## Error Handling

```rust
// Result return
fn parse_number(s: &str) -> Result<i32, ParseIntError> {
    s.parse()
}

// Using ? operator
fn process_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

// Custom error types (use thiserror)
#[derive(Error, Debug)]
enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Parse error: {0}")]
    Parse(String),
}
```

**Rules**:
- Prefer `Result<T, E>` over panicking
- Use `?` operator for error propagation
- Use `thiserror` for library errors, `anyhow` for application errors
- Provide context with error messages

## Build Performance

### Linker Optimization (Linux)
```toml
# .cargo/config.toml
[target.'cfg(target_os = "linux")']
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

### Compilation Caching
```bash
# Install sccache
cargo install --locked sccache

# Set environment variable
export RUSTC_WRAPPER=sccache
```

### Development Workflow
- Use `cargo check` for rapid iteration (faster than `cargo build`)
- Use `cargo clippy` for linting
- Use `cargo fmt` for formatting
- Split large projects into workspaces to avoid monolithic rebuilds

### Profile Optimization
```toml
# Cargo.toml
[profile.dev]
opt-level = 0
debug = true

[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
```

## Documentation

```rust
/// A rectangle defined by width and height.
///
/// # Examples
///
/// ```
/// let rect = Rectangle::new(10, 20);
/// assert_eq!(rect.area(), 200);
/// ```
pub struct Rectangle {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

//! This module provides geometric primitives.
//!
//! It includes shapes like rectangles, circles, and triangles.
```

**Rules**:
- Use `///` for item documentation
- Use `//!` for module-level documentation
- Include examples in doc comments for public APIs
- Use markdown formatting in doc comments