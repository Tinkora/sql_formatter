use crate::error::CoreError;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Public API types
// ---------------------------------------------------------------------------

/// Supported SQL dialects.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SqlDialect {
    /// Standard / ANSI SQL.
    Generic,
    /// PostgreSQL (supports `::` casts, `$1` params, `"ident"`).
    PostgreSql,
    /// MySQL (supports backtick identifiers, `#` comments).
    MySql,
    /// SQLite (supports `"ident"`, square-bracket identifiers).
    Sqlite,
}

impl SqlDialect {
    /// Parse from lowercase string; case-insensitive.
    pub fn from_str(s: &str) -> Result<Self, CoreError> {
        match s.to_lowercase().as_str() {
            "generic" => Ok(Self::Generic),
            "postgresql" | "postgres" | "pg" => Ok(Self::PostgreSql),
            "mysql" | "mariadb" => Ok(Self::MySql),
            "sqlite" => Ok(Self::Sqlite),
            other => Err(CoreError::UnsupportedDialect(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Generic => "generic",
            Self::PostgreSql => "postgresql",
            Self::MySql => "mysql",
            Self::Sqlite => "sqlite",
        }
    }

    pub fn list_all() -> Vec<&'static str> {
        vec!["generic", "postgresql", "mysql", "sqlite"]
    }
}

/// Keyword casing style.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeywordCase {
    Upper,
    Lower,
    Capitalize,
}

impl KeywordCase {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "upper" | "uppercase" => Self::Upper,
            "lower" | "lowercase" => Self::Lower,
            "capitalize" => Self::Capitalize,
            _ => Self::Upper,
        }
    }
}

/// Options controlling SQL formatting output.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FormatOptions {
    /// Target SQL dialect (affects keyword matching and comment styles).
    pub dialect: SqlDialect,
    /// Number of spaces per indent level (2, 4, or 8).
    pub indent_size: u8,
    /// Keyword casing: uppercase, lowercase, or capitalize (first letter upper).
    pub keyword_case: KeywordCase,
    /// When true, commas are placed before the item (leading style).
    pub comma_before: bool,
    /// Number of blank lines between semicolon-separated statements.
    pub lines_between_statements: u8,
    /// Target maximum line width (reserved for future line-wrapping support).
    pub max_line_width: u32,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            dialect: SqlDialect::Generic,
            indent_size: 4,
            keyword_case: KeywordCase::Upper,
            comma_before: false,
            lines_between_statements: 1,
            max_line_width: 120,
        }
    }
}

// ---------------------------------------------------------------------------
// Token definitions
// ---------------------------------------------------------------------------

/// A single token produced by the SQL tokenizer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// An SQL keyword (e.g., SELECT, FROM, WHERE).
    Keyword,
    /// An unquoted identifier (table name, column name, alias).
    Identifier,
    /// A quoted identifier: `"ident"` (standard / PG / SQLite),
    /// `` `ident` `` (MySQL), or `[ident]` (SQL Server / SQLite).
    QuotedIdentifier,
    /// A single-quoted string literal: `'value'` with `''` escaping.
    StringLiteral,
    /// A numeric literal: integer, decimal, or scientific notation.
    Number,
    /// An operator: `=`, `<>`, `!=`, `<`, `>`, `<=`, `>=`, `+`, `-`,
    /// `*`, `/`, `%`, `||`, `&&`, `::`.
    Operator,
    /// Punctuation: `(`, `)`, `,`, `;`, `.`.
    Punctuation,
    /// A line comment: `-- text` (or `# text` in MySQL).
    LineComment,
    /// A block comment: `/* text */`.
    BlockComment,
    /// A named parameter: `:param` or `$1`.
    NamedParam,
}

/// A token with its text content.
#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    /// The original text of the token as it appears in the source.
    pub text: String,
}

// ---------------------------------------------------------------------------
// Keyword sets
// ---------------------------------------------------------------------------

/// SQL keywords that trigger a newline before them in formatted output.
/// These are the "major clause" keywords.
const NEWLINE_BEFORE_KEYWORDS: &[&str] = &[
    "SELECT", "FROM", "WHERE", "AND", "OR", "NOT",
    "INNER", "LEFT", "RIGHT", "FULL", "CROSS", "NATURAL",
    "JOIN", "ON", "USING",
    "GROUP", "ORDER", "BY", "HAVING", "LIMIT", "OFFSET", "FETCH",
    "UNION", "INTERSECT", "EXCEPT", "MINUS",
    "INSERT", "UPDATE", "DELETE", "SET", "VALUES", "INTO",
    "CREATE", "ALTER", "DROP", "TRUNCATE", "REPLACE",
    "BEGIN", "COMMIT", "ROLLBACK", "SAVEPOINT",
    "RETURNING", "WITH", "RECURSIVE",
    "WHEN", "THEN", "ELSE", "END",
    "ASC", "DESC", "NULLS",
    "OVER", "PARTITION", "WINDOW",
    "LATERAL", "EXISTS", "ANY", "ALL", "SOME",
    "IS", "NULL", "NOTNULL",
    "BETWEEN", "LIKE", "ILIKE", "SIMILAR", "IN", "GLOB", "REGEXP",
    "DISTINCT", "ALL",
    "AS",
    "CASE",
    "PRIMARY", "FOREIGN", "KEY", "REFERENCES", "CONSTRAINT", "CHECK", "DEFAULT",
    "UNIQUE", "INDEX", "CASCADE",
    "ADD", "COLUMN",
    "GRANT", "REVOKE",
    "DECLARE", "EXEC", "EXECUTE", "CALL",
    "LOCK", "UNLOCK",
    "ANALYZE", "EXPLAIN", "VACUUM", "REINDEX", "ATTACH", "DETACH",
    "PRAGMA",
    "DO", "NOTHING",
    "CONFLICT",
    "MERGE",
    "LANGUAGE", "IMMUTABLE", "STABLE", "VOLATILE", "STRICT",
    "RETURNS", "AS",
    "TEMP", "TEMPORARY",
    "IF",
    "MATERIALIZED",
    "REFRESH",
    "CONCURRENTLY",
    "ON",
    "USING",
    "HAVING",
];

