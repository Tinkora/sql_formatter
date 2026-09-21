# sql_formatter 产品规格说明书

## 1. 产品概述

sql_formatter 是一个浏览器原生的 SQL 格式化与压缩工具。通过 WASM 技术，所有处理均在浏览器端完成，无需服务器，即开即用。

## 2. 核心功能

### 2.1 SQL 格式化

将任意 SQL 语句格式化为美观、一致的缩进风格。

**支持的配置项：**

| 配置项 | 类型 | 默认值 | 说明 |
|--------|------|--------|------|
| `dialect` | enum | Generic | SQL 方言：Generic, PostgreSql, MySql, Sqlite |
| `indent_size` | u8 | 4 | 缩进空格数 (2/4/8) |
| `keyword_case` | enum | Upper | 关键字大小写：Upper, Lower, Capitalize |
| `comma_before` | bool | false | `true` 时逗号前置；`false` 时逗号后置 |
| `lines_between_statements` | u8 | 1 | 语句间的空行数 |
| `max_line_width` | u32 | 120 | 最大行宽（预留，暂不实现自动换行） |

### 2.2 SQL 压缩

将 SQL 压缩为单行，移除所有不必要的空白和注释。

### 2.3 语法校验

对输入的 SQL 进行基础的语法检查，返回警告和错误列表。

## 3. 技术架构

```
sql_formatter/
├── crates/
│   ├── sql_formatter_core/     # Rust 核心：分词、格式化、压缩、校验
│   └── sql_formatter_web/      # WASM 桥接 + HTML 编辑器
├── docs/                        # 文档
└── skills/                      # Agent 技能定义
```

### 3.1 分词器 (Tokenizer)

自定义分词器，将 SQL 文本分解为以下 Token 类型：

- **Keyword** — SQL 关键字和常见函数名
- **Identifier** — 未加引号的标识符
- **QuotedIdentifier** — `"ident"`、`` `ident` ``、`[ident]`
- **String** — 单引号字符串 `'literal'`，支持 `''` 转义
- **Number** — 整数、小数、科学计数法
- **Operator** — `=`, `<>`, `!=`, `<`, `>`, `<=`, `>=`, `+`, `-`, `*`, `/`, `%`, `||`, `&&`, `::`
- **Punctuation** — `(`, `)`, `,`, `;`, `.`
- **Comment** — `-- 行注释`, `/* 块注释 */`
- **NamedParam** — `:param` 或 `$1` 样式的参数占位符

### 3.2 格式化规则

1. **换行前**（新行开始前插入换行）：SELECT, FROM, WHERE, AND, OR, JOIN 系列, ON, GROUP BY, ORDER BY, HAVING, LIMIT, OFFSET, UNION, INTERSECT, EXCEPT, INSERT, UPDATE, DELETE, SET, VALUES, CREATE, ALTER, DROP, BEGIN, COMMIT, ROLLBACK, RETURNING, WITH, ELSE, WHEN, THEN

2. **缩进增**（深度 +1）：`(` 和 `BEGIN`, `CASE`

3. **缩进减**（深度 -1）：`)` 和 `END`

4. **逗号位置**：
   - 后置模式：`, ` 跟在元素后面
   - 前置模式：换行 + 缩进 + `,` + 空格

5. **语句间空行**：两个 `;` 之间按 `lines_between_statements` 插入空行

### 3.3 压缩规则

- 移除所有注释
- 移除所有不必要的空白和换行
- 关键字间保留单个空格
- 运算符两侧保留单个空格（或按需移除）
- 逗号后保留单个空格
- 分号后保留单个空格

### 3.4 语法校验规则

- 检查括号是否匹配
- 检查字符串是否闭合
- 检查块注释是否闭合
- 返回警告列表（如：可能存在问题的写法）

## 4. 关键设计决策

| 决策 | 理由 |
|------|------|
| 自定义分词器 | 避免依赖 `sqlparser`，减小 WASM 体积，保持零外部依赖 |
| 不支持自动换行 | 当前版本优先保证输出正确性，后续版本再实现 |
| 不修改语义 | 格式化仅调整空白，绝不改变 SQL 含义 |
| 方言影响关键字集合 | 不同数据库有不同的关键字，格式化需要正确识别 |

## 5. 非功能性需求

- WASM 体积 < 200 KB (gzip)
- 格式化 10 KB SQL 应在 50ms 内完成
- 支持日间/夜间模式（CSS 暗色主题）
- 响应式布局，支持移动端
- 所有文案中文
