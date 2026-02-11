# rs-sql-indent

A fast, opinionated SQL formatter CLI tool written in Rust.

![CI](https://github.com/takeokunn/rs-sql-indent/actions/workflows/ci.yml/badge.svg) ![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

## Features

- Token-stream based formatting -- no AST parsing, no external dependencies
- Two formatting styles: **Standard** (traditional) and **River** (leading comma)
- Uppercase/lowercase keyword control
- Zero-copy lexer for fast execution
- Reads from stdin -- works with pipes and redirects

## Install

```sh
cargo install --git https://github.com/takeokunn/rs-sql-indent.git
```

## Usage

Basic usage:

```sh
echo "select * from users where id = 1" | rs-sql-indent
```

Read from file:

```sh
rs-sql-indent < query.sql
```

### Options

- `--lowercase`: Output keywords in lowercase

  ```sh
  echo "SELECT * FROM users" | rs-sql-indent --lowercase
  ```

- `--style <standard|river>`: Choose formatting style (default: standard)

  ```sh
  echo "select id, name from users where active = true" | rs-sql-indent --style river
  ```

## Formatting Styles

**Standard** (default):

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

**River**:

```sql
SELECT id
       , name
       , email
  FROM users
 WHERE active = TRUE
 ORDER BY name
```

## Build

```sh
cargo build --release
```

## License

MIT -- see [LICENSE](LICENSE) for details.
