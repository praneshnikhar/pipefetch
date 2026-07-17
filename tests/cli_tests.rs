use std::process::Command;

#[test]
fn test_help() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run pipefetch --help");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("pipefetch"));
    assert!(stdout.contains("get"));
    assert!(stdout.contains("post"));
    assert!(stdout.contains("delete"));
    assert!(output.status.success());
}

#[test]
fn test_get_with_no_args_shows_error() {
    let output = Command::new("cargo")
        .args(["run", "--"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("Failed to run pipefetch");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error") || stderr.contains("subcommand"));
    assert!(!output.status.success());
}
