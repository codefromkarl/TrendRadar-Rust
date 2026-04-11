# Wave 3 system storage dedup report

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `storage -> report` 去重组合
- 目标：把“重复标题保留更优 rank”从 `storage` crate 内测试提升到跨 crate 系统测试，证明进入 `report` 前已经完成去重收口

## 本次完成内容

- 在 `tests/system/storage_to_report.rs` 中新增重复标题系统测试
- 断言 `SqliteNewsRepository` 保留更优 `rank` 后，`render_news_json` 输出的 `item_count = 1`
- 同步更新 `tests/README.md`

## 阶段结论

根级 `storage -> report` 系统测试现在同时覆盖有数据、空数据和去重后三种组合路径。
