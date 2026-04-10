# 系统性测试模板

## 文档目标

这份文档用于给当前仓库提供一个可直接复用的系统性测试模板。

这里的“系统性测试”不是简单单元测试，而是围绕一个完整链路或一个跨 crate 行为做验证：

- 有明确输入
- 有稳定 fixture
- 有可比较输出
- 有固定验证命令

## 适用范围

当前仓库里的系统性测试，优先覆盖这些场景：

- 配置加载到运行上下文的链路
- 抓取结果到统一领域模型的归一化
- 调度、过滤、聚合、排序的组合逻辑
- 存储写入后的结果一致性
- 结构化输出和快照结果

## 测试文件组织

建议按下面方式组织：

```text
tests/
  system/
    README.md
    config_to_bootstrap.rs
    fetch_to_domain.rs
    analyze_pipeline.rs
fixtures/
  system/
    config/
    fetch/
    analyze/
    report/
tests/snapshots/
```

## 一个基础系统测试的最小构成

每个系统测试建议都具备这 5 部分：

1. 测试目标：要验证哪条链路。
2. 输入 fixture：稳定的 JSON、YAML、文本或内联样例。
3. 执行步骤：调用哪些 crate 或公开函数。
4. 期望结果：结构化断言或快照。
5. 验证入口：本地和 CI 用哪条命令跑。

## Rust 测试模板

```rust
//! 示例：系统性测试模板

use anyhow::Result;

#[test]
fn system_case_name() -> Result<()> {
    // 1. 准备 fixture
    // let input = std::fs::read_to_string("fixtures/system/...");

    // 2. 调用跨 crate 链路
    // let config = trendradar_config::...;
    // let output = trendradar_app::...;

    // 3. 做结构化断言
    // assert_eq!(output.xxx, expected.xxx);

    // 4. 如有必要，做 snapshot
    // insta::assert_yaml_snapshot!(output);

    Ok(())
}
```

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

## 当前已落地样例

当前仓库已经补了第一条真实样例：

- fixture：`fixtures/system/config/minimal-valid.json`
- fixture：`fixtures/system/config/invalid-empty-timezone.json`
- 测试：`crates/app/tests/config_to_bootstrap.rs`

这个样例的作用不是覆盖复杂业务，而是先把“fixture -> config -> app”这条最小跨 crate 链路跑通。

## 下一步建议

- 先为 `config -> app::bootstrap` 补第一条系统性测试样例
- 再为 `domain + analyze` 补固定输入输出样例
- 后续逐步把 `fetch`、`storage`、`report` 接入同一套模板
