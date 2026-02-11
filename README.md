# rs-sql-indent

A CLI tool that formats SQL from stdin.

## Usage

```sh
echo "select * from users where id = 1" | rs-sql-indent
```

```sql
SELECT
  *
FROM
  users
WHERE
  id = 1
```

Read from a file:

```sh
rs-sql-indent < query.sql
```

## Install

```sh
cargo install --path .
```

## Build

```sh
cargo build --release
```

## License

MIT