/// SQL keywords that are recognised as keywords (for case-transformation)
/// but don't necessarily trigger a newline.
const ALL_KEYWORDS: &[&str] = &[
    "ABORT", "ACTION", "ADD", "AFTER", "ALL", "ALTER", "ANALYZE", "AND",
    "ANY", "AS", "ASC", "ATTACH", "AUTOINCREMENT", "BEFORE", "BEGIN",
    "BETWEEN", "BY", "CASCADE", "CASE", "CAST", "CHECK", "COALESCE",
    "COLLATE", "COLUMN", "COMMIT", "CONCURRENTLY", "CONFLICT", "CONSTRAINT",
    "CREATE", "CROSS", "CURRENT", "CURRENT_DATE", "CURRENT_TIME",
    "CURRENT_TIMESTAMP", "DATABASE", "DECLARE", "DEFAULT", "DEFERRABLE",
    "DEFERRED", "DELETE", "DESC", "DETACH", "DISTINCT", "DO", "DROP",
    "EACH", "ELSE", "END", "ESCAPE", "EXCEPT", "EXCLUDE", "EXCLUSIVE",
    "EXEC", "EXECUTE", "EXISTS", "EXPLAIN", "FAIL", "FETCH", "FILTER",
    "FIRST", "FOLLOWING", "FOR", "FOREIGN", "FROM", "FULL", "FUNCTION",
    "GLOB", "GRANT", "GROUP", "GROUPS", "HAVING", "IF", "IGNORE",
    "ILIKE", "IMMEDIATE", "IMMUTABLE", "IN", "INDEX", "INDEXED", "INITIALLY",
    "INNER", "INSERT", "INSTEAD", "INTERSECT", "INTO", "IS", "ISNULL",
    "JOIN", "KEY", "LANGUAGE", "LAST", "LATERAL", "LEFT", "LIKE", "LIMIT",
    "LOCK", "MATCH", "MATERIALIZED", "MERGE", "MINUS", "NATURAL", "NO",
    "NOT", "NOTHING", "NOTNULL", "NULL", "NULLIF", "NULLS", "OF", "OFFSET",
    "ON", "OR", "ORDER", "OTHERS", "OUTER", "OVER", "PARTITION", "PLAN",
    "PRAGMA", "PRECEDING", "PRIMARY", "QUERY", "RAISE", "RANGE", "RECURSIVE",
    "REFERENCES", "REFRESH", "REGEXP", "REINDEX", "RELEASE", "RENAME",
    "REPLACE", "RESTRICT", "RETURNING", "RETURNS", "REVOKE", "RIGHT",
    "ROLLBACK", "ROW", "ROWS", "SAVEPOINT", "SELECT", "SET", "SIMILAR",
    "SOME", "STABLE", "STRICT", "TABLE", "TEMP", "TEMPORARY", "THEN",
    "TIES", "TO", "TRANSACTION", "TRIGGER", "TRUNCATE", "UNBOUNDED",
    "UNION", "UNIQUE", "UNLOCK", "UPDATE", "USING", "VACUUM", "VALUES",
    "VIEW", "VOLATILE", "WHEN", "WHERE", "WINDOW", "WITH", "WITHOUT",
    // Additional common keywords
    "ACCESS", "ARRAY", "BIGINT", "BINARY", "BIT", "BLOB", "BOOLEAN",
    "CHAR", "CHARACTER", "CLOB", "DATE", "DATETIME", "DEC", "DECIMAL",
    "DOUBLE", "ENUM", "FLOAT", "GEOMETRY", "GRAPH", "IMAGE", "INT",
    "INTEGER", "INTERVAL", "JSON", "JSONB", "LONGTEXT", "MEDIUMINT",
    "MEDIUMTEXT", "MONEY", "NCHAR", "NUMBER", "NUMERIC", "NVARCHAR",
    "PRECISION", "REAL", "SERIAL", "SET", "SMALLINT", "TEXT", "TIME",
    "TIMESTAMP", "TIMESTAMPTZ", "TIMETZ", "TINYINT", "VARBINARY",
    "VARCHAR", "VARYING", "XML", "YEAR", "ZONE",
];

// ---------------------------------------------------------------------------
// Tokenizer
// ---------------------------------------------------------------------------

