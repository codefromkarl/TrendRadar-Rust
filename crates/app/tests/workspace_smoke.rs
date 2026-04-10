//! 工作区初始化后的基础冒烟测试。

use trendradar_app::{app_name, bootstrap};

#[test]
fn workspace_exposes_app_identity() {
    assert_eq!(app_name(), "trendradar-rust");
}

#[test]
fn bootstrap_loads_default_config() {
    assert!(bootstrap().is_ok());
}
