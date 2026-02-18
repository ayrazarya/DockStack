# DockStack - DevStack Manager

DockStack is a high-performance, native desktop application designed for managing local development environments powered by Docker Compose. Developed in Rust using the egui framework, it provides a resource-efficient alternative to traditional web-based management tools.

## Core Philosophy

- **Native Performance**: Compiled Rust binary with minimal memory footprint.
- **Privacy**: Local-first operation with no external data telemetry.
- **Efficiency**: Low CPU and RAM overhead, optimized for development workstations.

---

## Features

### Service Management
Manage infrastructure services through a unified interface:
- **Databases**: PostgreSQL (12-16), MySQL (5.7-8.0), Redis (6-7)
- **Web Servers**: Nginx, Apache
- **Runtimes**: PHP-FPM (7.4-8.3)
- **Administration**: phpMyAdmin, pgAdmin, Adminer
- **Security**: Local SSL/HTTPS reverse proxy generation

### Advanced Capabilities
- **Bento Interface**: A modular dashboard structured for high information density and visual clarity.
- **System Monitoring**: Real-time analysis of CPU, memory, and container metrics.
- **Embedded Terminal**: Integrated portable PTY terminal for direct shell access.
- **Conflict Resolution**: Automated port scanning and conflict detection.
- **System Integration**: Native Linux support with dedicated tray functionality.

---

## Installation

### Binary Distribution
Pre-compiled binaries are available on the Releases page for Linux, Windows, and macOS.

### Building from Source

**Requirements:**
1. **Rust Toolchain**: Current stable version via rustup.
2. **System Dependencies (Linux)**:
   ```bash
   sudo apt-get install -y libgtk-3-dev libglib2.0-dev libgdk-pixbuf-2.0-dev libatk1.0-dev libpango1.0-dev libcairo2-dev libssl-dev libxdo-dev pkg-config
   ```
3. **Docker Engine**: Must be installed and active on the host system.

**Compilation:**
```bash
git clone https://github.com/ayrazarya/DockStack.git
cd dockstack
cargo build --release
```

The optimized binary is located at `target/release/dockstack`.

---

## Configuration

Configuration is managed via a TOML file located at:
- **Linux**: `~/.config/dockstack/config.toml`
- **macOS**: `~/Library/Application Support/dockstack/config.toml`
- **Windows**: `%APPDATA%/dockstack/config.toml`

---

## License

Distributed under the MIT License.