/// Tokenize an SQL string into a vector of tokens.
/// Whitespace is NOT included — it will be re-generated during formatting.
/// Comments ARE included so the formatter can decide whether to keep or strip them.
pub fn tokenize(sql: &str) -> Result<Vec<Token>, CoreError> {
    let chars: Vec<char> = sql.chars().collect();
    let len = chars.len();
    let mut pos = 0usize;
    let mut tokens: Vec<Token> = Vec::new();

    while pos < len {
        let ch = chars[pos];

        // Whitespace — skip
        if ch.is_whitespace() {
            pos += 1;
            continue;
        }

        // Line comment: -- (and MySQL #)
        if ch == '-' && pos + 1 < len && chars[pos + 1] == '-' {
            let start = pos;
            pos += 2; // skip --
            while pos < len && chars[pos] != '\n' && chars[pos] != '\r' {
                pos += 1;
            }
            tokens.push(Token {
                kind: TokenKind::LineComment,
                text: chars[start..pos].iter().collect(),
            });
            // Don't consume the newline — let whitespace handling skip it
            continue;
        }

        // MySQL # line comment
        if ch == '#' {
            let start = pos;
            pos += 1;
            while pos < len && chars[pos] != '\n' && chars[pos] != '\r' {
                pos += 1;
            }
            tokens.push(Token {
                kind: TokenKind::LineComment,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // Block comment: /* ... */
        if ch == '/' && pos + 1 < len && chars[pos + 1] == '*' {
            let start = pos;
            pos += 2; // skip /*
            let mut depth = 1i32;
            while pos < len && depth > 0 {
                if chars[pos] == '/' && pos + 1 < len && chars[pos + 1] == '*' {
                    depth += 1;
                    pos += 2;
                } else if chars[pos] == '*' && pos + 1 < len && chars[pos + 1] == '/' {
                    depth -= 1;
                    pos += 2;
                } else {
                    pos += 1;
                }
            }
            if depth != 0 {
                return Err(CoreError::ParseError(
                    "Unterminated block comment starting at /*".to_string(),
                ));
            }
            tokens.push(Token {
                kind: TokenKind::BlockComment,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // Single-quoted string
        if ch == '\'' {
            let start = pos;
            pos += 1; // skip opening quote
            while pos < len {
                if chars[pos] == '\'' {
                    if pos + 1 < len && chars[pos + 1] == '\'' {
                        // Escaped quote within string
                        pos += 2;
                    } else {
                        pos += 1; // skip closing quote
                        break;
                    }
                } else {
                    pos += 1;
                }
            }
            if pos > len || (pos <= len && start + 1 >= pos) {
                // Check: did we exit without finding closing quote?
                // We need to verify the string ended properly.
                if pos > len || chars.get(pos - 1) != Some(&'\'') {
                    return Err(CoreError::ParseError(
                        "Unterminated string literal".to_string(),
                    ));
                }
            }
            tokens.push(Token {
                kind: TokenKind::StringLiteral,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // Double-quoted identifier
        if ch == '"' {
            let start = pos;
            pos += 1;
            while pos < len {
                if chars[pos] == '"' {
                    if pos + 1 < len && chars[pos + 1] == '"' {
                        pos += 2;
                    } else {
                        pos += 1;
                        break;
                    }
                } else {
                    pos += 1;
                }
            }
            tokens.push(Token {
                kind: TokenKind::QuotedIdentifier,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // Backtick identifier (MySQL)
        if ch == '`' {
            let start = pos;
            pos += 1;
            while pos < len {
                if chars[pos] == '`' {
                    if pos + 1 < len && chars[pos + 1] == '`' {
                        pos += 2;
                    } else {
                        pos += 1;
                        break;
                    }
                } else {
                    pos += 1;
                }
            }
            tokens.push(Token {
                kind: TokenKind::QuotedIdentifier,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // Square-bracket identifier (SQL Server / SQLite)
        if ch == '[' {
            let start = pos;
            pos += 1;
            while pos < len && chars[pos] != ']' {
                pos += 1;
            }
            if pos < len {
                pos += 1; // skip ]
            }
            tokens.push(Token {
                kind: TokenKind::QuotedIdentifier,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // Named parameter: :param or $1 (PostgreSQL style)
        if ch == ':' || ch == '$' {
            let start = pos;
            pos += 1;
            if ch == '$' {
                // Must be followed by digits for $1 style
                while pos < len && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
            } else {
                // :param — must be followed by a letter or underscore
                if pos < len && (chars[pos].is_alphabetic() || chars[pos] == '_') {
                    while pos < len && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                        pos += 1;
                    }
                }
            }
            let text: String = chars[start..pos].iter().collect();
            if text.len() > 1 {
                tokens.push(Token {
                    kind: TokenKind::NamedParam,
                    text,
                });
            } else {
                // Single ':' or '$' is an operator or part of something else
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    text,
                });
            }
            continue;
        }

        // PostgreSQL type cast operator ::
        if ch == ':' && pos + 1 < len && chars[pos + 1] == ':' {
            tokens.push(Token {
                kind: TokenKind::Operator,
                text: "::".to_string(),
            });
            pos += 2;
            continue;
        }

        // Numbers
        if ch.is_ascii_digit() || (ch == '.' && pos + 1 < len && chars[pos + 1].is_ascii_digit()) {
            let start = pos;
            // Hexadecimal: 0x...
            if ch == '0' && pos + 1 < len && (chars[pos + 1] == 'x' || chars[pos + 1] == 'X') {
                pos += 2;
                while pos < len && chars[pos].is_ascii_hexdigit() {
                    pos += 1;
                }
            } else {
                while pos < len && chars[pos].is_ascii_digit() {
                    pos += 1;
                }
                // Fractional part
                if pos < len && chars[pos] == '.' {
                    pos += 1;
                    while pos < len && chars[pos].is_ascii_digit() {
                        pos += 1;
                    }
                }
                // Scientific notation
                if pos < len && (chars[pos] == 'e' || chars[pos] == 'E') {
                    pos += 1;
                    if pos < len && (chars[pos] == '+' || chars[pos] == '-') {
                        pos += 1;
                    }
                    while pos < len && chars[pos].is_ascii_digit() {
                        pos += 1;
                    }
                }
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // Multi-char operators
        if ch == '<' || ch == '>' || ch == '!' || ch == '=' {
            let start = pos;
            pos += 1;
            // Check for two-char operators: <>, <=, >=, !=, ==
            if pos < len && chars[pos] == '=' {
                pos += 1;
            } else if ch == '<' && pos < len && chars[pos] == '>' {
                pos += 1;
            } else if ch == '!' && pos < len && chars[pos] == '=' {
                pos += 1;
            } else if ch == '>' && pos < len && chars[pos] == '>' {
                // >> operator
                pos += 1;
            } else if ch == '<' && pos < len && chars[pos] == '<' {
                // << operator
                pos += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Operator,
                text: chars[start..pos].iter().collect(),
            });
            continue;
        }

        // || (string concatenation) and && (overlap in PostgreSQL)
        if (ch == '|' || ch == '&') && pos + 1 < len && chars[pos + 1] == ch {
            tokens.push(Token {
                kind: TokenKind::Operator,
                text: chars[pos..pos + 2].iter().collect(),
            });
            pos += 2;
            continue;
        }

        // Single-char operators
        if matches!(ch, '+' | '-' | '*' | '/' | '%' | '^' | '~' | '@') {
            tokens.push(Token {
                kind: TokenKind::Operator,
                text: ch.to_string(),
            });
            pos += 1;
            continue;
        }

        // Punctuation
        if matches!(ch, '(' | ')' | ',' | ';' | '.') {
            tokens.push(Token {
                kind: TokenKind::Punctuation,
                text: ch.to_string(),
            });
            pos += 1;
            continue;
        }

        // Identifier (must start with letter or underscore)
        if ch.is_alphabetic() || ch == '_' {
            let start = pos;
            while pos < len && (chars[pos].is_alphanumeric() || chars[pos] == '_') {
                pos += 1;
            }
            let text: String = chars[start..pos].iter().collect();
            let upper = text.to_uppercase();

            // Check if it's a keyword
            if ALL_KEYWORDS.contains(&upper.as_str()) {
                tokens.push(Token {
                    kind: TokenKind::Keyword,
                    text,
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Identifier,
                    text,
                });
            }
            continue;
        }

        // Unknown character — treat as identifier or operator depending on context
        // This catches Unicode and other edge cases
        tokens.push(Token {
            kind: TokenKind::Identifier,
            text: ch.to_string(),
        });
        pos += 1;
    }

    Ok(tokens)
}

// ---------------------------------------------------------------------------
// Keyword helpers
// ---------------------------------------------------------------------------

/// Check if a keyword text (uppercased) should trigger a newline before it.
fn is_newline_before_keyword(text: &str) -> bool {
    let upper = text.to_uppercase();
    NEWLINE_BEFORE_KEYWORDS.contains(&upper.as_str())
}

/// Check if a keyword text (uppercased) is one that typically starts a clause
/// and should be at the left margin (same indent as the enclosing block).
fn is_major_clause(text: &str) -> bool {
    let upper = text.to_uppercase();
    matches!(
        upper.as_str(),
        "SELECT" | "FROM" | "WHERE" | "JOIN" | "INNER" | "LEFT" | "RIGHT"
            | "FULL" | "CROSS" | "NATURAL" | "ON" | "GROUP" | "ORDER"
            | "HAVING" | "LIMIT" | "OFFSET" | "UNION" | "INTERSECT"
            | "EXCEPT" | "INSERT" | "UPDATE" | "DELETE" | "SET" | "VALUES"
            | "CREATE" | "ALTER" | "DROP" | "RETURNING" | "WITH"
            | "WHEN" | "ELSE" | "END" | "BEGIN" | "COMMIT" | "ROLLBACK"
            | "AND" | "OR"
    )
}

/// Apply keyword case transformation.
fn apply_keyword_case(text: &str, case: KeywordCase) -> String {
    match case {
        KeywordCase::Upper => text.to_uppercase(),
        KeywordCase::Lower => text.to_lowercase(),
        KeywordCase::Capitalize => {
            let lower = text.to_lowercase();
            let mut chars: Vec<char> = lower.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_uppercase().next().unwrap_or(*first);
            }
            chars.into_iter().collect()
        }
    }
}

// ---------------------------------------------------------------------------
// Formatter
// ---------------------------------------------------------------------------

/// Format SQL tokens into a pretty-printed string.
pub fn format_tokens(tokens: &[Token], options: &FormatOptions) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let indent_str = " ".repeat(options.indent_size as usize);
    let mut depth: i32 = 0;
    let mut output = String::new();
    let mut i = 0usize;
    let len = tokens.len();

    // Track state for comma placement and whitespace decisions
    // pending_newline: we need to emit a newline + indent before the next real token
    let mut pending_newline = false;
    // after_newline: we just emitted a newline, so we are at the start of a line
    let mut after_newline = true;

    while i < len {
        let token = &tokens[i];
        let next_kind = if i + 1 < len {
            Some(&tokens[i + 1].kind)
        } else {
            None
        };

        match &token.kind {
            TokenKind::Keyword => {
                let upper = token.text.to_uppercase();

                // Special handling: END should dedent BEFORE being emitted
                if upper == "END" && depth > 0 {
                    depth -= 1;
                }

                // Newline before major clauses
                if is_newline_before_keyword(&token.text) && !after_newline {
                    // Don't newline if the previous token was '('
                    let prev_is_open_paren = i > 0
                        && matches!(
                            tokens[i - 1].kind,
                            TokenKind::Punctuation
                        )
                        && tokens[i - 1].text == "(";
                    if !prev_is_open_paren {
                        output.push('\n');
                        after_newline = true;
                    }
                }

                // Indent when at start of line
                if after_newline && !output.is_empty() {
                    let effective_depth = if is_major_clause(&token.text) {
                        depth
                    } else {
                        depth
                    };
                    for _ in 0..effective_depth.max(0) {
                        output.push_str(&indent_str);
                    }
                }

                // Add space before keyword if not at line start and previous wasn't '('
                if !after_newline && !output.is_empty() && !output.ends_with('(') {
                    output.push(' ');
                }

                let cased = apply_keyword_case(&token.text, options.keyword_case);
                output.push_str(&cased);
                after_newline = false;
                pending_newline = false;
            }

            TokenKind::Identifier | TokenKind::QuotedIdentifier => {
                if after_newline && !output.is_empty() {
                    for _ in 0..depth.max(0) + 1 {
                        output.push_str(&indent_str);
                    }
                } else if !after_newline && !output.is_empty() && !output.ends_with('(') && !output.ends_with('.') {
                    output.push(' ');
                }

                output.push_str(&token.text);
                after_newline = false;
                pending_newline = false;
            }

            TokenKind::StringLiteral | TokenKind::Number | TokenKind::NamedParam => {
                if !after_newline && !output.is_empty() && !output.ends_with('(') {
                    output.push(' ');
                } else if after_newline && !output.is_empty() {
                    for _ in 0..depth.max(0) + 2 {
                        output.push_str(&indent_str);
                    }
                }
                output.push_str(&token.text);
                after_newline = false;
                pending_newline = false;
            }

            TokenKind::Operator => {
                // Add space before operator (unless at line start or after '(')
                if !after_newline && !output.is_empty() && !output.ends_with('(') && !output.ends_with(' ') {
                    output.push(' ');
                }
                output.push_str(&token.text);
                // Don't add trailing space yet — the next token will decide
                after_newline = false;
                pending_newline = false;
            }

            TokenKind::Punctuation => {
                let ch = token.text.chars().next().unwrap();

                match ch {
                    '(' => {
                        // No space before '(' unless preceded by a keyword or identifier
                        if !after_newline && !output.is_empty() && !output.ends_with(' ') && !output.ends_with('(') {
                            // Check if the previous token was certain keywords like IN, VALUES, etc.
                            let prev_is_keyword = i > 0 && matches!(tokens[i - 1].kind, TokenKind::Keyword);
                            if prev_is_keyword {
                                output.push(' ');
                            }
                        }
                        output.push('(');
                        depth += 1;
                        // Check if next token is ')' (empty parens) or SELECT (subquery)
                        if let Some(next) = next_kind {
                            if matches!(next, TokenKind::Keyword) && i + 1 < len {
                                let next_text = &tokens[i + 1].text.to_uppercase();
                                if next_text == "SELECT" {
                                    output.push('\n');
                                    after_newline = true;
                                }
                            }
                        }
                        after_newline = false;
                    }
                    ')' => {
                        if depth > 0 {
                            depth -= 1;
                        }
                        // No space before ')'
                        output.push(')');
                        after_newline = false;
                    }
                    ',' => {
                        if options.comma_before {
                            // Leading comma: newline + indent + comma + space
                            output.push('\n');
                            for _ in 0..depth.max(0) {
                                output.push_str(&indent_str);
                            }
                            output.push(',');
                            after_newline = false;
                        } else {
                            // Trailing comma: comma + (optionally newline for multi-line)
                            output.push(',');
                            // Check if we should newline after comma in certain contexts
                            // For now, just add space; newlines are triggered by subsequent keywords
                            after_newline = false;
                        }
                    }
                    ';' => {
                        output.push(';');
                        // Add blank lines between statements
                        if i + 1 < len {
                            let blank_lines = options.lines_between_statements as usize + 1;
                            for _ in 0..blank_lines {
                                output.push('\n');
                            }
                            after_newline = true;
                            depth = 0;
                        }
                        after_newline = false;
                    }
                    '.' => {
                        output.push('.');
                        after_newline = false;
                    }
                    _ => {
                        output.push_str(&token.text);
                        after_newline = false;
                    }
                }
                pending_newline = false;
            }

            TokenKind::LineComment | TokenKind::BlockComment => {
                // In formatting mode, preserve comments
                if !after_newline && !output.is_empty() {
                    output.push(' ');
                }
                output.push_str(&token.text);
                // Line comments should be followed by a newline
                if matches!(token.kind, TokenKind::LineComment) {
                    output.push('\n');
                    after_newline = true;
                }
            }
        }

        i += 1;
    }

    // Clean up: trim trailing whitespace
    let trimmed = output.trim_end().to_string();

    // Ensure single trailing newline
    if trimmed.is_empty() {
        trimmed
    } else {
        format!("{}\n", trimmed)
    }
}

// ---------------------------------------------------------------------------
// Minifier
// ---------------------------------------------------------------------------

/// Minify SQL tokens into a single line with minimal whitespace.
pub fn minify_tokens(tokens: &[Token]) -> String {
    if tokens.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    let mut prev_kind: Option<&TokenKind> = None;
    let mut prev_text: Option<&str> = None;

    for token in tokens {
        // Skip comments entirely
        if matches!(token.kind, TokenKind::LineComment | TokenKind::BlockComment) {
            continue;
        }

        let needs_space = match &token.kind {
            TokenKind::Keyword => {
                // Space before keyword unless after '(' or at start
                !output.is_empty()
                    && !output.ends_with('(')
                    && !matches!(prev_kind, Some(TokenKind::Punctuation) if prev_text == Some("("))
            }
            TokenKind::Identifier | TokenKind::QuotedIdentifier => {
                !output.is_empty()
                    && !output.ends_with('(')
                    && !output.ends_with('.')
                    && !matches!(prev_kind, Some(TokenKind::Operator))
            }
            TokenKind::StringLiteral | TokenKind::Number | TokenKind::NamedParam => {
                !output.is_empty()
                    && !output.ends_with('(')
                    && !matches!(prev_kind, Some(TokenKind::Operator))
            }
            TokenKind::Operator => {
                // Space before operator unless after '('
                !output.is_empty() && !output.ends_with('(')
            }
            TokenKind::Punctuation => {
                let ch = token.text.chars().next().unwrap();
                match ch {
                    '(' | '[' => {
                        // Space before '(' if preceded by keyword or identifier
                        matches!(prev_kind, Some(TokenKind::Keyword) | Some(TokenKind::Identifier) | Some(TokenKind::QuotedIdentifier) | Some(TokenKind::StringLiteral) | Some(TokenKind::Number) | Some(TokenKind::NamedParam) | Some(TokenKind::Punctuation) if prev_text == Some(")"))
                    }
                    ',' | ';' => false, // No space before comma/semicolon
                    '.' => false,      // No space before dot
                    ')' | ']' => false, // No space before closing paren/bracket
                    _ => !output.is_empty(),
                }
            }
            _ => !output.is_empty(),
        };

        if needs_space {
            output.push(' ');
        }

        // Apply uppercase to keywords for minified output (standard practice)
        match &token.kind {
            TokenKind::Keyword => {
                output.push_str(&token.text.to_uppercase());
            }
            _ => {
                output.push_str(&token.text);
            }
        }

        prev_kind = Some(&token.kind);
        prev_text = Some(&token.text);
    }

    // Clean up: collapse multiple spaces
    let mut result = String::new();
    let mut last_was_space = false;
    for ch in output.chars() {
        if ch.is_whitespace() {
            if !last_was_space {
                result.push(' ');
                last_was_space = true;
            }
        } else {
            result.push(ch);
            last_was_space = false;
        }
    }

    result.trim().to_string()
}

// ---------------------------------------------------------------------------
// Validator
// ---------------------------------------------------------------------------

/// Validate SQL tokens and return a list of issues (warnings and errors).
pub fn validate_tokens(tokens: &[Token]) -> Vec<String> {
    let mut issues: Vec<String> = Vec::new();
    let mut paren_depth: i32 = 0;

    for (i, token) in tokens.iter().enumerate() {
        match &token.kind {
            TokenKind::Punctuation => {
                match token.text.as_str() {
                    "(" => paren_depth += 1,
                    ")" => {
                        paren_depth -= 1;
                        if paren_depth < 0 {
                            issues.push(format!(
                                "Unmatched closing parenthesis at position ~{}",
                                i
                            ));
                            paren_depth = 0; // reset to avoid cascading errors
                        }
                    }
                    _ => {}
                }
            }
            TokenKind::BlockComment => {
                // Check for unclosed block comments (nested /* /* */ is OK)
                // Our tokenizer already handles this, but double-check
                if !token.text.ends_with("*/") {
                    issues.push(format!(
                        "Possibly unclosed block comment: {}",
                        &token.text[..token.text.len().min(40)]
                    ));
                }
            }
            _ => {}
        }
    }

    // Check final parenthesis balance
    if paren_depth > 0 {
        issues.push(format!(
            "{} unclosed opening parenthesis/parentheses",
            paren_depth
        ));
    }

    // Check empty input
    if tokens.is_empty() {
        issues.push("Input contains no SQL tokens".to_string());
    }

    issues
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Format an SQL string with the given options.
pub fn format_sql(sql: &str, options: &FormatOptions) -> Result<String, CoreError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyInput);
    }

    let tokens = tokenize(trimmed)?;
    Ok(format_tokens(&tokens, options))
}

/// Minify an SQL string to a single line.
pub fn minify_sql(sql: &str) -> Result<String, CoreError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyInput);
    }

    let tokens = tokenize(trimmed)?;
    Ok(minify_tokens(&tokens))
}

/// Validate an SQL string and return a list of issues (empty = valid).
pub fn validate_sql(sql: &str) -> Result<Vec<String>, CoreError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Err(CoreError::EmptyInput);
    }

    let tokens = tokenize(trimmed)?;
    Ok(validate_tokens(&tokens))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Tokenizer tests ---

    #[test]
    fn test_tokenize_simple_select() {
        let tokens = tokenize("SELECT * FROM users WHERE id = 1").unwrap();
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind.clone()).collect();
        assert_eq!(kinds[0], TokenKind::Keyword); // SELECT
        assert_eq!(tokens[0].text, "SELECT");
        assert_eq!(kinds[1], TokenKind::Operator); // *
        assert_eq!(kinds[2], TokenKind::Keyword); // FROM
        assert_eq!(kinds[3], TokenKind::Identifier); // users
        assert_eq!(kinds[4], TokenKind::Keyword); // WHERE
        assert_eq!(kinds[5], TokenKind::Identifier); // id
        assert_eq!(kinds[6], TokenKind::Operator); // =
        assert_eq!(kinds[7], TokenKind::Number); // 1
    }

    #[test]
    fn test_tokenize_string() {
        let tokens = tokenize("SELECT 'hello''world'").unwrap();
        assert_eq!(tokens[0].text, "SELECT");
        assert_eq!(tokens[1].text, "'hello''world'");
        assert_eq!(tokens[1].kind, TokenKind::StringLiteral);
    }

    #[test]
    fn test_tokenize_quoted_identifier() {
        let tokens = tokenize(r#"SELECT "my column" FROM t"#).unwrap();
        assert_eq!(tokens[1].text, r#""my column""#);
        assert_eq!(tokens[1].kind, TokenKind::QuotedIdentifier);
    }

    #[test]
    fn test_tokenize_backtick_identifier() {
        let tokens = tokenize("SELECT `my column` FROM t").unwrap();
        assert_eq!(tokens[1].text, "`my column`");
        assert_eq!(tokens[1].kind, TokenKind::QuotedIdentifier);
    }

    #[test]
    fn test_tokenize_line_comment() {
        let tokens = tokenize("SELECT 1 -- this is a comment\nFROM t").unwrap();
        let comments: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::LineComment))
            .collect();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "-- this is a comment");
    }

    #[test]
    fn test_tokenize_block_comment() {
        let tokens = tokenize("SELECT /* block */ 1").unwrap();
        let comments: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::BlockComment))
            .collect();
        assert_eq!(comments.len(), 1);
        assert_eq!(comments[0].text, "/* block */");
    }

    #[test]
    fn test_tokenize_nested_block_comment() {
        let tokens = tokenize("SELECT /* /* nested */ */ 1").unwrap();
        assert_eq!(tokens[0].text, "SELECT");
        assert_eq!(tokens[1].text, "/* /* nested */ */");
        assert_eq!(tokens[2].text, "1");
    }

    #[test]
    fn test_tokenize_number_formats() {
        let tokens = tokenize("SELECT 123, 3.14, 1e10, 0xABCD").unwrap();
        let nums: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Number))
            .collect();
        assert_eq!(nums.len(), 4);
        assert_eq!(nums[0].text, "123");
        assert_eq!(nums[1].text, "3.14");
        assert_eq!(nums[2].text, "1e10");
        assert_eq!(nums[3].text, "0xABCD");
    }

    #[test]
    fn test_tokenize_operators() {
        let tokens = tokenize("SELECT * FROM t WHERE a <> b AND c != d AND e >= f AND g <= h").unwrap();
        let ops: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Operator))
            .map(|t| t.text.as_str())
            .collect();
        assert!(ops.contains(&"*"));
        assert!(ops.contains(&"<>"));
        assert!(ops.contains(&"!="));
        assert!(ops.contains(&">="));
        assert!(ops.contains(&"<="));
    }

    #[test]
    fn test_tokenize_punctuation() {
        let tokens = tokenize("SELECT a, b, (c);").unwrap();
        let puncts: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::Punctuation))
            .map(|t| t.text.as_str())
            .collect();
        assert!(puncts.contains(&","));
        assert!(puncts.contains(&"("));
        assert!(puncts.contains(&")"));
        assert!(puncts.contains(&";"));
    }

    #[test]
    fn test_tokenize_postgres_params() {
        let tokens = tokenize("SELECT $1, $2").unwrap();
        let params: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::NamedParam))
            .collect();
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].text, "$1");
        assert_eq!(params[1].text, "$2");
    }

    #[test]
    fn test_tokenize_named_params() {
        let tokens = tokenize("SELECT :name, :age").unwrap();
        let params: Vec<_> = tokens
            .iter()
            .filter(|t| matches!(t.kind, TokenKind::NamedParam))
            .collect();
        assert_eq!(params.len(), 2);
    }

    // --- Format tests ---

    #[test]
    fn test_format_basic_select() {
        let result = format_sql(
            "select * from users where id = 1",
            &FormatOptions {
                dialect: SqlDialect::Generic,
                indent_size: 4,
                keyword_case: KeywordCase::Upper,
                comma_before: false,
                lines_between_statements: 1,
                max_line_width: 120,
            },
        )
        .unwrap();
        assert!(result.contains("SELECT"));
        assert!(result.contains("FROM"));
        assert!(result.contains("WHERE"));
        assert!(result.contains("\n"));
    }

    #[test]
    fn test_format_keyword_case_lower() {
        let result = format_sql(
            "SELECT * FROM users",
            &FormatOptions {
                keyword_case: KeywordCase::Lower,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(result.contains("select"));
        assert!(result.contains("from"));
    }

    #[test]
    fn test_format_keyword_case_capitalize() {
        let result = format_sql(
            "SELECT * FROM users",
            &FormatOptions {
                keyword_case: KeywordCase::Capitalize,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(result.contains("Select"));
        assert!(result.contains("From"));
    }

    #[test]
    fn test_format_with_subquery() {
        let sql = "SELECT * FROM (SELECT id FROM users) sub WHERE id > 0";
        let result = format_sql(sql, &FormatOptions::default()).unwrap();
        assert!(result.contains("("));
        assert!(result.contains(")"));
        // Should have indentation
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 2);
    }

    #[test]
    fn test_format_with_join() {
        let sql = "SELECT u.name, o.total FROM users u INNER JOIN orders o ON u.id = o.user_id WHERE o.total > 100";
        let result = format_sql(sql, &FormatOptions::default()).unwrap();
        assert!(result.contains("INNER JOIN"));
        assert!(result.contains("ON"));
    }

    #[test]
    fn test_format_empty_input() {
        let result = format_sql("   ", &FormatOptions::default());
        assert!(result.is_err());
        match result {
            Err(CoreError::EmptyInput) => {}
            _ => panic!("Expected EmptyInput error"),
        }
    }

    #[test]
    fn test_format_multiple_statements() {
        let sql = "SELECT 1; SELECT 2; SELECT 3";
        let result = format_sql(sql, &FormatOptions::default()).unwrap();
        let semicolons = result.matches(';').count();
        assert_eq!(semicolons, 2);
        // Should have blank lines between statements
        assert!(result.contains("\n\n"));
    }

    // --- Minify tests ---

    #[test]
    fn test_minify_basic() {
        let sql = "SELECT\n  *\nFROM\n  users\nWHERE\n  id = 1";
        let result = minify_sql(sql).unwrap();
        assert!(!result.contains('\n'));
        assert_eq!(result, "SELECT * FROM users WHERE id = 1");
    }

    #[test]
    fn test_minify_strips_comments() {
        let sql = "SELECT * -- comment\nFROM /* block */ users";
        let result = minify_sql(sql).unwrap();
        assert!(!result.contains("comment"));
        assert!(!result.contains("block"));
    }

    #[test]
    fn test_minify_empty_input() {
        let result = minify_sql("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_minify_preserves_strings() {
        let sql = "SELECT 'hello world' FROM users";
        let result = minify_sql(sql).unwrap();
        assert!(result.contains("'hello world'"));
    }

    // --- Validate tests ---

    #[test]
    fn test_validate_valid_sql() {
        let issues = validate_sql("SELECT * FROM users WHERE id = 1").unwrap();
        assert!(issues.is_empty());
    }

    #[test]
    fn test_validate_unmatched_paren() {
        let issues = validate_sql("SELECT * FROM (users WHERE id = 1").unwrap();
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.contains("unclosed")));
    }

    #[test]
    fn test_validate_empty() {
        let result = validate_sql("   ");
        assert!(result.is_err());
    }

    // --- Dialect tests ---

    #[test]
    fn test_dialect_from_str() {
        assert_eq!(SqlDialect::from_str("generic").unwrap(), SqlDialect::Generic);
        assert_eq!(SqlDialect::from_str("postgresql").unwrap(), SqlDialect::PostgreSql);
        assert_eq!(SqlDialect::from_str("postgres").unwrap(), SqlDialect::PostgreSql);
        assert_eq!(SqlDialect::from_str("mysql").unwrap(), SqlDialect::MySql);
        assert_eq!(SqlDialect::from_str("sqlite").unwrap(), SqlDialect::Sqlite);
    }

    #[test]
    fn test_dialect_from_str_invalid() {
        let result = SqlDialect::from_str("oracle");
        assert!(result.is_err());
    }

    #[test]
    fn test_comma_before_style() {
        let sql = "SELECT a, b, c FROM t";
        let result = format_sql(
            sql,
            &FormatOptions {
                comma_before: true,
                ..Default::default()
            },
        )
        .unwrap();
        // Leading commas: each comma should appear at start of its line
        let has_leading_comma = result.lines().any(|l| l.trim_start().starts_with(','));
        assert!(has_leading_comma);
    }

    #[test]
    fn test_format_complex_query() {
        let sql = "WITH cte AS (SELECT id, name FROM users WHERE active = 1) SELECT cte.name, COUNT(orders.id) FROM cte LEFT JOIN orders ON cte.id = orders.user_id GROUP BY cte.name HAVING COUNT(orders.id) > 5 ORDER BY cte.name LIMIT 10";
        let result = format_sql(sql, &FormatOptions::default()).unwrap();
        assert!(result.contains("WITH"));
        assert!(result.contains("GROUP BY"));
        assert!(result.contains("ORDER BY"));
        assert!(result.contains("LIMIT"));
    }

    #[test]
    fn test_case_expression_format() {
        let sql = "SELECT CASE WHEN x > 0 THEN 'positive' ELSE 'negative' END FROM t";
        let result = format_sql(sql, &FormatOptions::default()).unwrap();
        assert!(result.contains("CASE"));
        assert!(result.contains("WHEN"));
        assert!(result.contains("THEN"));
        assert!(result.contains("ELSE"));
        assert!(result.contains("END"));
    }
}
