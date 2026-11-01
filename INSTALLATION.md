# RustProxy Installation & Build Guide

This document provides verified, step-by-step installation and build instructions for **RustProxy v1.0.0**.

---

## Supported Operating Systems

| Operating System | Support Level | Notes |
| :--- | :--- | :--- |
| **Arch Linux / Manjaro** | **Tier 1 (Fully Verified)** | Primary tested development and target platform |
| **Debian / Ubuntu 22.04+** | **Tier 2 (Supported via Source)** | Requires standard `build-essential` and `libwebkit2gtk-4.1-dev` |
| **Fedora 39+** | **Tier 2 (Supported via Source)** | Requires `webkit2gtk4.1-devel` and development tools |

---

## 1. Quickstart: Pre-built Binary Execution

If you already have a compiled release binary (`rustproxy`):

1. **Verify Binary Location**:
   ```bash
   ./target/release/rustproxy --help
   ```

2. **Run RustProxy Directly**:
   ```bash
   ./target/release/rustproxy
   ```

The application will start immediately:
- **Proxy Listener**: `http://127.0.0.1:8080`
- **Web / Desktop GUI**: `http://127.0.0.1:8081`

Open `http://127.0.0.1:8081` in your browser to access the complete graphical interface. No Node.js or development toolchain is required when running the standalone binary.

---

## 2. Building from Source on Arch Linux

### Step 2.1: Install System Dependencies

Open a terminal and install the essential build tools, OpenSSL, and WebKitGTK (for Tauri desktop windowing):

```bash
sudo pacman -Syu --needed \
    base-devel \
    curl \
    git \
    pkgconf \
    openssl \
    sqlite \
    webkit2gtk-4.1 \
    libsoup3 \
    gtk3
```

### Step 2.2: Install Rust Toolchain

RustProxy requires Rust 1.80 or later (Rust 2021 edition):

```bash
# Install rustup if not already present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Configure environment in current shell
source "$HOME/.cargo/env"

# Verify toolchain
rustc --version
cargo --version
```

### Step 2.3: Install Node.js & npm (Frontend Tooling)

Node.js v20+ and npm are required to build the React 19 desktop interface:

```bash
sudo pacman -S --needed nodejs npm

# Verify versions
node --version
npm --version
```

### Step 2.4: Clone the Repository

```bash
git clone https://github.com/balasaravanakumar7890/Rustproxy.git
cd Rustproxy
```

### Step 2.5: Build the Desktop Frontend Bundle

Install frontend dependencies and compile the production bundle:

```bash
cd desktop
npm ci
npm run build
cd ..
```

This compiles the React 19 and Tailwind CSS frontend into static production assets under `desktop/dist/`.

### Step 2.6: Compile the Rust Workspace

Compile the optimized release binary with Cargo:

```bash
source "$HOME/.cargo/env"
cargo build --release
```

The compiled binary will be placed at:
```bash
target/release/rustproxy
```

---

## 3. Running RustProxy

Launch the application:

```bash
./target/release/rustproxy
```

Console output will display:
```text
  ____           _   ____                    
 |  _ \ _   _ ___| |_|  _ \ _ __ _____  _   _ 
 | |_) | | | / __| __| |_) | '__/ _ \ \/ / | | |
 |  _ <| |_| \__ \ |_|  __/| | | (_) >  <| |_| |
 |_| \_\\__,_|___/\__|_|   |_|  \___/_/\_\\__, |
                                          |___/ 
  RustProxy v1.0.0 - Defensive Traffic Analysis Platform
  Built with Rust, Tokio, Rustls, and Hyper

  [+] Generated Root CA Certificate: /home/user/.rustproxy/rustproxy_ca.crt
  [+] SQLite Flow Database: rustproxy.db
  [*] Proxy listening on http://127.0.0.1:8080
  [*] Desktop/Web API listening on http://127.0.0.1:8081
```

Access the UI:
- Open your browser to **`http://127.0.0.1:8081`**
- Or launch via Tauri desktop runner:
  ```bash
  cd desktop && npm run tauri dev
  ```

---

## 4. Troubleshooting & Common Build Issues

### Issue 1: `cargo: command not found`
- **Cause**: Cargo's binary path is not in the active `$PATH` variable.
- **Solution**: Execute `source "$HOME/.cargo/env"` or add `export PATH="$HOME/.cargo/bin:$PATH"` to your `~/.bashrc` or `~/.zshrc`.

### Issue 2: `Package webkit2gtk-4.1 was not found in the pkg-config search path`
- **Cause**: Missing WebKitGTK development libraries required for native desktop windowing.
- **Solution (Arch Linux)**: `sudo pacman -S webkit2gtk-4.1 libsoup3 gtk3`
- **Solution (Ubuntu/Debian)**: `sudo apt install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev`
- **Alternative**: RustProxy includes an embedded web server that serves the identical full-featured UI over `http://127.0.0.1:8081` without requiring native WebKitGTK compilation.

### Issue 3: `Error reading SQLite database file`
- **Cause**: Existing database locks or stale `.db-shm` / `.db-wal` files after abrupt shutdown.
- **Solution**: RustProxy operates in SQLite WAL mode. Ensure you have write permissions to the working directory or remove stale `rustproxy.db-shm` files.

### Issue 4: Port `8080` or `8081` already in use
- **Cause**: Another service (or an earlier instance of RustProxy) is holding the port.
- **Solution**: Check holding processes:
  ```bash
  ss -tulpn | grep -E '8080|8081'
  ```
  Terminate the existing process or adjust the bind address.
