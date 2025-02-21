# ShadowProxy GUI

**ShadowProxy GUI** is an HTTP interceptor tool designed for security testing of web applications. It provides an intuitive graphical user interface (GUI) to intercept, inspect, and modify HTTP/HTTPS requests and responses, making it easier for security professionals and developers to identify vulnerabilities and test the security of their web applications.

---

## Features

- **HTTP/HTTPS Interception**: Intercept and inspect HTTP/HTTPS traffic between the client and server.
- **Request/Response Modification**: Modify requests and responses in real-time to test edge cases and vulnerabilities.
- **User-Friendly GUI**: Intuitive graphical interface for easy navigation and usage.
- **Security Testing**: Designed specifically for security testing, including testing for vulnerabilities like SQL injection, XSS, CSRF, and more.
- **Custom Rules**: Define custom rules for intercepting and modifying traffic.
- **Cross-Platform**: Works on Windows, macOS, and Linux.

---

## Installation

### Prerequisites
- Rust (if building from source)
- Git (optional, for cloning the repository)

### Steps
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/zohaib2k2/shadowproxy_gui.git
   cd shadowproxy_gui
   cargo run```
2. **Usage**
   ```bash
   ./mitmdump --listen-port 8081 -s interceptor.py
   chromium --proxy-server=127.0.0.1:8081   
   ```
