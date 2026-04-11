# Wave 3 app report-only empty

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`app` 的空报告阶段组合
- 目标：固定 `collect=false, push=true` 时的系统行为，证明 `app` 只是消费阶段开关，不会偷偷补数据

## 本次完成内容

- 新增 `fixtures/system/config/report-only-empty.json`
- 在 `wave3_schedule_gate.rs` 中增加空报告系统测试
- 固定无采集数据时 `report_json` 仍可输出 `item_count = 0` 与空数组
- 同步更新 `app` 实施文档与验收矩阵

## 阶段结论

`app` 的系统级阶段组合样例进一步细化：现在不仅验证“跳不跳过”，也验证“在不采集数据但允许输出时，输出仍保持稳定空结构”。
