# Tests

工作区级测试说明：

- crate 内部的单元测试放在各自 `src/` 中
- crate 级集成测试放在对应 crate 的 `tests/` 目录
- 根目录 `tests/` 主要用于跨 crate 集成场景

## 当前约束

- `tests/system/` 预留给系统性测试
- 根目录测试优先覆盖跨 crate 行为，而不是重复 crate 内部单元测试
- 每个系统性测试应尽量对应稳定 fixture 或明确的内联样例
- 根目录通过 `tests/system.rs` 挂载系统性测试模块
- 共享 fixture loader 与通用 helper 统一放在 `tests/common/mod.rs`

## 推荐新增顺序

1. `config -> app::bootstrap`
2. `domain + analyze`
3. `fetch -> domain`
4. `storage -> report`

## 当前已落地样例

- `crates/app/tests/config_to_bootstrap.rs`
  使用 `fixtures/system/config/` 下的最小配置样例，验证 `config -> app::bootstrap` 基础链路
- `tests/system/fetch_to_domain.rs`
  使用 `fixtures/system/fetch/` 和最小配置样例，验证 `config -> fetch -> domain`
  同时覆盖合法空 RSS、合法空热榜、非法 RSS、非法热榜、部分抓取成功后被后续错误整体中断，以及双来源同时为空的组合路径
- `tests/system/fetch_to_analyze.rs`
  使用 `fixtures/system/fetch/` 和最小配置样例，验证 `fetch -> domain -> analyze` 的跨 crate 组合结果
  同时覆盖双来源都为空、单来源为空但另一来源仍可分析、真实抓取输出上的同 rank 稳定排序、来源聚合在同计数时按 best_rank 排序、来源聚合在同计数不同时按 item_count 排序，以及 RSS / 热榜错误输入中断组合链路和“部分抓取成功但后续失败仍整体中断”的双向路径
- `tests/system/analyze_pipeline.rs`
  使用 `fixtures/system/analyze/` 和配置样例，验证 `config -> schedule -> analyze`
  同时覆盖同排名稳定排序、`analyze=false` 门控禁止、`rank = 0` 评分上界边界与空输入路径
- `tests/system/storage_to_report.rs`
  使用 `fixtures/system/storage/`、内存 SQLite 与固定运行上下文，验证 `storage -> report`
  同时覆盖有数据快照、空仓库到空报告、重复标题保留更优 rank、重复标题同 rank 仍只保留一条、同标题不同来源在相同 rank 下仍保持分离、乱序写入后稳定排序，以及同 rank 时按 `source_id + title` 稳定排序八条组合路径
- `tests/system/app_pipeline_modes.rs`
  使用 `fixtures/system/config/` 和全链路 fixture，验证 `config -> app pipeline` 在系统层的阶段组合行为
  当前已覆盖最小正向全链路、缺省 `schedule` 回退全链路、空来源全链路、单来源全链路、RSS-only 全链路、hotlist-only 全链路、跨午夜窗口内放行 / 窗口外阻断全链路、`collect=false` 时跳过损坏 source、窗口阻断时跳过损坏 source、`collect-only` 时仍传播损坏 source 错误、窗口放行时仍传播损坏 source 错误的路径、8 个 `collect/analyze/push` 布尔组合、窗口内放行 / 窗口外阻断，以及 RSS / 热榜两类 `fetch` 解析错误的向上传播路径
- `tests/system/config_schedule_errors.rs`
  使用 `fixtures/system/config/` 与 `fixtures/system/schedule/`，验证默认值回退、空时区、未知时区、越界小时、相等小时三类非法配置，以及白天 / 跨午夜窗口的成功与失败判定

## 默认新增流程

1. 先补 fixture
2. 再补 crate 内测试或系统性测试
3. 确认测试先失败
4. 再进入实现

不要把测试留到功能完成后再集中补。

## 参考模板

- [system-test-template.md](../docs/system-test-template.md)
- [tests/system/README.md](./system/README.md)
