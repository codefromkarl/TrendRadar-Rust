# `schedule` 实施骨架

## 目标

把调度配置转成稳定的执行决策接口。

## 前置依赖

- `config` 调度字段契约稳定

## 输入与输出

- 输入：调度配置、测试时钟或固定时间
- 输出：`ScheduleDecision` 或等价决策结构

## 本轮范围

- 已绑定配置输入模型
- 已实现纯逻辑决策
- 已补固定样例测试

## 暂不处理

- 复杂运行时编排
- 外部调度器集成

## 建议子任务

- 从 `AppConfig.schedule` 映射到决策结构
- 时间窗口表达
- 决策逻辑
- fixture 测试

## 完成定义

- 固定样例输出稳定
- 输入输出边界写入契约文档
- 不依赖 `app` 层业务逻辑

## 当前进展

- 已可从 `AppConfig.schedule` 映射到 `ScheduleDecision`
- 已用显式配置和 `fixtures/system/config/minimal-valid.json` 覆盖最小样例
- 时间窗口表达仍留待后续阶段

## 验证命令

```bash
cargo test -p trendradar-schedule
```
