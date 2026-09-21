# Changelog

## [0.1.0] - 2026-08-06

### Added
- `sql_formatter_core`: SQL tokenizer, formatter, minifier, validator
- `sql_formatter_web`: WASM bridge, HTML editor with syntax highlighting
- Agent Skill definition (`skills/`)
- Landing page (`index.html`)
- CI workflow (native test, clippy, WASM check, wasm-pack build)
- Documentation: product spec

### Features
- Format SQL with configurable indent (2/4/8 spaces)
- Keyword case: uppercase, lowercase, capitalize
- Comma placement: leading or trailing
- Configurable blank lines between statements
- Max line width for wrapping
- Multi-dialect support: Generic, PostgreSQL, MySQL, SQLite
- Minify to single line
- Syntax validation with error/warning reporting
- Copy and Download buttons
