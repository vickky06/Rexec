# 🧠 Rexec (Rust + Exec) Engine

A high-performance, Rust-based code execution engine designed for evaluating DSA (Data Structures & Algorithms) problems in multiple languages. It executes user-submitted code in isolated Docker containers and communicates via gRPC with a gateway.

---

## 🚀 Features

- ⚙️ **Multi-language Support**: Python, Go, Java (extensible)
- 🐳 **Containerized Execution**: Docker isolation for security
- 🔗 **gRPC Interface**: High-performance communication with frontend/backend
- 🛡 **Safe & Fast**: Built with Rust for speed and memory safety
- 📦 **WebSockets**: Continuous conversation for syntax and code validation
- 📦 **Pluggable Queues (Upcoming)**: Kafka/RabbitMQ for async execution

---

## 🏗 Architecture

```plaintext
Frontend
   ↓
Gateway (Node.js + gRPC)
   ↓
DSA Engine (Rust)
   ↓
Docker Runner (per language)
```

---

## 🛠️ Prerequisites

- Rust toolchain (https://rustup.rs)
- Docker
- Make
- Bash (for shell scripts)
- grpcui (for gRPC UI, optional)

---

## 🗂️ Directory Structure

- `src/` - Rust source code
- `shell_scripts/` - Shell scripts for port management and configuration
- `Makefile` - Build and utility commands

---

## 🧰 Makefile Commands

| Command            | Description                                                                 |
|--------------------|-----------------------------------------------------------------------------|
| `make build`       | Build the Rust project                                                      |
| `make run`         | Run the service (uses dynamic port config from `shell_scripts/get_config.sh`)|
| `make clean`       | Clean build artifacts and show port info                                    |
| `make fmt`         | Format Rust code using `cargo fmt`                                          |
| `make grpcui`      | Start the gRPC UI for service inspection                                    |
| `make all`         | Format, build, and test the project                                         |
| `make kill_ports`  | Kill processes on configured ports (via `shell_scripts/kill_ports.sh`)      |

---

## ▶️ Example Usage

```sh
# Build the project
make build

# Run the service
make run

# Format code
make fmt

# Start gRPC UI
make grpcui

# Clean build artifacts
make clean

# Kill processes on service ports
make kill_ports
```

---

## 📦 Configuration

Ports and service configuration are managed via shell scripts in `shell_scripts/`.  
The Makefile dynamically fetches and uses these values for running