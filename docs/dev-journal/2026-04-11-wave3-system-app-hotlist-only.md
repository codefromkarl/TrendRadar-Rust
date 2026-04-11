# Wave 3 system app hotlist-only

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的 hotlist-only 闭环
- 目标：补齐只提供热榜 source、不提供 RSS source 时的系统行为，验证 `app` 不依赖固定的双来源组合

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增 hotlist-only 全链路系统测试
- 复用 `minimal-valid.json` 与 `hotlist-weibo.json`
- 固定只存在热榜输入时的报告输出，断言系统仍会稳定完成 `config -> app -> analyze -> storage -> report`
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

根级 `app` 系统测试现在已经覆盖空来源、RSS-only、hotlist-only 和双来源几类核心输入形态。`app` 的职责仍然是按输入列表做薄编排，而不是对来源组合施加额外业务规则。
