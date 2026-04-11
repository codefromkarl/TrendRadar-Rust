# Wave 3 system storage report order

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `storage -> report` 排序稳定性
- 目标：把“写入顺序不影响最终报告顺序”提升到根级系统测试，验证仓储排序在跨 crate 输出里仍然稳定

## 本次完成内容

- 在 `tests/system/storage_to_report.rs` 中新增乱序写入系统测试
- 断言 `render_news_json` 最终仍按仓储排序输出
- 同步更新 `tests/README.md`

## 阶段结论

根级 `storage -> report` 系统测试现在不仅覆盖有数据、空数据和去重，还覆盖了写入顺序不影响输出顺序的稳定性。
