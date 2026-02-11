use std::fs;
use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::*;

fn cmd() -> Command {
    Command::cargo_bin("rs-sql-indent").unwrap()
}

#[test]
fn test_basic_formatting() {
    cmd()
        .write_stdin("select * from rockets")
        .assert()
        .success()
        .stdout(
            r#"SELECT
    *
FROM
    rockets
"#,
        );
}

#[test]
fn test_empty_input() {
    cmd()
        .write_stdin("")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no SQL input provided"));
}

#[test]
fn test_complex_query() {
    cmd()
        .write_stdin("select g.luminosity, s.velocity from galaxies g join stars s on g.cluster_id = s.galaxy_ref where s.velocity > 9000")
        .assert()
        .success()
        .stdout(
            r#"SELECT
    g.luminosity,
    s.velocity
FROM
    galaxies g
JOIN stars s
    ON g.cluster_id = s.galaxy_ref
WHERE
    s.velocity > 9000
"#,
        );
}

#[test]
fn test_multiple_queries() {
    cmd()
        .write_stdin("select 1; select 2")
        .assert()
        .success()
        .stdout(
            r#"SELECT
    1;

SELECT
    2
"#,
        );
}

#[test]
fn test_lowercase_flag() {
    cmd()
        .arg("--lowercase")
        .write_stdin("SELECT power_level FROM potions")
        .assert()
        .success()
        .stdout(
            r#"select
    power_level
from
    potions
"#,
        );
}

#[test]
fn test_version_flag() {
    cmd()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("rs-sql-indent"));
}

#[test]
fn test_style_aligned() {
    cmd()
        .args(["--style", "aligned"])
        .write_stdin("select radius, color from crystals")
        .assert()
        .success()
        .stdout(
            r#"SELECT radius
       , color
  FROM crystals
"#,
        );
}

#[test]
fn test_style_aligned_with_lowercase() {
    cmd()
        .args(["--style", "aligned", "--lowercase"])
        .write_stdin("select altitude from volcanoes where dormant = true")
        .assert()
        .success()
        .stdout(
            r#"select altitude
  from volcanoes
 where dormant = true
"#,
        );
}

#[test]
fn test_style_basic_explicit() {
    cmd()
        .args(["--style", "basic"])
        .write_stdin("select wingspan from dragons")
        .assert()
        .success()
        .stdout(
            r#"SELECT
    wingspan
FROM
    dragons
"#,
        );
}

#[test]
fn test_style_streamline() {
    cmd()
        .args(["--style", "streamline"])
        .write_stdin("select wingspan from dragons")
        .assert()
        .success()
        .stdout(
            r#"select
  wingspan
from
  dragons
"#,
        );
}

#[test]
fn test_style_streamline_with_uppercase() {
    cmd()
        .args(["--style", "streamline", "--uppercase"])
        .write_stdin("select id from users")
        .assert()
        .success()
        .stdout(
            r#"SELECT
  id
FROM
  users
"#,
        );
}

#[test]
fn test_style_dataops() {
    cmd()
        .args(["--style", "dataops"])
        .write_stdin("select velocity, color from rockets")
        .assert()
        .success()
        .stdout(
            r#"SELECT
    velocity
    , color
FROM
    rockets
"#,
        );
}

#[test]
fn test_style_invalid_is_error() {
    cmd()
        .args(["--style", "foobar"])
        .write_stdin("select something from somewhere")
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}

#[test]
fn test_fixtures() {
    let fixture_dir = Path::new("tests/fixtures");
    let mut entries: Vec<_> = fs::read_dir(fixture_dir)
        .expect("failed to read fixtures directory")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
        .collect();
    entries.sort_by_key(|e| e.path());

    assert!(!entries.is_empty(), "no fixture .sql files found");

    for entry in entries {
        let path = entry.path();
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let expected_path = path.with_extension("expected");

        assert!(
            expected_path.exists(),
            "missing .expected file for fixture: {}",
            stem
        );

        let input = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", path.display(), e));
        let expected = fs::read_to_string(&expected_path)
            .unwrap_or_else(|e| panic!("failed to read {}: {}", expected_path.display(), e));

        // Parse naming convention: {style}__{description} or {style}__{description}__lower
        let parts: Vec<&str> = stem.splitn(3, "__").collect();
        assert!(
            parts.len() >= 2,
            "fixture filename must match {{style}}__{{name}} convention: {}",
            stem
        );

        let style = parts[0];
        let lowercase = parts.last().is_some_and(|&p| p == "lower");

        let mut cmd = Command::cargo_bin("rs-sql-indent").unwrap();
        cmd.args(["--style", style]);
        if lowercase {
            cmd.arg("--lowercase");
        }

        // CLI uses println! which adds trailing newline
        let expected_stdout = format!("{}\n", expected.trim_end());

        cmd.write_stdin(input)
            .assert()
            .success()
            .stdout(predicates::ord::eq(expected_stdout).normalize());
    }
}
