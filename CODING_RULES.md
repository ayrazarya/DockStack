# DockStack Coding Rules & Technical Guidelines

This document outlines the specific coding rules, patterns, and architectural constraints for developing **DockStack**. While `CONTRIBUTING.md` provides general guidelines, this document serves as the absolute technical rulebook for writing code in this repository.

## 1. UI & State Management (Egui)
- **Immediate Mode Paradigm**: Remember that `eframe/egui` is an immediate mode GUI. The `update()` function can run at up to 60 frames per second.
- **Zero Allocations in `update()`**: Do not perform heavy allocations, cloning of large arrays (`Vec`, `VecDeque`), or complex calculations directly inside the UI `render` or `update` loops.
- **State Separation**: Keep stable application state (`AppConfig`, `DockerManager`, `SystemStats`) strictly decoupled from UI transient states.
- **Locking Strategy**: When accessing shared state (`Arc<Mutex<T>>`), acquire the lock, extract or mutate the data as quickly as possible, and drop the lock guard immediately. Never hold a mutex lock while performing I/O operations.

## 2. Concurrency & Multithreading
- **No Blocking the Main Thread**: All Docker CLI invocations, file system writes (except simple config saves), and network requests must be spawned in a separate background thread (`std::thread::spawn`).
- **Message Passing**: Use `crossbeam-channel` (`Sender` / `Receiver`) to communicate data changes and state updates from background worker threads back to the main GUI thread.
- **Graceful Thread Shutdown**: Infinite loop background threads (like log streamers or resource monitors) must regularly check a shared boolean flag (e.g., `Arc<Mutex<bool>>`) so they know when to safely terminate.

## 3. Error Handling
- **No `unwrap()` or `expect()`**: Using panic-inducing methods is strictly prohibited in business logic and UI rendering.
  - Use the `let else` syntax (`let Some(val) = option else { return; }`) for handling empty Options cleanly.
  - Use the `?` operator for propagating `Result` errors gracefully.
  - Pattern matching (`match`) is preferred for handling complex variants.
- **User-Facing Errors**: Use `log::error!` for logging technical details to the terminal, but also emit an associated `DockerEvent::Error` so the user is informed of the failure via the graphical interface.

## 4. Docker Command Integration
- **CLI Abstraction**: All `std::process::Command` calls related to Docker or Docker Compose must reside exclusively in `src/docker/manager.rs` or `src/docker/compose.rs`. Do not scatter subprocess logic across the UI components.
- **Subprocess Management**: Always call `.wait().ok()` on long-running child processes (e.g., live streaming logs) to reap the process and prevent OS zombie processes.
- **Deterministic Parsing**: Never assume the text output of the Docker CLI will always be identical across versions. Whenever dealing with lists, force Docker to output in structured templates using explicit formatting flags like `--format '{{.ID}}|{{.Names}}|{{.State}}'`.

## 5. Security & Safety
- **Anti-Command Injection**: Never directly construct or pass unsanitized user inputs into a continuous raw string passed to a shell. Always use explicit `.arg("param")` arrays on `Command::new()`.
- **Filesystem Boundaries**: When generating configuration files (`docker-compose.yml`, `nginx.conf`), ensure path operations do not perform directory traversal. Rely on specific absolute paths generated from the `ProjectConfig.directory`.

## 6. Rust Idioms
- **Ownership over Cloning**: Prefer passing references (`&T`) to methods. Only use `.clone()` when absolutely necessary to satisfy lifetime constraints across thread boundaries (e.g., moving an `Arc` pointer into a closure).
- **Data Structures**: If a data structure requires intense push and pop operations from both ends (like a Chat UI or a sliding window Log terminal), use `VecDeque` instead of `Vec` to achieve O(1) performance and prevent Memory CPU bottlenecks.
- **Dead Code Warning Check**: Standard implementations must avoid generating `dead_code` warnings. Ensure all helper methods, properties, and trait implementations are actually utilized, or actively remove them to keep the codebase clean.
