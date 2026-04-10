# `schedule` 契约骨架

## 模块目标

把调度相关配置解析为稳定的执行决策。

## 当前实现基线

- `ScheduleDecision`
- `AppConfig.schedule`
- `decision_from_schedule`
- `decision_from_config`

## 需要固化的契约

### 1. 输入契约

依赖的配置字段：

- `AppConfig.schedule.collect`
- `AppConfig.schedule.analyze`
- `AppConfig.schedule.push`

当前时间窗口表达方式：

- 首版尚未进入时间窗口表达
- 当前仅支持布尔阶段开关

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

默认决策：

- `ScheduleDecision::default()` 当前为全部 `true`
- 在 schedule 进入真正实现前，配置层默认值与决策层默认值保持一致

### 3. 纯逻辑约束

是否要求纯函数：

- 是
- 从 `AppConfig.schedule` 生成决策时，不应依赖外部状态

是否允许读取外部系统时间：

- 当前不允许
- 当后续支持时间窗口时，也应通过显式注入时间上下文测试

如何注入测试时间：

- 当前尚不需要
- 后续若引入时间窗口，应通过参数或上下文对象注入固定时间

## 错误契约

非法调度配置：

- 当前布尔字段不引入额外错误
- 后续若增加复杂表达式，非法值统一进入 `TrendRadarError::InvalidConfig`

时区相关错误：

- 当前无独立时区错误
- 一旦调度窗口开始依赖时区，应继续复用配置校验错误模型

## 验证方式

fixture：

- 当前先复用 [minimal-valid.json](../../fixtures/system/config/minimal-valid.json)
- 当前先复用 [invalid-empty-timezone.json](../../fixtures/system/config/invalid-empty-timezone.json)

测试：

- `cargo test -p trendradar-schedule`
- `cargo test -p trendradar-config`
- `cargo test --workspace`

快照：

- 当前不需要
- 当前布尔阶段开关已由 fixture 驱动测试覆盖
- 后续如果进入时间窗口表达，再考虑固定输出快照

## 开放问题

- 是否需要支持工作日、小时段、冷却周期等更细粒度规则
