use std::path::PathBuf;
use std::process::Command;

fn bin_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // deps
    path.pop(); // debug
    path.push("coh-validator");
    if !path.exists() {
        path.pop();
        path.pop();
        path.push("debug");
        path.push("coh-validator");
    }
    path
}

fn examples_dir() -> PathBuf {
    let mut path = std::env::current_dir().unwrap();
    if path.ends_with("coh-cli") {
        path.pop(); // crates
        path.pop(); // root
    }
    path.push("examples");
    path
}

fn run_cmd(args: &[String]) -> (i32, String) {
    let output = Command::new(bin_path())
        .args(args)
        .output()
        .expect("Failed to execute command");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).to_string(),
    )
}

#[test]
fn test_fixture_oracle_sweep() {
    let dir = examples_dir();

    let fixtures = vec![
        ("verify-micro", "micro_valid.json", 0, "ACCEPT"),
        ("verify-micro", "micro_invalid_policy.json", 1, "REJECT"),
        ("verify-micro", "micro_malformed_json.json", 2, "REJECT"),
        ("verify-chain", "chain_valid.jsonl", 0, "ACCEPT"),
        ("verify-chain", "chain_documented.jsonl", 0, "ACCEPT"),
        ("verify-chain", "chain_invalid_digest.jsonl", 1, "REJECT"),
        ("verify-chain", "chain_invalid_index.jsonl", 1, "REJECT"),
        ("verify-chain", "chain_invalid_state.jsonl", 1, "REJECT"),
        ("verify-slab", "slab_valid.json", 0, "ACCEPT"),
        ("verify-slab", "slab_invalid_policy.json", 1, "REJECT"),
        ("build-slab", "chain_valid.jsonl", 0, "SLAB_BUILT"),
        ("build-slab", "chain_invalid_digest.jsonl", 4, "REJECT"),
    ];

    for (cmd, file, expected_code, expected_text) in fixtures {
        let mut path = dir.clone();
        path.push(file);

        let mut args = vec![cmd.to_string(), path.to_str().unwrap().to_string()];

        // build-slab needs an --out arg
        if cmd == "build-slab" {
            let out_path = dir.join("temp_slab_fixture.json");
            args.push("--out".to_string());
            args.push(out_path.to_str().unwrap().to_string());
        }

        let (code, stdout) = run_cmd(&args);

        assert_eq!(
            code, expected_code,
            "Fixture {} failed code check. Stdout: {}",
            file, stdout
        );
        assert!(
            stdout.contains(expected_text),
            "Fixture {} failed text check. Stdout: {}",
            file,
            stdout
        );

        if cmd == "build-slab" && code == 0 {
            let _ = std::fs::remove_file(dir.join("temp_slab_fixture.json"));
        }
    }
}
