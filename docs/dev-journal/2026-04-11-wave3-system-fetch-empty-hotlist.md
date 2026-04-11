# Wave 3 system fetch empty hotlist

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> domain` 空热榜路径
- 目标：把合法空热榜 fixture 的行为提升到根级系统测试，补齐 `fetch -> domain` 在两类来源上的空输入覆盖

## 本次完成内容

- 新增 `fixtures/system/fetch/empty-hotlist.json`
- 在 `tests/system/fetch_to_domain.rs` 中新增空热榜系统测试
- 固定 `FixtureHotlistFetcher` 在根级系统验证里返回空集合
- 同步更新 `tests/README.md`

## 阶段结论

根级 `fetch -> domain` 系统测试现在同时覆盖正常输入、合法空 RSS、合法空热榜和非法 RSS 四类关键路径。
