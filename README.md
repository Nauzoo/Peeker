# 📦 peeker

![Rust](https://img.shields.io/badge/rust-stable-orange)
![Status](https://img.shields.io/badge/status-in%20development-yellow)
![License](https://img.shields.io/badge/license-GPLv3.0-blue)

> A secure, lightweight cloud file manager for home servers — built in Rust.

---

## 🚀 Overview

**peeker** is a self-hosted cloud file manager designed for simplicity, performance, and security.  
It provides authenticated access to files over HTTP, making it ideal for personal servers and private infrastructure.

The project emphasizes:

- 🔐 Secure authentication & session handling  
- ⚡ High performance with Rust  
- 🧩 Modular and extensible architecture  
- 🏠 First-class support for home-server environments  

---

## ✨ Current Features

- [X] User authentication (login endpoint)
- [X] Access sanitization for secure file handling
- [X] JSON Web Token (JWT) based session validation

---

## 🗺️ Roadmap

- [ ] SQLite integration (persistent storage)
- [ ] Device whitelist (authorized devices only)
- [ ] Secure tunneling for private connections
- [ ] Web-based frontend interface

---

## 📦 Installation

### Prerequisites

- Rust toolchain installed (`cargo`)  
  👉 https://github.com/rust-lang/cargo

### Build & Run

```bash
git clone https://github.com/your-username/peeker.git
cd peeker
cargo run
```

Server will start at:

```
http://localhost:3000
```

---

## 🧪 API Usage

Make sure the server is running before making requests.

### 🔑 Authenticate

```bash
curl -X POST http://localhost:3000/login \
     -H "Content-Type: application/json" \
     -d '{
           "username": "nauzoo",
           "password": "senha_super_segura",
           "device": "my_device"
         }'
```

Response will include a JWT token.

---

### 📂 Fetch a File

```bash
curl -i -H "Authorization: Bearer <TOKEN>" \
http://localhost:3000/files/<file_name.extension>
```

---

## 🧱 Architecture (High-Level)

```
Client → HTTP API → Auth Layer (JWT)
                      ↓
                 File Access Layer
                      ↓
                  File System
```

Future versions will introduce:

- Database layer (SQLite)
- Device validation middleware
- Secure networking layer (tunneling)

---

## 🔐 Security Notes

- All file access is sanitized to prevent path traversal
- Authentication is required for protected routes
- JWT tokens are used for stateless session validation

---

## 🤝 Contributing

Contributions are welcome!

```bash
# Fork the repo
# Create your feature branch
git checkout -b feature/my-feature

# Commit your changes
git commit -m "feat: add new feature"

# Push to your fork
git push origin feature/my-feature
```

Then open a Pull Request 🚀

---

## 📄 License

This project is licensed under the [GPL 3.0](LICENSE) License.
