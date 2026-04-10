use std::process::Command;
use std::fs;
use std::path::PathBuf;

fn bin_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps
    path.pop(); // debug
    path.push("coh-validator");
    if !path.exists() {
        // Try one more level up if workspace root is different
        path.pop();
        path.pop();
        path.push("debug");
        path.push("coh-validator");
    }
    path
}

fn examples_dir() -> PathBuf {
    // CWD is crates/coh-cli
    let mut path = std::env::current_dir().unwrap();
    path.pop(); // crates
    path.pop(); // root
    path.push("examples");
    path
}

#[test]
fn test_cli_verify_micro_success() {
    let mut path = examples_dir();
    path.push("micro_valid.json");
    
    let output = Command::new(bin_path())
        .arg("verify-micro")
        .arg(path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0), "CLI failed with status {:?}. Stdout: {}, Stderr: {}", output.status.code(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("ACCEPT"));
}

#[test]
fn test_cli_verify_micro_reject() {
    let mut path = examples_dir();
    path.push("micro_invalid_policy.json");

    let output = Command::new(bin_path())
        .arg("verify-micro")
        .arg(path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("REJECT"));
}

#[test]
fn test_cli_verify_micro_malformed() {
    let mut path = examples_dir();
    path.push("micro_malformed_json.json"); // Fixed name

    let output = Command::new(bin_path())
        .arg("verify-micro")
        .arg(path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn test_cli_verify_chain_success() {
    let mut path = examples_dir();
    path.push("chain_valid.jsonl");

    let output = Command::new(bin_path())
        .arg("verify-chain")
        .arg(path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_cli_build_slab_success() {
    let mut in_path = examples_dir();
    in_path.push("chain_valid.jsonl");

    let mut out_path = examples_dir();
    out_path.push("slab_integration_temp.json");

    let output = Command::new(bin_path())
        .arg("build-slab")
        .arg(in_path)
        .arg("--out")
        .arg(&out_path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0), "CLI failed with status {:?}. Stdout: {}, Stderr: {}", output.status.code(), String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(out_path.exists());
    let _ = fs::remove_file(out_path);
}

#[test]
fn test_cli_build_slab_source_failure() {
    let mut in_path = examples_dir();
    in_path.push("chain_invalid_digest.jsonl");

    let output = Command::new(bin_path())
        .arg("build-slab")
        .arg(in_path)
        .arg("--out")
        .arg("examples/should_not_exist.json")
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(4));
}

#[test]
fn test_cli_verify_slab_success() {
    let mut path = examples_dir();
    path.push("slab_valid.json");

    let output = Command::new(bin_path())
        .arg("verify-slab")
        .arg(path)
        .output()
        .expect("Failed to execute command");

    assert_eq!(output.status.code(), Some(0));
}
