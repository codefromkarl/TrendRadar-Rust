# System Tests

这个目录用于放置跨 crate 或完整链路的系统性测试。

## 目标

- 用稳定输入覆盖跨模块行为
- 让迁移阶段的回归风险尽量前置暴露
- 为后续 parity 测试、snapshot 和覆盖率统计提供固定挂载点

## 推荐结构

- 一个测试文件对应一条链路或一类能力
- 对应 fixture 放在 `fixtures/system/` 下
- 如果输出适合做快照，统一放入 `tests/snapshots/`
- 工作区入口由 `tests/system.rs` 统一挂载
- 共享 helper 放在 `tests/common/mod.rs`

## 命名建议

- `config_to_bootstrap.rs`
- `fetch_to_domain.rs`
- `analyze_pipeline.rs`
- `storage_roundtrip.rs`
- `storage_to_report.rs`
- `app_pipeline_modes.rs`

## 新增测试时至少要回答的问题

- 这个测试保护的是哪条链路
- 输入样例从哪里来
- 期望输出如何比较
- 失败时如何快速定位是配置、抓取、分析还是输出层的问题

## 当前已落地链路

- `fetch_to_domain.rs`
  覆盖正常 RSS / 热榜归一化、合法空 RSS、合法空热榜、非法 RSS、非法热榜五条路径
- `fetch_to_analyze.rs`
  覆盖 `fetch -> domain -> analyze` 的成功、空输入和错误三类组合路径
- `analyze_pipeline.rs`
  覆盖允许分析、禁止分析、同排名稳定排序、零排名边界与空输入五条路径
- `storage_to_report.rs`
  覆盖有数据快照、空仓库到空报告、去重后进入报告、来源主键语义、写入顺序稳定性五条路径
- `app_pipeline_modes.rs`
  覆盖最小正向全链路、缺省 `schedule` 回退全链路、窗口内放行 / 窗口外阻断、8 个 `collect/analyze/push` 布尔组合，以及 RSS / 热榜两类来源的上游解析错误透传
- `config_schedule_errors.rs`
  覆盖默认值回退、非法配置和白天 / 跨午夜窗口成功与失败判定

## 默认要求

- 新增系统测试时，优先复用 `tests/common/mod.rs` 的 fixture loader
- 如果输出结构稳定且适合审查，优先补 `insta` snapshot
- 系统测试应在对应模块进入实现前就先落骨架，而不是收尾时再补
- 如果 `insta` 生成了 `.pending-snap` 或其他临时快照工件，应在同轮验证后清理，不把它们当作正式产物保留

## 参考模板

- [系统性测试模板](../../docs/system-test-template.md)

## 当前扩展说明

- 根级 `tests/system/` 当前共有 62 条系统测试
- `config_schedule_errors.rs` 当前已覆盖默认值回退、空时区、越界小时、相等小时，以及白天 / 跨午夜窗口的成功与失败判定
- `fetch_to_domain.rs` 当前已覆盖 RSS / 热榜两类来源的正常、空输入、错误路径、部分抓取成功后被后续错误整体中断，以及双来源同时为空的组合路径
- `fetch_to_analyze.rs` 当前已覆盖抓取结果进入排序和来源聚合的成功、真实抓取输出上的同 rank 稳定排序、来源聚合在同计数时按 best_rank 排序、来源聚合在同计数不同时按 item_count 排序、空输入、部分来源为空、错误路径，以及部分抓取成功后被后续错误整体中断的双向路径
- `analyze_pipeline.rs` 当前已覆盖门控允许、门控禁止、同排名稳定排序、零排名边界与空输入
- `storage_to_report.rs` 当前已覆盖有数据、空数据、去重、相同 rank 重复写入仍只保留一条、同标题不同来源在相同 rank 下仍保持分离、来源主键语义、同 rank 时按 `source_id + title` 稳定排序，以及写入顺序不影响输出顺序
- `app_pipeline_modes.rs` 当前已覆盖最小正向全链路、空来源全链路、单来源全链路、RSS-only 全链路、hotlist-only 全链路、跨午夜窗口内放行 / 窗口外阻断全链路、`collect=false` 时跳过损坏 source、窗口阻断时跳过损坏 source、`collect-only` 时仍传播损坏 source 错误、窗口放行时仍传播损坏 source 错误的路径、8 个 `collect/analyze/push` 布尔组合、窗口内放行 / 窗口外阻断，以及 RSS / 热榜两类来源的上游解析错误透传
