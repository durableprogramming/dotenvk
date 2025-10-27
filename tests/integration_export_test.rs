// Integration tests for the export command to validate bash compatibility
// These tests verify that the exported bash code can actually be parsed and executed by bash

use assert_cmd::Command;
use std::fs;
use std::process::Command as StdCommand;
use tempfile::TempDir;

#[test]
fn test_export_bash_simple_values_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        "SIMPLE=value\nNUMERIC=12345\nUNDERSCORE=under_score\n",
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    // Verify bash can parse it
    verify_bash_parseable(&bash_output);
}

#[test]
fn test_export_bash_with_special_characters_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"SPACES="value with spaces"
DOLLAR="$HOME/path"
BACKTICK="`echo test`"
QUOTES="She said \"hello\""
NEWLINE="line1\nline2"
EXCLAMATION="alert!"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    // Verify bash can parse it
    verify_bash_parseable(&bash_output);

    // Verify values are properly escaped
    assert!(bash_output.contains(r#""value with spaces""#));
    assert!(bash_output.contains(r"\$HOME"));
    assert!(bash_output.contains(r"\`echo test\`"));
}

#[test]
fn test_export_bash_command_injection_prevention() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"INJECTION1="$(whoami)"
INJECTION2="`whoami`"
INJECTION3="; rm -rf /"
INJECTION4="| cat /etc/passwd"
INJECTION5="&& ls -la"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    // Verify bash can parse it without executing injected commands
    verify_bash_parseable(&bash_output);

    // Verify dangerous characters are escaped
    assert!(bash_output.contains(r"\$"));
    assert!(bash_output.contains(r"\`"));
}

#[test]
fn test_export_bash_url_values_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"API_URL="https://api.example.com/v1?key=value&token=abc123"
WEBHOOK="http://example.com/webhook?token=abc123"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);
}

#[test]
fn test_export_bash_json_values_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"JSON_CONFIG="{\"name\":\"test\",\"value\":123}"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);
    // Verify quotes are escaped
    assert!(bash_output.contains(r#"\""#));
}

#[test]
fn test_export_bash_database_urls_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"DATABASE_URL="postgresql://user:p@ss$word@localhost:5432/dbname?sslmode=require"
REDIS_URL="redis://:p@ssw0rd!@localhost:6379/0"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);

    // Verify special characters in passwords are escaped
    assert!(bash_output.contains(r"\$")); // $ in password
    assert!(bash_output.contains(r"\!")); // ! in password
}

#[test]
fn test_export_bash_unicode_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"EMOJI="Hello 👋 World 🌍"
UNICODE="Ñoño en español"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);
}

#[test]
fn test_export_bash_complex_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"MULTILINE="line1\nline2\nline3"
TABS="col1\tcol2\tcol3"
MIXED="Hello\nWorld\tTest"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);
}

#[test]
fn test_export_bash_wildcards_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"GLOB1="*.txt"
GLOB2="file?.dat"
GLOB3="[a-z]*"
GLOB4="{a,b,c}"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);
}

#[test]
fn test_export_bash_paths_parseable() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"UNIX_PATH="/path/to/My Documents/file.txt"
WINDOWS_PATH="C:\Program Files\MyApp\bin"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);
    assert!(bash_output.contains(r"\\")); // Windows paths should have escaped backslashes
}

#[test]
fn test_export_bash_empty_values() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(&env_file, "EMPTY=\nSPACES=\"   \"\n").unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);
}

#[test]
fn test_export_bash_comprehensive_special_chars() {
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");

    fs::write(
        &env_file,
        r#"ALL_SYMBOLS="!@#$%^&*()_+-=[]{}|;:,.<>?"
COMPLEX="$USER: \"Hello!\", path=`pwd`, cmd;ls&echo"
REDIRECT="output>file<input"
SQL="SELECT * FROM users WHERE name = \"John\" AND age > 25;"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());
    let bash_output = String::from_utf8_lossy(&output.stdout);

    verify_bash_parseable(&bash_output);

    // Verify critical escaping
    assert!(bash_output.contains(r"\$"));
    assert!(bash_output.contains(r"\`"));
    assert!(bash_output.contains(r"\!"));
    assert!(bash_output.contains(r#"\""#));
}

#[test]
fn test_export_bash_roundtrip_values() {
    // Test that values exported and then sourced preserve their meaning
    let temp_dir = TempDir::new().unwrap();
    let env_file = temp_dir.path().join(".env");
    let export_file = temp_dir.path().join("exports.sh");

    fs::write(
        &env_file,
        r#"TEST_VAR="value with $special chars!"
MULTILINE="line1\nline2"
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("durable-appconfig-dotenv").unwrap();
    let output = cmd
        .arg("--file")
        .arg(&env_file)
        .arg("export")
        .arg("--format")
        .arg("bash")
        .output()
        .unwrap();

    assert!(output.status.success());

    // Write export output to a file
    fs::write(&export_file, &output.stdout).unwrap();

    // Verify bash can source it
    let bash_check = StdCommand::new("bash")
        .arg("-c")
        .arg(format!("source {:?} && echo $TEST_VAR", export_file))
        .output();

    if let Ok(result) = bash_check {
        assert!(result.status.success(), "Bash failed to source export file");
    }
}

/// Helper function to verify that bash can parse the export statements
/// without executing them (using bash -n for syntax check)
fn verify_bash_parseable(bash_output: &str) {
    let temp_dir = TempDir::new().unwrap();
    let script_file = temp_dir.path().join("test_exports.sh");

    // Write the output to a temporary file
    fs::write(&script_file, bash_output).unwrap();

    // Use bash -n to check syntax without executing
    let output = StdCommand::new("bash")
        .arg("-n")
        .arg(&script_file)
        .output()
        .expect("Failed to execute bash -n");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "Bash syntax check failed for output:\n{}\n\nBash error:\n{}",
            bash_output, stderr
        );
    }
}

// Copyright (c) 2025 Durable Programming, LLC. All rights reserved.
