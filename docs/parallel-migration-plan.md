# 并行迁移总方案

## 文档目标

这份文档用于承接环境准备阶段之后的功能迁移工作，给出 TrendRadar Rust 的并行迁移方式、任务切分边界、依赖关系和统一验收标准。

这不是对旧 Python 系统的逐文件翻译计划，而是基于当前设计文档整理出的 Rust 首版迁移执行方案。

## 设计依据

本方案以下列文档为准：

- `docs/architecture.md`：确定 crate 分层与依赖方向
- `docs/migration-strategy.md`：确定首版保留、延后、删除与重设计范围
- `docs/module-map.md`：确定按 crate 收敛的迁移主归属
- `docs/invariants.md`：确定不可打破的迁移边界
- `docs/api-contracts.md`：确定优先固化的输入输出契约
- `docs/system-test-template.md`：确定 fixture、快照与系统测试组织方式

如果后续实现与这些文档冲突，应优先更新设计文档，再调整迁移任务范围。

## 执行层文档入口

进入具体实现时，除了本总方案，还应配合下面三类执行层文档：

- [contracts/README.md](./contracts/README.md)
- [implementation/README.md](./implementation/README.md)
- [acceptance-matrix.md](./acceptance-matrix.md)

## 当前判断

结合仓库现状，当前已经具备进入并行迁移的基础，但只具备“最小起跑线”：

- workspace、验证命令、Git 规范和 AI 协作规则已经落地
- `config -> app::bootstrap` 的最小链路已经有真实测试
- `domain`、`schedule`、`analyze`、`fetch`、`storage`、`report` 仍主要是骨架
- 关键契约、fixture 体系和跨 crate 系统测试还没有进入功能级收敛阶段

这意味着并行迁移不能直接从“各自照着旧代码开写”开始，而应先固化共享契约和测试基线，再并行展开各 crate 的实现。

## 并行迁移原则

### 1. 先共享基线，再并行实现

并行迁移的第一步不是写业务逻辑，而是冻结足够稳定的共享基线：

- `domain` 最小统一模型
- `config` 最小配置结构
- 错误分类与错误消息边界
- fixture 命名与目录约定
- 首批系统性测试样例

没有这些共享基线，就会出现不同分支对“输入是什么、输出是什么、错误怎么报”的理解漂移，最后集成成本高于串行开发。

### 2. 按 crate 和契约切分，不按“大功能”切分

建议的并行任务单位应为下面之一：

- 一个 crate 的最小闭环
- 一个明确契约
- 一个独立 adapter
- 一组固定 fixture 与快照

不建议把“抓取 + 分析 + 存储 + 输出”混成一个任务，也不建议直接以“完整复刻某个旧模块”作为任务单位。

### 3. 共享接口先稳，内部实现后快

可被多个分支依赖的内容必须先稳定：

- 领域模型字段
- 配置字段
- trait 形状
- 错误分类
- JSON 输出结构

这些内容一旦进入并行开发，后续修改应按显式变更处理，并同步更新文档、fixture 和系统测试。

### 4. 纯逻辑优先于外部集成

根据迁移策略，优先顺序应保持为：

1. `domain`
2. `config`
3. `schedule`
4. `analyze`
5. `fetch`
6. `storage`
7. `report`
8. `app`

其中真正适合率先并行的是纯逻辑与稳定契约，外部依赖更强的抓取、存储和报告层应在共享模型稳定后接入。

## 推荐并行波次

### Wave 0：共享基线冻结

这是所有后续并行工作的共同前置阶段。

### 目标

- 固化最小领域模型和配置契约
- 固化错误分类与首版输出边界
- 建好首批 fixture、快照和系统测试入口

### 工作包

| 工作包 | 主责任 | 主要产出 | 完成信号 |
| --- | --- | --- | --- |
| `W0-domain-contract` | `domain` | `NewsItem`、`RssItem`、`RunContext`、共享错误分类 | 字段、序列化和错误边界进入文档与测试 |
| `W0-config-contract` | `config` | `AppConfig` 最小字段、默认值、校验规则 | 有合法与非法 fixture，错误可定位 |
| `W0-fixture-baseline` | `tests` / `fixtures` | 最小配置、最小输入、最小输出样例 | 样例可被至少一条真实测试消费 |
| `W0-acceptance-baseline` | `docs` / `app` | 首版系统测试路径与输出比对方法 | 文档和测试模板一致 |

