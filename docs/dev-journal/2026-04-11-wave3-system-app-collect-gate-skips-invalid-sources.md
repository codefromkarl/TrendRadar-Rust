# Wave 3 system app collect gate skips invalid sources

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的 collect 门控优先级
- 目标：补齐 `collect=false` 时对损坏 source 的系统行为，验证 `app` 会先应用阶段门控，再决定是否触碰上游 fixture

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增 `collect=false` 跳过损坏 source 的系统测试
- 复用 `report-only-empty.json` 配置和 `invalid-hotlist.json`、`invalid-rss.json`
- 固定即使传入损坏 source，`collect=false` 时仍会稳定输出空报告
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

这条测试把一个关键执行顺序钉死了：`app` 会先消费调度决策，再决定是否解析 source。这样后续即使补更多 source 类型或错误路径，也不容易把抓取副作用提前到已禁用的阶段之前。
