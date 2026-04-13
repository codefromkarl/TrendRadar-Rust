#[path = "common/mod.rs"]
mod common;

#[path = "system/analyze_pipeline.rs"]
mod analyze_pipeline;

#[path = "system/fetch_to_domain.rs"]
mod fetch_to_domain;

#[path = "system/fetch_to_analyze.rs"]
mod fetch_to_analyze;

#[path = "system/storage_to_report.rs"]
mod storage_to_report;

#[path = "system/app_pipeline_modes.rs"]
mod app_pipeline_modes;

#[path = "system/config_schedule_errors.rs"]
mod config_schedule_errors;

#[path = "system/large_input_stability.rs"]
mod large_input_stability;

#[path = "system/http_resilient_recovery.rs"]
mod http_resilient_recovery;

#[path = "system/large_output_consistency.rs"]
mod large_output_consistency;

#[path = "system/remote_storage_contract.rs"]
mod remote_storage_contract;