### Wave 0 验收标准

- `docs/api-contracts.md` 中已补齐首批具体契约入口
- `docs/system-test-template.md` 或对应测试说明中已绑定真实 fixture
- `cargo test --workspace` 可覆盖至少一条非空系统链路
- 后续 crate 不再需要自行猜测统一模型和配置字段

只有 Wave 0 完成，后续并行才有意义。

### Wave 1：可独立推进的核心能力

Wave 1 应按“共享契约已稳定、内部逻辑可以独立推进”的标准切开。

### Lane A：纯逻辑迁移

| 工作包 | 主责任 | 前置依赖 | 主要产出 | 并行说明 |
| --- | --- | --- | --- | --- |
| `W1-schedule-core` | `schedule` | `W0-config-contract` | 调度输入模型、决策逻辑、fixture 驱动测试 | 可与 `analyze` 并行 |
| `W1-analyze-core` | `analyze` | `W0-domain-contract` | 过滤、聚合、排序、评分的稳定接口与快照 | 可与 `schedule` 并行 |

### Lane B：适配器迁移

| 工作包 | 主责任 | 前置依赖 | 主要产出 | 并行说明 |
| --- | --- | --- | --- | --- |
| `W1-fetch-rss` | `fetch` | `W0-domain-contract`、`W0-config-contract` | RSS 抓取到统一模型的最小链路 | 应独立子模块，避免与热榜抓取抢写 |
| `W1-fetch-hotlist` | `fetch` | `W0-domain-contract`、`W0-config-contract` | 一个热榜源到统一模型的最小链路 | 与 RSS 抓取并行，但必须拆子模块 |
| `W1-storage-local` | `storage` | `W0-domain-contract` | 本地存储 trait、去重策略、最小 SQLite 实现 | 可独立并行 |
| `W1-report-json` | `report` | `W0-domain-contract` | 首版 JSON 输出契约与快照 | 可独立并行 |

### Wave 1 验收标准

- `schedule` 与 `analyze` 至少各有一组 fixture 驱动测试
- `fetch` 至少打通一个 RSS 源和一个热榜源，且均能产出统一内部模型
- `storage` 已明确主键或去重规则，并有固定测试证明行为稳定
- `report` 的 JSON 输出有快照或结构断言，不依赖人工肉眼确认

### Wave 2：跨 crate 集成

当 Wave 1 的工作包都已有稳定接口后，再进入 `app` 编排集成。

### 目标

- 将配置、抓取、分析、调度、存储、输出串成最小运行闭环
- 把 crate 级完成转换为系统级完成

### 工作包

| 工作包 | 主责任 | 前置依赖 | 主要产出 |
| --- | --- | --- | --- |
| `W2-app-pipeline` | `app` | Wave 1 全部核心工作包 | 薄编排入口与阶段性 pipeline |
| `W2-system-fixture` | `app` / `tests` | `W2-app-pipeline` | 从配置到输出的系统性 fixture |
| `W2-parity-review` | `docs` / `tests` | `W2-system-fixture` | 与设计文档和旧系统目标边界的一致性检查 |

### Wave 2 验收标准

- 存在至少一条从配置到结构化输出的完整系统性测试
- `app` 没有吸入本应留在 `schedule`、`analyze`、`fetch`、`storage`、`report` 中的业务逻辑
- 系统输出可通过结构断言或快照稳定比较
- 文档、fixture、crate 接口和系统测试之间没有明显冲突

### Wave 2 当前结论（2026-04-11）

- `crates/app/tests/wave2_pipeline.rs` 已把 `config -> fetch -> analyze -> storage -> report` 最小闭环跑通
- `W2-parity-review` 已确认 `app` 当前只做配置校验、阶段决策串接、fixture 数据源编排、存储落库和报告调用
- 排序、来源聚合、抓取解析、存储去重和 JSON 渲染仍分别留在 `analyze`、`fetch`、`storage`、`report`
- 当前基线满足进入 Wave 3 的前置条件

### Wave 3：边界内扩展

