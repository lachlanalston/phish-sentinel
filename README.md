# Phish Sentinel

**Phish Sentinel** is a backend-powered email analysis tool that detects phishing attempts using file or header input. The core logic is written in **Rust** for performance and safety. This is the early stage of development focusing on building reliable analysis utilities.

---

## 🚧 Project Status

🔧 **In Development**  
Currently building the backend components in Rust. Future plans include adding a web frontend
---

## 🦀 Built with Rust

Phish Sentinel uses Rust to parse and analyze email headers and files, identifying common signs of phishing such as:

- Suspicious sender domains
- Mismatched `From` and `Reply-To`
- Known phishing patterns
- Malformed headers
- Obfuscated links

---

## 🧪 Usage (Coming Soon)

Right now, the project includes command-line Rust tools. You'll be able to:

- Run analysis on `.eml` or `.txt` files
- Pipe or paste email headers for inspection
- Get structured output (e.g. JSON) indicating risk level

Example usage (Coming Soon):

```bash
cargo run -- analyze headers.txt
