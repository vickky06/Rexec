
# 🧠 Rexec (Rust + Exec) Engine

A high-performance, Rust-based code execution engine designed for evaluating DSA (Data Structures & Algorithms) problems in multiple languages. It executes user-submitted code in isolated Docker containers and communicates via gRPC with a gateway.

---

## 🚀 Features

- ⚙️ **Multi-language Support**: Python, Go, Java (extensible)
- 🐳 **Containerized Execution**: Docker isolation for security
- 🔗 **gRPC Interface**: High-performance communication with frontend/backend
- 🛡 **Safe & Fast**: Built with Rust for speed and memory safety
- 📦 **WebSockets**: Web Sockets for continious conversation, to validate syntex and code verfication.
- 📦 **Pluggable Queues (Upcoming)**: Kafka/RabbitMQ for async execution

---

## 🏗 Architecture

```plaintext
Frontend  -> Code Validation --> Validation using Tree Library. <Non Docker> 
   ↓
Request for Code <submission>
   ↓
DSA Engine (Rust)
   ↓
Docker Runner (per language)
