# Wave 3 system fixture growth

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`app` 的阶段开关系统证明
- 目标：通过系统 fixture 证明 `app` 只执行 `schedule` 决策，不自己实现 analyze / report 规则

## 本次完成内容

- 新增 `fixtures/system/config/collect-only.json`
- 新增 `fixtures/system/config/disabled-all.json`
- 新增 `crates/app/tests/wave3_schedule_gate.rs`
- 断言 `collect=true, analyze=false, push=false` 时仍会抓取和落库，但不会执行分析或渲染报告
- 断言 `collect=false, analyze=false, push=false` 时返回空 pipeline 状态
- 同步更新 `app` 实施文档与验收矩阵

## 阶段结论

这一组系统测试把 Wave 3 的边界证明从 crate 级推进到了系统级：`app` 当前只消费 `ScheduleDecision`，没有自行决定何时抓取、分析或输出。

## 下一步

- 继续补更多系统级 fixture，例如 `collect=false` 的空链路样例
- 如果未来接入时间窗口到系统 pipeline，先明确上下文注入方式，再补对应系统测试
