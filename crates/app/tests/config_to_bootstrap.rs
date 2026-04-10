//! 配置 fixture 到应用 bootstrap 的系统性测试。

use std::{fs, path::PathBuf};

use trendradar_app::bootstrap_with_config;
use trendradar_config::load_config_from_json_str;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/system/config")
        .join(name)
}

#[test]
fn valid_config_fixture_bootstraps_successfully() {
    let fixture = fs::read_to_string(fixture_path("minimal-valid.json"))
        .expect("config fixture should be readable");

    let config = load_config_from_json_str(&fixture).expect("fixture should load as config");

    assert_eq!(config.timezone, "Asia/Shanghai");
    assert_eq!(config.platforms, vec!["weibo", "zhihu"]);
    assert!(bootstrap_with_config(&config).is_ok());
}

#[test]
fn invalid_config_fixture_is_rejected() {
    let fixture = fs::read_to_string(fixture_path("invalid-empty-timezone.json"))
        .expect("config fixture should be readable");

    let error = load_config_from_json_str(&fixture).expect_err("fixture should be rejected");

    assert_eq!(
        error.to_string(),
        "invalid config: timezone must not be empty"
    );
}
