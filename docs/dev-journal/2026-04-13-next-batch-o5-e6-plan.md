# 下一批任务准备与执行结果：O5-1 / O5-2 / E6-1

## 基本信息

- 日期：2026-04-13
- 阶段：下一批任务准备
- 主题：O5-1、O5-2、E6-1 执行计划与结果
- 目标：记录本批任务的原始计划、实际执行结果和后续延伸方向

## 任务范围

本批任务覆盖下面 3 项：

1. `O5-1` 大输入规模稳定性测试
2. `O5-2` 多慢源 / 多失败源系统层测试
3. `E6-1` benchmark 结果文档化

本批次默认不做：

- 远程对象存储
- AI 分析 / MCP
- 新通知渠道
- 新 parser 平台

## O5-1：大输入规模稳定性测试

### 当前状态

- 已完成

### 目标

验证在更大输入规模下，当前 pipeline 的排序、去重、存储和输出仍保持稳定，不因为输入量提升出现结果抖动。

### 主要范围

- `tests/system/`
- `fixtures/system/analyze/` 或 `fixtures/system/storage/`
- 必要时补 `crates/app/tests/`

### 建议做法

- 新增一组大输入 fixture，避免用随机生成数据
- 优先验证稳定顺序，而不是仅验证 count
- 重点看：
  - 排序稳定性
  - 同 rank tie-break
  - 去重后结果一致性
  - 报告输出条目顺序

### 验证命令

- `cargo test --workspace`
- 如落在 `app` 侧：`cargo test -p trendradar-app`

### 完成标准

- 存在至少 1 条大输入规模系统测试
- 输出顺序和数量可稳定断言
- 不需要靠人工肉眼确认

### 实际结果

- 已新增根级系统测试 `tests/system/large_input_stability.rs`
- 覆盖大输入批量写入、最优 rank 去重、稳定排序和 JSON 报告顺序
- 已通过：
  - `cargo test --test system large_input_roundtrip_remains_stably_sorted_and_deduplicated -- --nocapture`

## O5-2：多慢源 / 多失败源系统层测试

### 当前状态

- 已完成

### 目标

把目前已经补到 crate 级和 app 集成级的 resilient 恢复能力，再往“更接近生产”的系统层推进一层。

### 主要范围

- `crates/app/tests/wave4_http_pipeline.rs`
- 必要时新增根级 `tests/system/` 用例

### 建议做法

- 基于 mock HTTP server 构造：
  - 1 个慢成功源
  - 1 到 2 个快速失败源
  - 1 个正常成功源
- 重点验证：
  - 成功源 retained results 保留
  - 最终 `stored_items` 顺序稳定
  - 报告输出可预测
  - 不因为并发完成顺序不同而抖动

### 验证命令

- `cargo test -p trendradar-app --test wave4_http_pipeline`
- `cargo test -p trendradar-app`

### 完成标准

- 至少新增 1 条系统层或近系统层测试
- 覆盖“多慢源 + 多失败源 + 成功源并存”的组合
- 能稳定复现并通过

### 实际结果

- 已新增根级系统测试 `tests/system/http_resilient_recovery.rs`
- 覆盖慢成功 RSS、慢成功热榜和多个失败源同时存在时的 retained results 恢复路径
- 已通过：
  - `cargo test --test system config_pipeline_retains_slow_successes_when_multiple_http_sources_fail -- --nocapture`

## E6-1：benchmark 结果文档化

### 当前状态

- 已完成

### 目标

把已经存在的 benchmark 入口和基线结果整理到对外文档里，让后续优化有明确参考点。

### 主要范围

- `crates/app/benches/pipeline_bench.rs`
- `README.md` 或 `docs/roadmap.md`
- 必要时补 `docs/extension-execution-plan.md`

### 建议做法

- 不重做 benchmark 体系
- 先整理现有基线：
  - fixture pipeline total
  - fetch/analyze/storage/report stage
- 明确哪些结果是稳定基线，哪些只是历史备注

### 验证命令

- `cargo bench --package trendradar-app --bench pipeline_bench`

### 完成标准

- 至少 1 处面向仓库使用者的文档写清 benchmark 入口
- 至少 1 处文档保留当前基线结果
- 不引入与实际 benchmark 代码不一致的描述

### 实际结果

- 已在 `README.md` 写清 `cargo bench --package trendradar-app --bench pipeline_bench` 入口
- 已在 `README.md` 和 `docs/roadmap.md` 写入当前可复用基线
- 当前保留的基线包括：
  - `pipeline_total/fixture_pipeline_minimal`
  - `pipeline_stage/fetch_fixture_sources`
  - `pipeline_stage/analyze_filter_rank_group`
  - `pipeline_stage/storage_in_memory_roundtrip`
  - `pipeline_stage/report_render_all_formats`

## 执行顺序建议

本批次按下面顺序推进并已完成：

1. `O5-1`
2. `O5-2`
3. `E6-1`

原因：

- 先补稳定性测试，进一步压实当前内核边界
- 再做 benchmark 文档化，避免文档化内容在测试补齐前再次变化

## 风险提醒

- 大输入 fixture 不宜过大，否则会降低测试稳定性和执行速度
- 并发 / 多慢源场景要控制好 sleep 时间，避免测试偶发抖动
- benchmark 结果文档化时应区分“当前基线”与“历史结果”，避免误导

## 实际产出

本批次完成后，已经得到：

- 2 条新的根级高价值系统测试
- 更清晰的 O5 收口边界
- 一份可被后续优化直接引用的 benchmark 文档基线

## 下一步建议

- 继续推进 O5，补“更复杂的并发慢源组合 + 输出稳定性”场景
- 继续推进 E6，补 Python 对比和更外显的展示方式
- 视协作需要决定是否进入 O6 贡献者文档
