# Wave 3 schedule window cases

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`schedule` 最小时间窗口表达
- 目标：在不修改 `app` 业务边界的前提下，为 `schedule` 增加可验证的小时窗口决策能力

## 本次完成内容

- 给 `ScheduleConfig` 增加可选 `window`
- 增加 `ScheduleContext`、`decision_from_schedule_at`、`decision_from_config_at`
- 新增白天窗口、跨午夜窗口、非法窗口三个 fixture
- 后续补上越界小时非法窗口 fixture，固定 `0..=23` 校验边界
- 补齐 `config` 与 `schedule` 的 crate 级测试和契约文档

## 关键决策

- 决策内容：窗口能力先落在 `config` / `schedule`，不直接接进 `app`
- 原因：当前 `app` 仍应保持薄编排；时间上下文应由调用方显式注入，而不是让 `schedule` 或 `app` 偷读系统时间
- 备选方案：直接让 `app` 按 `started_at` 计算并参与窗口判断
- 为什么没有选备选方案：当前还没有把配置时区稳定折算成运行时本地小时的统一入口，贸然接入会把时间语义和编排耦合进 `app`

## 阶段结论

`W3-schedule-window-cases` 的最小闭环已经落地：配置可表达窗口，`schedule` 可在显式上下文下做窗口内外判定，非法窗口也有固定错误断言，并覆盖相等小时与越界小时两类失败路径。

## 下一步

- 如果要把窗口真正接入系统 pipeline，应先明确“配置时区 -> 本地小时上下文”的统一折算入口
- 继续推进 `W3-analyze-rule-cases` 或 `W3-system-fixture-growth`
