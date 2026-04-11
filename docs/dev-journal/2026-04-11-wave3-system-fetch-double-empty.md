# Wave 3 system fetch double empty

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> domain` 双来源空输入
- 目标：验证空 RSS 与空热榜同时出现时，跨 crate 归一化链路仍稳定返回空集合

## 本次完成内容

- 在 `tests/system/fetch_to_domain.rs` 中新增双来源同时为空的系统测试
- 复用 `fixtures/system/fetch/empty-rss.json` 与 `fixtures/system/fetch/empty-hotlist.json`
- 固定两类来源同时为空时都返回空集合
- 同步更新 `tests/README.md`

## 阶段结论

根级 `fetch -> domain` 系统测试已经不只覆盖单来源空输入，也覆盖了多来源同时为空的组合路径。
