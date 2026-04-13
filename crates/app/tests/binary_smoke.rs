//! Binary smoke test：验证 trendradar 可编译运行。

#![allow(clippy::panic)]

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn config_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system/config")
        .join(name)
}

/// trendradar binary 路径。
fn trendradar_bin() -> String {
    env!("CARGO_BIN_EXE_trendradar").to_owned()
}

#[test]
fn binary_runs_with_help_flag() {
    let output = Command::new(trendradar_bin())
        .arg("--help")
        .output()
        .unwrap_or_else(|e| panic!("failed to run trendradar binary: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "--help should exit 0");
    assert!(
        stdout.contains("trendradar"),
        "help output should mention trendradar"
    );
    assert!(
        stdout.contains("--config"),
        "help output should mention --config"
    );
    assert!(stdout.contains("--db"), "help output should mention --db");
    assert!(
        stdout.contains("--verbose"),
        "help output should mention --verbose"
    );
    assert!(
        stdout.contains("--dry-run"),
        "help output should mention --dry-run"
    );
}

#[test]
fn binary_returns_config_exit_code_for_invalid_config() {
    let output = Command::new(trendradar_bin())
        .arg("--config")
        .arg(config_fixture_path("invalid-empty-timezone.json"))
        .output()
        .unwrap_or_else(|e| panic!("failed to run trendradar binary: {e}"));

    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn binary_returns_storage_exit_code_for_invalid_db_path() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| panic!("system clock error: {e}"))
        .as_nanos();
    let temp_dir = std::env::temp_dir().join(format!("trendradar-binary-smoke-{unique}"));
    std::fs::create_dir_all(&temp_dir).unwrap_or_else(|e| panic!("create temp dir failed: {e}"));

    let output = Command::new(trendradar_bin())
        .arg("--config")
        .arg(config_fixture_path("minimal-valid-http.json"))
        .arg("--db")
        .arg(&temp_dir)
        .output()
        .unwrap_or_else(|e| panic!("failed to run trendradar binary: {e}"));

    assert_eq!(output.status.code(), Some(3));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn binary_runs_with_version_flag() {
    let output = Command::new(trendradar_bin())
        .arg("--version")
        .output()
        .unwrap_or_else(|e| panic!("failed to run trendradar binary: {e}"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "--version should exit 0");
    assert!(
        stdout.contains("trendradar"),
        "version output should mention trendradar"
    );
}

#[test]
fn binary_dry_run_succeeds_with_fixture_config() {
    let config_path = format!(
        "{}/../../fixtures/system/config/minimal-valid.json",
        env!("CARGO_MANIFEST_DIR")
    );

    let output = Command::new(trendradar_bin())
        .arg("--config")
        .arg(&config_path)
        .arg("--dry-run")
        .output()
        .unwrap_or_else(|e| panic!("failed to run trendradar binary: {e}"));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "dry-run should succeed, stderr: {stderr}"
    );
}
