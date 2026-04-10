use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use std::fs::read_to_string;
use std::path::PathBuf;

pub fn system_fixture_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/system")
        .join(relative_path)
}

pub fn read_system_fixture(relative_path: &str) -> Result<String> {
    let path = system_fixture_path(relative_path);
    read_to_string(&path).with_context(|| format!("failed to read fixture {}", path.display()))
}

pub fn load_json_fixture<T>(relative_path: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let contents = read_system_fixture(relative_path)?;
    serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse json fixture {relative_path}"))
}
