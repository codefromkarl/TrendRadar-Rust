use anyhow::Result;
use serde::Deserialize;

use crate::common::load_json_fixture;

#[derive(Debug, Deserialize)]
struct RemoteLayoutFixture {
    backend: String,
    layout_version: u32,
    index_object: String,
    shard_pattern: String,
    read_order: Vec<String>,
    dedupe_key: Vec<String>,
    sort_order: Vec<String>,
}

#[test]
fn remote_storage_layout_fixture_defines_minimal_s3_contract() -> Result<()> {
    let fixture: RemoteLayoutFixture = load_json_fixture("storage/remote-layout-s3.json")?;

    assert_eq!(fixture.backend, "s3");
    assert_eq!(fixture.layout_version, 1);
    assert_eq!(fixture.index_object, "trendradar/index/latest.json");
    assert!(fixture.index_object.ends_with(".json"));
    assert_eq!(
        fixture.shard_pattern,
        "trendradar/shards/{date}/{source}.json"
    );
    assert!(fixture.shard_pattern.contains("{date}"));
    assert!(fixture.shard_pattern.contains("{source}"));
    assert_eq!(fixture.read_order, vec!["index", "shards"]);
    assert_eq!(fixture.dedupe_key, vec!["source_id", "title"]);
    assert_eq!(fixture.sort_order, vec!["rank", "source_id", "title"]);
    Ok(())
}
