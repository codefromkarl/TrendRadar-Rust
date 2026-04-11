# Wave 3 system app RSS-only config

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的 RSS-only 配置边界
- 目标：补齐 `platforms=[]` 但只提供 RSS source 时的系统行为，验证 `app` 不会把未使用的热榜平台配置当成隐式前提

## 本次完成内容

- 新增 `fixtures/system/config/minimal-valid-rss-only.json`
- 在 `tests/system/app_pipeline_modes.rs` 中新增 RSS-only 全链路系统测试
- 固定 `platforms=[]`、单 RSS source 下仍可完成 `config -> app -> analyze -> storage -> report`
- 断言来源聚合结果只有 `rust-blog`，并固定报告 `meta.item_count = 2`
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

根级 `app` 系统测试现在不仅覆盖输入列表的空 / 单 / 双来源形态，也覆盖“未使用的热榜配置可缺省”的边界。`app` 仍只消费显式传入的 source 列表，没有吸入“必须配置哪些平台”这种业务规则。
