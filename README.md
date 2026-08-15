# sql_formatter

[![CI](https://github.com/Tinkora/sql_formatter/actions/workflows/test.yml/badge.svg)](https://github.com/Tinkora/sql_formatter/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](./LICENSE)
[![Rust 1.95+](https://img.shields.io/badge/rust-1.95%2B-orange.svg)](https://www.rust-lang.org)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](./CONTRIBUTING.md)

A browser-native SQL formatter and minifier. Support PostgreSQL, MySQL, SQLite, and generic SQL dialects. Format with configurable indent, uppercase/lowercase keywords, comma placement. Minify to single line. Syntax validation. All in WASM — no server needed.

## ✨ Features

- 🎨 **Format SQL** — Beautiful, consistent formatting with configurable indent, keyword case, and comma placement
- 📦 **Minify SQL** — Compress to single line for storage or transmission
- 🔍 **Syntax Validation** — Catch errors and warnings before sending to your database
- 🗄️ **Multi-Dialect** — PostgreSQL, MySQL, SQLite, and generic SQL support
- ⚡ **WASM-Powered** — Everything runs in-browser, instant and private
- 📋 **Copy & Download** — One-click copy or download formatted result

## 🚀 Quick Start

```bash
# Clone
git clone https://github.com/Tinkora/sql_formatter.git
cd sql_formatter

# Build Web WASM
wasm-pack build --target web crates/sql_formatter_web

# Launch
cp crates/sql_formatter_web/pkg/* crates/sql_formatter_web/static/pkg/
cd crates/sql_formatter_web/static && python3 -m http.server 8080
```

Open `http://localhost:8080` in your browser.

## 📂 Project Structure

| Component | Description | Status |
|-----------|-------------|--------|
| `sql_formatter_core` | Tokenizer, formatter, minifier, validator | ✅ |
| `sql_formatter_web` | WASM bridge + HTML editor | ✅ |
| `skills/` | Agent Skill definition (MCP tools) | ✅ |

## 🔧 Development

```bash
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p sql_formatter_web --target wasm32-unknown-unknown
```

## 📄 Docs

- [Product Spec (zh-CN)](docs/product_spec.zh-CN.md)

## 🤝 Community

- [Contributing](./CONTRIBUTING.md)
- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Security](./SECURITY.md)
- [Changelog](./CHANGELOG.md)
- [Support](./SUPPORT.md)

## Support the work

If sql_formatter saves you time, support Tinkora on [Ko-fi](https://ko-fi.com/tinkora).
Support is optional and never affects access or issue priority.

## 📜 License

MIT © [Tinkora](https://github.com/Tinkora)
