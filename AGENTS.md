# Repository Guide for AI Agents

## Project Overview

sql_formatter is a browser-native SQL formatter and minifier. It tokenizes SQL, applies formatting rules with configurable options (dialect, indent, keyword case, comma placement), and outputs beautifully formatted SQL — all in WASM with no server dependency.

## Architecture

```
sql_formatter/
├── crates/
│   ├── sql_formatter_core/       # Tokenizer, formatter, minifier, validator
│   └── sql_formatter_web/        # WASM bridge + HTML editor
├── docs/                          # Specifications
├── skills/                        # Agent Skill definitions (MCP tools)
└── index.html                     # Product landing page
```

## Key Files for AI Context

| File | Purpose |
|------|---------|
| `crates/sql_formatter_core/src/format.rs` | SQL tokenizer, formatter, minifier, validator |
| `crates/sql_formatter_core/src/error.rs` | CoreError enum with stable error codes |
| `crates/sql_formatter_core/src/wasm.rs` | WASM bindings — JS-facing functions |
| `crates/sql_formatter_core/src/lib.rs` | Crate root, re-exports |
| `crates/sql_formatter_web/src/lib.rs` | Web crate WASM bridge (re-exports core WASM) |
| `crates/sql_formatter_web/static/index.html` | Full-featured editor UI |
| `skills/sql_formatter.md` | Agent usage workflow |
| `skills/mcp-tools.json` | MCP tool definitions |
| `docs/product_spec.zh-CN.md` | Product specification |

## Build & Test Commands

```bash
# Run all tests
cargo test --workspace

# Format check
cargo fmt --all -- --check

# Lint (strict)
cargo clippy --workspace --all-targets -- -D warnings

# WASM compilation check
cargo check -p sql_formatter_web --target wasm32-unknown-unknown

# Build Web WASM for deployment
wasm-pack build --target web crates/sql_formatter_web
```

## Design Principles

1. **Browser-first**: All SQL processing happens in-browser via WASM; no data leaves the client
2. **Zero dependencies for tokenization**: Custom tokenizer — no `sqlparser` or `sqlformat` crates needed
3. **Dialect-aware**: Keyword lists differ per dialect (e.g., MySQL backtick identifiers, PostgreSQL `::` casts)
4. **Conservative formatting**: The formatter never changes SQL semantics — it only adjusts whitespace
5. **Syntax-aware but not semantic**: Validates token structure, not database-level correctness

## Tokenizer Design

### Token Types
- **Keyword**: SQL reserved words and common function names (SELECT, FROM, WHERE, COUNT, etc.)
- **Identifier**: Unquoted names like table/column names
- **QuotedIdentifier**: `"ident"` (standard), `` `ident` `` (MySQL), `[ident]` (SQL Server)
- **String**: `'string literal'` with `''` escaping
- **Number**: Integers, decimals, scientific notation (`123`, `3.14`, `1e10`)
- **Operator**: `=`, `<>`, `!=`, `<`, `>`, `<=`, `>=`, `+`, `-`, `*`, `/`, `%`, `||`, `&&`
- **Punctuation**: `(`, `)`, `,`, `;`, `.`
- **Comment**: `-- line`, `/* block */`
- **Whitespace**: spaces, tabs, newlines (discarded during tokenization, re-generated during formatting)

### Formatting Rules
- **Newline before**: SELECT, FROM, WHERE, JOIN, INNER JOIN, LEFT JOIN, RIGHT JOIN, FULL JOIN, CROSS JOIN, ON, GROUP BY, ORDER BY, HAVING, LIMIT, OFFSET, UNION, INTERSECT, EXCEPT, INSERT, UPDATE, DELETE, SET, VALUES, CREATE, ALTER, DROP, BEGIN, COMMIT, ROLLBACK, RETURNING, WITH
- **Indent after**: `(`, `BEGIN`, `CASE`
- **Dedent before**: `)`, `END`
- **Comma placement**: before (leading) or after (trailing) per options
- **Lines between statements**: controlled by `lines_between_statements`

## Error Codes (Stable Machine-Readable)

| Code | Meaning |
|------|---------|
| `PARSE_ERROR` | Tokenization or syntax error |
| `UNSUPPORTED_DIALECT` | Unknown dialect string |
| `EMPTY_INPUT` | Empty or whitespace-only SQL string |
| `VALIDATION_ERROR` | Validation warnings/errors found |

## Supported Dialects

- `Generic` — Standard SQL (ANSI/ISO)
- `PostgreSql` — PostgreSQL (supports `::` casts, `$1` params, `"ident"`)
- `MySql` — MySQL (supports backtick identifiers, `#` comments)
- `Sqlite` — SQLite (supports `"ident"`, square-bracket identifiers)

## Frontend Design Requirement

- Before creating, modifying, reviewing, or debugging any HTML page or user-facing frontend, invoke the `ui-ux-pro-max` skill.
- Run the skill's required `--design-system` search before editing, followed by relevant stack and UX searches.
- If `ui-ux-pro-max` is unavailable, stop frontend work and report the missing prerequisite.
- Verify the rendered result in a real browser at 375, 768, 1024, and 1440 pixel widths, including console, keyboard, accessibility, and overflow checks.