Wave 3 不再验证“能不能串起来”，而是沿着既有 crate 边界扩能力和补证据。

### 目标

- 在不打破薄编排前提下扩展各核心 crate 的能力覆盖
- 把最小闭环推进为可持续扩写的系统测试基线

### 推荐工作包

| 工作包 | 主责任 | 前置依赖 | 主要产出 |
| --- | --- | --- | --- |
| `W3-schedule-window-cases` | `schedule` | `W2-parity-review` | 更丰富的时间窗口与决策 fixture |
| `W3-analyze-rule-cases` | `analyze` | `W2-parity-review` | 更高阶过滤、聚合、排序样例 |
| `W3-adapter-expansion` | `fetch` / `storage` / `report` | `W2-parity-review` | 更多来源、持久化边界和输出断言 |
| `W3-system-fixture-growth` | `app` / `tests` | 以上工作包逐步完成 | 新增系统 fixture，但 `app` 只增挂载与断言，不增业务规则 |

### Wave 3 进入信号

- `W2-parity-review` 已完成并留下审查记录
- `cargo test -p trendradar-app` 通过
- 验收矩阵已把 `app` 标记为具备最小 fixture pipeline 和系统测试入口

### Wave 3 当前证据（2026-04-11）

- `domain` 已补共享模型的 JSON fixture roundtrip 测试
- `schedule` 已补白天窗口、跨午夜窗口的允许 / 禁止判定，以及相等小时非法窗口与越界小时非法窗口
- `analyze` 已补 `rank = 0` 的评分边界、同排名排序、空输入和 `analyze=false` 门控
- `fetch` 已补 RSS / 热榜两类来源的空输入与错误路径
- `storage` 已补空仓库初始读取边界，并把去重语义提升到 `storage -> report` 系统测试
- `report` 已补空输入时 `meta + items` 结构稳定性断言
- `app` 已补最小正向全链路、空来源全链路、单来源全链路、RSS-only 全链路、hotlist-only 全链路、跨午夜窗口内放行 / 窗口外阻断全链路，以及 `collect=false` 时跳过损坏 source、窗口阻断时跳过损坏 source、`collect-only` 时仍传播损坏 source 错误、窗口放行时仍传播损坏 source 错误的路径和 `collect-only`、`analyze-only-empty`、`report-only-empty`、`push-only-empty`、`collect-and-report-no-analyze`、`analyze-without-report`、`disabled-all` 等系统 fixture；当前在存在 `schedule.window` 时也会按 `started_at + timezone` 消费时间窗口
- 根级 `tests/system/` 当前已有 62 条 richer cases，已补同排名排序、零排名边界、空输入、`analyze=false` 门控、空 RSS、空热榜、非法 RSS、非法热榜、双来源同时为空、`fetch -> domain` 的部分抓取成功后整体中断语义、`fetch -> analyze` 的成功 / 空组合 / 混合来源 / 错误链路，以及真实抓取输出上的同 rank 排序稳定性、来源聚合在同计数时按 best-rank 排序、来源聚合在同计数不同时按 item_count 排序、部分抓取成功后的整体中断双向语义、空仓库到空报告、去重后进入报告、相同 rank 重复写入仍去重、同标题不同来源在相同 rank 下仍保留分离、乱序写入后稳定排序、同 rank 时按 `source_id + title` 稳定排序、非法时间窗口，以及 `app` 的空来源全链路、单来源全链路、RSS-only 全链路、hotlist-only 全链路、跨午夜窗口内放行 / 窗口外阻断全链路、`collect=false` 时跳过损坏 source 的路径、窗口阻断时跳过损坏 source 的路径、`collect-only` 时仍传播损坏 source 错误的路径、窗口放行时仍传播损坏 source 错误的路径、窗口内放行 / 窗口外阻断
- 根级 `tests/system/app_pipeline_modes.rs` 已补最小正向全链路，并把 `collect/analyze/push` 的关键阶段矩阵收口到完整覆盖

### Wave 3 当前判断

- Wave 3 已不只是“可进入”，而是已经形成一批可重复验证的 crate 级 / 系统级边界样例
- `app` 的薄编排边界已经有足够多的系统证据，当前更适合把 richer cases 继续扩到非 `app` 链路，而不是把新规则塞回 `app`

