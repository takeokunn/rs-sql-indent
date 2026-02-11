use std::io::Write;
use std::process::{Command, Stdio};

fn run_with_stdin(input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rs-sql-indent"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn rs-sql-indent");

    if !input.is_empty() {
        child
            .stdin
            .take()
            .expect("failed to open stdin")
            .write_all(input.as_bytes())
            .expect("failed to write to stdin");
    }

    child.wait_with_output().expect("failed to wait on child")
}

#[test]
fn test_basic_formatting() {
    let output = run_with_stdin("select * from users");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SELECT"), "expected SELECT in: {stdout}");
    assert!(stdout.contains("FROM"), "expected FROM in: {stdout}");
    assert!(stdout.contains("users"), "expected users in: {stdout}");
}

#[test]
fn test_empty_input() {
    let output = run_with_stdin("");
    assert!(!output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("no SQL input provided"),
        "expected error message in stderr: {stderr}"
    );
}

#[test]
fn test_complex_query() {
    let output = run_with_stdin(
        "select u.name, o.total from users u join orders o on u.id = o.user_id where o.total > 100",
    );
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("JOIN"), "expected JOIN in: {stdout}");
    assert!(stdout.contains("WHERE"), "expected WHERE in: {stdout}");
}

#[test]
fn test_multiple_queries() {
    let output = run_with_stdin("select 1; select 2");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout.matches("SELECT").count();
    assert!(
        count >= 2,
        "expected SELECT at least twice, found {count} times in: {stdout}"
    );
}
