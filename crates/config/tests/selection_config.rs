#![allow(clippy::expect_used, missing_docs)]

use trendradar_config::load_config_from_json_str;

#[test]
fn selection_config_loads_from_json() {
    let input = r#"{
        "timezone": "Asia/Shanghai",
        "selection": {
            "high_rank_fallback_max_rank": 3,
            "min_items_per_source": 2,
            "min_items_per_domain": 1
        }
    }"#;

    let config = load_config_from_json_str(input).expect("config should parse");
    assert_eq!(config.selection.high_rank_fallback_max_rank, Some(3));
    assert_eq!(config.selection.min_items_per_source, Some(2));
    assert_eq!(config.selection.min_items_per_domain, Some(1));
}

#[test]
fn missing_selection_uses_default_values() {
    let input = r#"{"timezone":"Asia/Shanghai"}"#;
    let config = load_config_from_json_str(input).expect("config should parse");
    assert_eq!(config.selection.high_rank_fallback_max_rank, None);
    assert_eq!(config.selection.min_items_per_source, None);
    assert_eq!(config.selection.min_items_per_domain, None);
}
