#![allow(clippy::expect_used, clippy::panic, missing_docs)]

use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn config_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system/config")
        .join(name)
}

fn trendradar_bin() -> String {
    env!("CARGO_BIN_EXE_trendradar").to_owned()
}

fn temp_file(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|e| panic!("system time should be after unix epoch: {e}"))
        .as_nanos();
    std::env::temp_dir().join(format!("trendradar-{name}-{nanos}"))
}

#[test]
fn binary_writes_structured_run_log_and_keeps_stdout_json_clean() {
    let db_path = temp_file("run-log.db");
    let log_path = temp_file("run-log.json");

    let output = Command::new(trendradar_bin())
        .arg("--config")
        .arg(config_fixture_path("minimal-valid.json"))
        .arg("--db")
        .arg(&db_path)
        .arg("--output")
        .arg("json")
        .arg("--run-log")
        .arg(&log_path)
        .output()
        .unwrap_or_else(|e| panic!("failed to run trendradar binary: {e}"));

    assert!(output.status.success(), "binary should exit 0");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed_stdout: serde_json::Value =
        serde_json::from_str(&stdout).expect("stdout should be clean json");
    assert!(parsed_stdout.get("meta").is_some());

    let log_contents = std::fs::read_to_string(&log_path).expect("run log should exist");
    let parsed_log: serde_json::Value =
        serde_json::from_str(&log_contents).expect("run log should be valid json");
    assert!(parsed_log.get("decision").is_some());
    assert!(parsed_log.get("counts").is_some());
    assert!(parsed_log.get("collected_items").is_some());
    assert!(parsed_log.get("filtered_items").is_some());
    assert!(parsed_log.get("ranked_items").is_some());
    assert!(parsed_log.get("source_summaries").is_some());
    assert!(parsed_log.get("domain_summaries").is_some());
    assert!(parsed_log.get("deduped_items").is_some());

    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(log_path);
}
