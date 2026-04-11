# Wave 3 system analyze disabled

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `analyze` 门控禁止路径
- 目标：把 `analyze=false` 的行为提升到根级系统测试，证明配置门控关闭时不会误跑排序和聚合

## 本次完成内容

- 新增 `fixtures/system/config/analyze-disabled.json`
- 在 `tests/system/analyze_pipeline.rs` 中新增门控禁止测试
- 固定 `decision.analyze = false` 时，排序与来源聚合结果都为空
- 同步更新 `tests/README.md`

## 阶段结论

根级 `analyze` 系统测试现在同时覆盖“允许分析”和“禁止分析”两条门控路径。
