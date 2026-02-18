# DockStack - DevStack Manager ⚡

![License](https://img.shields.io/badge/license-MIT-blue.svg) ![Rust](https://img.shields.io/badge/rust-v1.75%2B-orange.svg) ![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-lightgrey.svg)

**DockStack** is a modern, native, and lightning-fast local development environment manager. Built entirely in **Rust** using `egui` and `Docker Compose`, it is designed to replace heavy, bloated tools like XAMPP, Laragon, or web-based control panels.

**Philosophy:**
- **Zero Web Tech**: No Electron, no WebView, no Node.js. Just pure, compiled Rust.
- **Privacy First**: Everything runs locally.
- **Resource Efficient**: Uses <20MB RAM when idle.

---

## 🚀 Key Features

### 🛠️ Service Stack / Environment
Manage your favorite services with a single, high-performance interface:
- **Databases**: PostgreSQL (12-16), MySQL (5.7-8.0), Redis (6-7)
- **Web Servers**: Nginx (🌐), Apache (🎯)
- **Runtimes**: PHP-FPM (7.4-8.3)
- **Tools**: phpMyAdmin, pgAdmin, Adminer
- **Security**: Auto-generated local SSL/HTTPS reverse proxy

### 🌟 Advanced Capabilities
- **"Bento Box" UI**: A premium, modern dashboard designed for high information density and cinematic aesthetics.
- **System Wellness Metrics**: Real-time monitoring of CPU, RAM, and Docker container health with beautiful, unified stat cards.
- **Embedded Terminal**: High-performance portable PTY terminal integrated directly into the workspace.
- **Smart Port Checker**: Automatically detects and resolves port conflicts before deployment.
- **Deep Integration**: Native Linux integration with tray support and zero-latency UI interaction.

---

## 📦 Installation

### Pre-built Binaries
Check the [Releases](https://github.com/yourusername/dockstack/releases) page for the latest binaries.

### Building from Source

**Prerequisites:**
1. **Rust Toolchain**: Install via `rustup` (https://rustup.rs).
2. **System Dependencies (Linux only)**:
   ```bash
   sudo apt-get install -y libgtk-3-dev libglib2.0-dev libgdk-pixbuf-2.0-dev libatk1.0-dev libpango1.0-dev libcairo2-dev libssl-dev libxdo-dev pkg-config
   ```
3. **Docker**: Docker Engine or Docker Desktop must be installed and running.

**Build Commands:**
```bash
# Clone the repository
git clone https://github.com/yourusername/dockstack.git
cd dockstack

# Run in development mode
cargo run

# Build optimized release binary
cargo build --release
```

The binary will be located at `target/release/dockstack`.

---

## ⚙️ Configuration

DockStack stores its configuration in a simple TOML file located at:
- **Linux**: `~/.config/dockstack/config.toml`
- **macOS**: `~/Library/Application Support/dockstack/config.toml`
- **Windows**: `%APPDATA%/dockstack/config.toml`

### Example `config.toml`
```toml
active_project_id = "default"

[[projects]]
id = "my-web-app"
name = "My Web App"
directory = "/home/user/projects/webapp"
ssl_enabled = true

[projects.services.postgresql]
enabled = true
port = 5432
version = "16"

[projects.services.nginx]
enabled = true
port = 80
```

---

## 🤝 Contributing

Contributions are welcome! Whether it's reporting a bug, suggesting a feature, or writing code.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

---

## 📜 License

Distributed under the MIT License. See `LICENSE` for more information.

---

**Made with ❤️ in Rust**
