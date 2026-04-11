# 系统性测试模板

## 文档目标

这份文档用于给当前仓库提供一个可直接复用的系统性测试模板。

这里的“系统性测试”不是简单单元测试，而是围绕一个完整链路或一个跨 crate 行为做验证：

- 有明确输入
- 有稳定 fixture
- 有可比较输出
- 有固定验证命令

同时，这份文档也定义一个默认工作流：

1. 先写契约和验收项
2. 再补 fixture
3. 再写失败测试
4. 最后才进入实现

如果一个工作包还没有 fixture、测试入口和验证命令，就不应视为“开始实现完成”。

## 适用范围

当前仓库里的系统性测试，优先覆盖这些场景：

- 配置加载到运行上下文的链路
- 抓取结果到统一领域模型的归一化
- 调度、过滤、聚合、排序的组合逻辑
- 存储写入后的结果一致性
- 结构化输出和快照结果

## 当前已绑定的真实样例

在进入完整功能迁移前，当前仓库已经有一条真实、可重复执行的最小系统链路：

- 链路：`config -> app::bootstrap`
- 合法 fixture：`fixtures/system/config/minimal-valid.json`
- 非法 fixture：`fixtures/system/config/invalid-empty-timezone.json`
- 测试入口：`crates/app/tests/config_to_bootstrap.rs`
- 验证命令：`cargo test --workspace`

后续新增系统性测试时，建议先模仿这条链路，再扩展到抓取、分析、存储和输出组合测试。

## 测试文件组织

建议按下面方式组织：

```text
src/
  lib.rs
tests/
  common/
    mod.rs
  system.rs
  system/
    README.md
    config_to_bootstrap.rs
    fetch_to_domain.rs
    analyze_pipeline.rs
    storage_to_report.rs
fixtures/
  system/
    config/
    fetch/
    analyze/
    storage/
    report/
tests/snapshots/
```

说明：

- 根工作区通过 `src/lib.rs` 提供一个最小 test harness
- `tests/system.rs` 作为系统性测试入口，挂载 `tests/system/*.rs`
- `tests/common/mod.rs` 放共享 fixture loader、断言 helper、固定时间构造等通用能力
- fixture 继续统一放在 `fixtures/system/` 下，避免散落到各 crate

## 一个基础系统测试的最小构成

每个系统测试建议都具备这 5 部分：

1. 测试目标：要验证哪条链路。
2. 输入 fixture：稳定的 JSON、YAML、文本或内联样例。
3. 执行步骤：调用哪些 crate 或公开函数。
4. 期望结果：结构化断言或快照。
5. 验证入口：本地和 CI 用哪条命令跑。

## 默认流程

新增一个模块或工作包时，默认按下面顺序推进：

1. 在契约文档中写明输入、输出、错误语义
2. 在验收矩阵中补 fixture / 测试入口 / 最低验证命令
3. 先补最小 fixture，必要时同时补合法与非法样例
4. 先写 crate 内测试，再写系统性测试
5. 确认测试先失败，再补实现
6. 实现完成后，回填快照或更完整的结构断言
7. 最后跑 `cargo test --workspace`，必要时再跑 `just verify`

不要采用“先把功能写完，最后再集中补测试”的方式。

## Rust 测试模板

```rust
//! 示例：系统性测试模板

use anyhow::Result;

#[test]
fn system_case_name() -> Result<()> {
    // 1. 准备 fixture
    // let input = crate::common::load_json_fixture("...")?;

    // 2. 调用跨 crate 链路
    // let config = trendradar_config::...;
    // let output = trendradar_fetch::...;

    // 3. 做结构化断言
    // assert_eq!(output.xxx, expected.xxx);

    // 4. 如有必要，做 snapshot
    // insta::assert_json_snapshot!(output);

    Ok(())
}
```

## 推荐测试分层

- 单元测试：
  放在 crate 的 `src/` 内，验证纯函数、局部规则、错误分支
- crate 级集成测试：
  放在各 crate 自己的 `tests/` 目录，验证一个 crate 对外暴露的稳定接口
- 工作区系统测试：
  放在根 `tests/system/*.rs`，验证跨 crate 链路和固定 fixture

判断标准：

- 如果只保护一个 crate 内部规则，用单元测试
- 如果要验证 crate 对外 API，用 crate 集成测试
- 如果涉及两个及以上 crate，用系统性测试

## 测试用例模板

新增系统性测试时，建议先写一段 case 描述，再实现代码：

