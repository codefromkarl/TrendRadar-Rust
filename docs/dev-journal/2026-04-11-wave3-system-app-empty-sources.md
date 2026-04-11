# Wave 3 system app empty sources

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的空来源闭环
- 目标：补齐 `collect=true` 但 `sources=[]` 时的系统行为，验证 `app` 仍只做薄编排，不会因为缺少 fixture source 而报错或吸入额外规则

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增空来源全链路系统测试
- 固定 `minimal-valid.json` 配置下 `sources=[]` 仍可完成 `config -> app -> storage -> report`
- 断言 `collected_items`、`ranked_items`、`source_summaries`、`stored_items` 全为空
- 断言 `report_json` 仍会稳定输出空 `items` 与正确 `meta`
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

根级 `app` 系统测试现在不仅覆盖有输入、缺省 `schedule`、阶段门控和错误透传，也覆盖“无 source 但链路仍需稳定返回”的空来源场景。这个边界继续证明 `app` 只是串接现有模块，没有引入额外业务规则。
