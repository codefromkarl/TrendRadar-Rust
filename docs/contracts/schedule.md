# `schedule` 契约骨架

## 模块目标

把调度相关配置解析为稳定的执行决策。

## 当前实现基线

- `ScheduleDecision`
- `AppConfig.schedule`
- `decision_from_schedule`
- `decision_from_config`
- `decision_from_schedule_at`
- `decision_from_config_at`
- `ScheduleContext`

## 需要固化的契约

### 1. 输入契约

依赖的配置字段：

- `AppConfig.schedule.collect`
- `AppConfig.schedule.analyze`
- `AppConfig.schedule.push`
- `AppConfig.schedule.window.start_hour`
- `AppConfig.schedule.window.end_hour`
- `AppConfig.schedule.cooldown_minutes`
- `AppConfig.schedule.weekday.*`
- `AppConfig.schedule.weekend.*`

当前时间窗口表达方式：

- 在布尔阶段开关之外，新增可选 `schedule.window`
- `schedule.weekday` / `schedule.weekend` 可按日类型覆盖阶段开关与窗口
- `ScheduleContext.local_hour` 表示已按配置时区折算后的本地小时
- `ScheduleContext.is_weekend` 表示当前本地时间是否落在周末
- `ScheduleContext.current_time` 表示当前运行时间
- `ScheduleContext.last_success_at` 表示上次成功运行时间
- 当前窗口采用半开区间 `[start_hour, end_hour)` 语义
- 当 `start_hour > end_hour` 时表示跨午夜窗口
- 当存在 `cooldown_minutes` 且 `current_time < last_success_at + cooldown` 时，三个阶段均返回 `false`

时区来源：

- 时区仍来自 `AppConfig.timezone`
- 但当前布尔阶段开关不直接依赖时区计算

### 2. 输出契约

决策字段：

- `collect`
- `analyze`
- `push`

字段含义：

- `collect` 为 `true` 时，允许抓取阶段执行
- `analyze` 为 `true` 时，允许分析阶段执行
- `push` 为 `true` 时，允许推送阶段执行
- 若显式注入的本地小时不在窗口内，三个阶段均返回 `false`
- 若存在 `weekday/weekend` 覆盖规则，则当前日类型优先使用覆盖值，否则回落到顶层 `schedule`

默认决策：

- `ScheduleDecision::default()` 当前为全部 `true`
- 在 schedule 进入真正实现前，配置层默认值与决策层默认值保持一致

### 3. 纯逻辑约束

是否要求纯函数：

- 是
- 从 `AppConfig.schedule` 生成决策时，不应依赖外部状态
- 当需要时间窗口时，只能依赖显式注入的 `ScheduleContext`

是否允许读取外部系统时间：

- 当前运行时仍不允许直接读取系统时间
- 时间窗口测试与实现必须通过显式注入时间上下文完成

如何注入测试时间：

- 当前通过 `ScheduleContext { local_hour, is_weekend, current_time, last_success_at }` 注入
- 后续若引入更细粒度上下文，也应保持显式注入

## 错误契约

非法调度配置：

- 当前布尔字段不引入额外错误
- `schedule.window.start_hour == end_hour` 时进入 `TrendRadarError::InvalidConfig`
- 超出 `0..=23` 的小时值进入 `TrendRadarError::InvalidConfig`

时区相关错误：

- 当前无独立时区错误
- 一旦调度窗口开始依赖时区，应继续复用配置校验错误模型

## 验证方式

fixture：

- 复用 [minimal-valid.json](../../fixtures/system/config/minimal-valid.json)
- 新增 [window-daytime.json](../../fixtures/system/schedule/window-daytime.json)
- 新增 [window-overnight.json](../../fixtures/system/schedule/window-overnight.json)
- 新增 [invalid-window-equal-hours.json](../../fixtures/system/schedule/invalid-window-equal-hours.json)
- 新增 [invalid-window-out-of-range.json](../../fixtures/system/schedule/invalid-window-out-of-range.json)

测试：

- `cargo test -p trendradar-schedule`
- `cargo test -p trendradar-config`
- `cargo test --workspace`

快照：

- 当前不需要
- 当前窗口决策已由 fixture 驱动测试覆盖
- 当前仍不需要快照

## 开放问题

- `cooldown` 的状态来源当前由 `app` 层 sidecar 文件提供；如果后续改为数据库或远程状态，需要保持 `schedule` 层的显式上下文契约不变
