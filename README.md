# rs-sql-indent

A fast, opinionated SQL formatter CLI tool written in Rust.

![CI](https://github.com/takeokunn/rs-sql-indent/actions/workflows/ci.yml/badge.svg) ![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

## Features

- Token-stream based formatting -- no AST parsing, no external dependencies
- Four formatting styles: **Basic**, **Streamline**, **Aligned**, and **Dataops**
- Uppercase/lowercase keyword control
- Zero-copy lexer for fast execution
- Reads from stdin -- works with pipes and redirects
- [Playground](https://takeokunn.github.io/rs-sql-indent/) -- try it in your browser via WebAssembly

## Install

```sh
cargo install --git https://github.com/takeokunn/rs-sql-indent.git
```

## Usage

```sh
echo "select id, name from users where active = true" | rs-sql-indent
```

```sh
rs-sql-indent < query.sql
```

### Options

| Flag | Description |
|------|-------------|
| `--style <STYLE>` | Formatting style: `basic` (default), `streamline`, `aligned`, `dataops` |
| `--lowercase` | Output keywords in lowercase |
| `--uppercase` | Output keywords in uppercase (overrides style default) |

```sh
echo "select id, name from users" | rs-sql-indent --style aligned
echo "select id, name from users" | rs-sql-indent --style streamline --uppercase
```

## Formatting Styles

**Basic** (default) -- 4-space indent, uppercase, trailing comma:

```sql
SELECT
    id,
    name,
    email
FROM
    users
WHERE
    active = TRUE
ORDER BY
    name
```

**Streamline** -- 2-space indent, lowercase, trailing comma:

```sql
select
  id,
  name,
  email
from
  users
where
  active = true
order by
  name
```

**Aligned** -- right-aligned keywords, uppercase, leading comma:

```sql
SELECT id
       , name
       , email
  FROM users
 WHERE active = TRUE
 ORDER BY name
```

**Dataops** -- 4-space indent, uppercase, leading comma:

```sql
SELECT
    id
    , name
    , email
FROM
    users
WHERE
    active = TRUE
ORDER BY
    name
```

## Build

```sh
cargo build --release
```

## License

MIT -- see [LICENSE](LICENSE) for details.
