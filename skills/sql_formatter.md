# sql_formatter Agent Skill

A browser-native SQL formatter and minifier. Format, minify, and validate SQL via WASM — no server upload required.

## Workflow

1. **Format SQL**: Call `format_sql` with the SQL text, dialect, indent size, keyword case preference, and comma placement.
2. **Minify SQL**: Call `minify_sql` to compress SQL to a single line.
3. **Validate SQL**: Call `validate_sql` to check for syntax errors or warnings before executing.
4. **List Dialects**: Call `list_sql_dialects` to see available dialect options.

## Tool Definitions

### `format_sql`

Format SQL with configurable options.

**Parameters:**
- `sql` (string, required): The SQL text to format
- `dialect` (string, optional): SQL dialect — `"generic"`, `"postgresql"`, `"mysql"`, `"sqlite"`. Default `"generic"`
- `indent_size` (integer, optional): Indent in spaces — 2, 4, or 8. Default 4
- `keyword_case` (string, optional): Keyword casing — `"upper"`, `"lower"`, `"capitalize"`. Default `"upper"`
- `comma_before` (boolean, optional): Place commas before items (leading style). Default false
- `lines_between_statements` (integer, optional): Blank lines between semicolon-separated statements. Default 1
- `max_line_width` (integer, optional): Target max line width. Default 120

**Returns:**
- `formatted`: The formatted SQL string

### `minify_sql`

Compress SQL to a single line by removing unnecessary whitespace and comments.

**Parameters:**
- `sql` (string, required): The SQL text to minify

**Returns:**
- `minified`: The minified SQL string (single line)

### `validate_sql`

Check SQL for syntax errors and warnings.

**Parameters:**
- `sql` (string, required): The SQL text to validate
- `dialect` (string, optional): SQL dialect. Default `"generic"`

**Returns:**
- `valid`: Boolean indicating if SQL is syntactically valid
- `issues`: Array of issue strings (warnings and errors)

### `list_sql_dialects`

Return the list of supported SQL dialects.

**Parameters:** none

**Returns:**
- `dialects`: Array of dialect name strings

## Agent Rules

- Never claim the formatter can detect database-level semantic errors — it only checks syntax.
- Never suggest uploading SQL to a server for formatting — everything runs in-browser.
- Prefer `format_sql` over `minify_sql` for readability recommendations.
- When returning formatted SQL, use a code block with `sql` language tag.