```md
## 用例名称

### 目标

验证哪条业务链路。

### 输入

- fixture 文件：
- 来源：
- 是否脱敏：

### 执行步骤

1. 加载配置或输入样例
2. 调用对应 crate / 函数
3. 收集输出

### 断言

- 结构断言：
- 数值断言：
- 错误断言：
- snapshot：

### 覆盖的风险

- 这条测试在防止什么回归
```

## 命名建议

- 测试文件名使用“链路”或“能力”命名
- fixture 目录按能力分组，不按临时人名或日期命名
- snapshot 名称与测试函数保持一致

## 共享 helper 约束

- fixture 读取优先通过 `tests/common/mod.rs` 的共享 loader 完成
- 通用 helper 只放“读取样例、固定时间、共享断言”这类稳定能力
- 不要把业务逻辑偷偷塞进 helper，否则系统测试会失去可读性

## Snapshot 使用建议

- 对顶层 JSON 输出、聚合结果、系统链路结果，优先考虑 `insta`
- 对数值小、结构稳定的结果，可以用 inline snapshot
- 当输出结构变大、需要多人审查时，再切到 `tests/snapshots/` 文件快照
- snapshot 更新必须和 fixture / 契约更新一起出现，不能只改 snapshot 不解释原因

## 推荐验证命令

在当前仓库中，系统性测试至少应纳入下面这些入口：

```bash
cargo test --workspace
cargo nextest run --workspace --all-features
cargo test --doc --workspace
```

如果后续引入 snapshot 或覆盖率，还应补：

```bash
cargo llvm-cov nextest --workspace --all-features
```

## 当前阶段的最低执行标准

在首版迁移阶段，一个 crate 要被视为“具备基础系统验证”，至少应满足：

- 有一个跨函数或跨 crate 的系统性测试样例
- 有对应 fixture 或明确的内联样例
- 能用固定命令重复执行
- 能从失败结果中看出是哪条链路出错
- 验收矩阵已标出测试入口和最低验证命令

## 当前已落地样例

当前仓库已经不只一条真实样例，而是形成了几条可复用链路：
- `crates/app/tests/config_to_bootstrap.rs`
  先把 `fixture -> config -> app` 的最小入口跑通
- `tests/system/fetch_to_domain.rs`
  覆盖 `fetch -> domain` 的正常、空输入、错误路径与双来源同时为空
- `tests/system/analyze_pipeline.rs`
  覆盖 `config -> schedule -> analyze` 的允许门控、禁止门控、同排名排序、零排名边界与空输入
- `tests/system/storage_to_report.rs`
  覆盖 `storage -> report` 的有数据、空数据、去重、来源主键和排序稳定性
- `tests/system/app_pipeline_modes.rs`
  覆盖 `app` 全链路的最小正向闭环、空来源闭环、单来源闭环、RSS-only 闭环、hotlist-only 闭环、跨午夜窗口内放行 / 窗口外阻断闭环、`collect=false` 时跳过损坏 source、窗口阻断时跳过损坏 source、`collect-only` 时仍传播损坏 source 错误、窗口放行时仍传播损坏 source 错误的路径、8 个 `collect/analyze/push` 布尔组合和上游解析错误透传
- `tests/system/config_schedule_errors.rs`
  覆盖配置 / 调度边界的默认值回退、成功窗口判定与错误路径

## 下一步建议

- 当前根级系统测试已经覆盖：
  `fetch -> domain` 的正常、空输入、错误路径与双来源同时为空，
  `config -> schedule` 的默认值回退、窗口成功 / 失败与错误路径，
  `config -> schedule -> analyze` 的允许门控、禁止门控、tie-break、零排名和空输入，
  `storage -> report` 的有数据、空数据、去重、来源主键和排序稳定性，
  以及 `app` 全链路的最小正向闭环、空来源闭环、单来源闭环、RSS-only 闭环、hotlist-only 闭环、跨午夜窗口内放行 / 窗口外阻断闭环、`collect=false` 时跳过损坏 source、窗口阻断时跳过损坏 source、`collect-only` 时仍传播损坏 source 错误、窗口放行时仍传播损坏 source 错误的路径、8 个 `collect/analyze/push` 布尔组合和上游解析错误透传。
- 继续补更多跨 crate richer cases，而不是只停留在 crate 内边界测试
- `app` 的阶段门控、来源形态、动态窗口和错误传播矩阵已经比较完整，下一批 richer cases 更适合优先转向非 `app` 链路
- 优先扩 `fetch -> analyze`、`storage -> report` 或后续新增链路中的系统级错误路径、空输入路径和结构快照
- 在适合快照的链路上继续引入更明确的结构对比
