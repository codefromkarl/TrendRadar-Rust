//! Binary smoke test：验证 trendradar 可编译运行。

#![allow(clippy::panic)]

use std::process::Command;

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