## 模块级并行清单

下表给出当前仓库推荐的并行迁移任务单位。

| 模块 | 当前状态 | 推荐任务单位 | 主要前置条件 | 关闭条件 |
| --- | --- | --- | --- | --- |
| `domain` | 最小模型骨架 | 字段冻结、错误分类、序列化测试 | 无 | 契约稳定并被其他 crate 复用 |
| `config` | 最小配置骨架 | 配置分段、默认值、校验与错误语义 | `domain` 错误边界 | 配置 fixture 和错误 fixture 稳定 |
| `schedule` | 默认决策骨架 | 时间窗口解析、决策逻辑、纯函数测试 | `config` 调度字段契约 | 固定样例输出稳定 |
| `analyze` | 基础评分骨架 | 过滤、聚合、排序、快照测试 | `domain` 输入模型 | 样例结果稳定且可复查 |
| `fetch` | trait 骨架 | RSS adapter、热榜 adapter、归一化映射 | `domain` + `config` | 至少两个来源打通 |
| `storage` | trait 骨架 | 仓储接口、SQLite、去重策略 | `domain` | 写入读取在 fixture 下稳定 |
| `report` | JSON 骨架 | 首版 JSON 结构、元数据、快照 | `domain` + 输出契约 | 输出结构冻结 |
| `app` | 最小 bootstrap | pipeline 编排、错误穿透、系统测试挂载 | 以上模块完成最小闭环 | 完整系统样例通过 |

## 统一验收标准

每个迁移工作包在声称完成前，至少同时满足下面六项。

### 1. 契约验收

- 输入、输出、错误语义已经写入 `docs/api-contracts.md` 或对应模块文档
- 新增字段是否兼容、是否默认值、是否允许扩展有明确说明

### 2. 结构验收

- crate 依赖方向符合 `docs/architecture.md`
- 没有把业务逻辑偷偷塞进 `app`
- 没有引入与首版边界无关的 AI、MCP、通知矩阵或远程存储能力

### 3. 行为验收

- 至少有一条真实 fixture 或快照覆盖新能力
- 对固定输入能产生稳定、可复查的输出
- 错误场景可被断言，而不是只有“失败了”

### 4. 文档验收

- 相关迁移文档已同步更新
- 如果改动影响模块边界、契约、验证入口或完成定义，必须更新 `README.md`、`docs/module-map.md`、`docs/invariants.md`、`docs/api-contracts.md` 或开发日志中的相应入口

### 5. 命令验收

最低要求：

```bash
cargo fmt --all --check
cargo check --workspace --all-targets
cargo test --workspace
```

建议完整要求：

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo nextest run --workspace --all-features
cargo test --doc --workspace
```

### 6. 审查验收

- `git diff` 能清楚看出本次只解决一个明确问题
- 分支名和提交标题符合 `docs/git-workflow.md`
- 未经说明不混入无关重构

## 并行迁移的 Definition of Done

只有下面条件同时满足，才能认为 Rust 首版核心迁移达到可验收状态：

- `config -> fetch -> normalize -> analyze -> schedule -> storage -> report` 最小链路已打通
- 至少一个 RSS 源和一个热榜源被纳入统一模型
- 调度、分析、输出都有稳定 fixture 或快照
- 本地存储已具备可重复验证的最小实现
- `app` 只做编排，没有吞掉业务边界
- `just verify-basic` 持续通过
- 在工具齐全环境下，`just verify` 可以作为完整门禁
- 文档与实现对齐，评审者可仅通过仓库内文档和测试理解系统边界

## 建议执行顺序

如果现在要正式开始并行迁移，推荐按下面顺序开工：

1. 先完成 Wave 0，冻结共享契约和首批 fixture。
2. 并行启动 `schedule`、`analyze`、`fetch`、`storage`、`report` 的最小闭环。
3. `fetch` 内部按 RSS 与热榜 adapter 分子任务，避免多人改同一文件。
4. 每完成一个工作包就补文档、fixture 和验证证据，不等到集成阶段再回填。
5. 最后由 `app` 收口系统链路与系统性测试。

这样做的目标不是让更多人同时写代码，而是让并行分支在集成时仍然有同一套边界、契约和验收语言。
