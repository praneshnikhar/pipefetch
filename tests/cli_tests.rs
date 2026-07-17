use std::process::Command;

fn pipefetch(args: &[&str]) -> std::process::Output {
    Command::new("cargo")
        .args(["run", "--"])
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run pipefetch")
}

#[test]
fn test_help() {
    let output = pipefetch(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pipefetch"));
    assert!(stdout.contains("get"));
    assert!(stdout.contains("post"));
    assert!(stdout.contains("delete"));
    assert!(stdout.contains("auth"));
    assert!(stdout.contains("run"));
    assert!(output.status.success());
}

#[test]
fn test_get_with_no_args_shows_error() {
    let output = pipefetch(&[]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("subcommand"));
    assert!(!output.status.success());
}

#[test]
fn test_help_auth() {
    let output = pipefetch(&["auth", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("add"));
    assert!(stdout.contains("list"));
    assert!(stdout.contains("remove"));
    assert!(output.status.success());
}

#[test]
fn test_help_run() {
    let output = pipefetch(&["run", "--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Run a YAML collection file"));
    assert!(output.status.success());
}

#[test]
fn test_dry_run_shows_method_and_url() {
    let output = pipefetch(&["get", "https://example.com/test", "--dry-run"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("GET"));
    assert!(stdout.contains("https://example.com/test"));
    assert!(output.status.success());
}

#[test]
fn test_dry_run_post_shows_body() {
    let output = pipefetch(&[
        "post",
        "https://example.com/data",
        r#"{"key":"val"}"#,
        "--dry-run",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("POST"));
    assert!(stdout.contains("key"));
    assert!(stdout.contains("val"));
    assert!(output.status.success());
}

#[test]
fn test_auth_add_list_remove() {
    let add = pipefetch(&[
        "auth",
        "add",
        "test-profile",
        "--auth-type",
        "bearer",
        "--value",
        "test-token",
    ]);
    assert!(add.status.success());

    let list = pipefetch(&["auth", "list"]);
    let stdout = String::from_utf8_lossy(&list.stdout);
    assert!(stdout.contains("test-profile"));
    assert!(list.status.success());

    let rm = pipefetch(&["auth", "remove", "test-profile"]);
    assert!(rm.status.success());

    let list2 = pipefetch(&["auth", "list"]);
    let stdout2 = String::from_utf8_lossy(&list2.stdout);
    assert!(!stdout2.contains("test-profile"));
}

#[test]
fn test_extract_flag_requires_url() {
    let output = pipefetch(&["get", "--extract", ".name"]);
    assert!(!output.status.success());
}

#[test]
fn test_unknown_command_fails() {
    let output = pipefetch(&["unknown-command-12345"]);
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("unrecognized"));
}

#[test]
fn test_run_missing_file_fails() {
    let output = pipefetch(&["run", "/tmp/nonexistent-file-12345.yaml"]);
    assert!(!output.status.success());
}
