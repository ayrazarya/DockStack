# Code Standardization & Contribution Guide

Welcome to the `DockStack` repository. To ensure this project's codebase remains highly maintainable and readable for everyone in the team as well as open source contributors, we have agreed to follow the standard conventions of the **Rust** ecosystem.

## 1. Code Formatting
All Rust code in this project **must** be formatted using `cargo fmt` so that indentation, line length limits, spacing, and curly brace placement maintain a 100% consistent style from start to finish.
- No trailing whitespaces.
- Do not commit files that have not been formatted with `cargo fmt`.

## 2. Static Analysis (Linter)
In addition to compiling successfully, the correct implementation of memory practices (*Borrowing / Mutability*) must pass the *Clippy Linter*.
Running `cargo clippy --all-targets --all-features -- -D warnings` must succeed with zero warnings.

Best practices to follow:
- Do not `.clone()` heavy data structures (`Arc`, `Vec`, etc.) inside looping functions (especially the UI event loop like Egui's `update()`). Use iteration methods with *lock guard* memory references (*Borrowing*).
- Only pass variables as *mutable* (`&mut self` or `&mut value`) if the method/function genuinely intends to write data. If it is only reading, use an **immutable reference** (`&self`, `&value`).
- Utilize *pattern matching* (`match`) as much as possible when encountering `Result` or rich Enum variants. Avoid reckless use of `.unwrap()`, which can cause a *Panic* (force close) in the GUI if it fails.

## 3. Background Thread I/O Management
DockStack features a 60FPS GUI rendered on the Main Thread. You are **strictly forbidden** from calling OS processes (like `Command::new()`, `reqwest`, `db queries`, or long calculation processes) directly from the GUI thread, as it will cause "Lag" or "Freezing".
Offload these heavy processes to a `thread::spawn` that communicates back to the GUI using the *crossbeam-channel* (Message Passing) system.

## 4. "Zombie Process" Prevention
When spawning third-party extensions (e.g., `docker logs stream` or *Interactive Shell Console PTY*), you **must** include functions to kill or detect exited children (e.g., via `child.wait()`). This ensures our application shuts down cleanly *(Clean Shutdown)* when the GUI is closed, leaving no memory leaks or hanging / *Zombie* processes running in the OS background.

## 5. Naming Conventions
Follow standard Rust conventions:
- **`snake_case`**: for variable names, file/module names, function names, and macros.
- **`UpperCamelCase`**: for Class/Struct names, Traits, and Enum types.
- **`SCREAMING_SNAKE_CASE`**: for constants / static global tokens.

## 6. Logs & Debugging
Do not spam the console using `println!()`. Use the standard logging framework macros: `log::info!`, `log::warn!`, `log::error!`. Use representative error messages with the `{}` format within the pattern string.

```rust
// Do not do this (Unrepresentative, slow, spammy):
println!("Error fetching data");

// Do this instead:
log::error!("Failed to fetch container data from the docker engine: {e}");
```

Thank you for contributing to keeping "DockStack" clean, fast, safe from memory leaks, and up to international standards!
