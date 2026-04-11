# Wave 3 system fetch empty

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> domain` 空输入系统测试
- 目标：把合法空 RSS fixture 的行为从 crate 单测提升到根级系统测试，证明跨 crate 归一化链路在空输入下仍稳定

## 本次完成内容

- 在 `tests/system/fetch_to_domain.rs` 中新增空 RSS 系统测试
- 复用 `fixtures/system/fetch/empty-rss.json`
- 固定 `FixtureRssFetcher` 在根级系统验证里返回空集合
- 同步更新 `tests/README.md`

## 阶段结论

Wave 3 的根级系统测试已开始同时覆盖正常输入与合法空输入，不再只验证“有数据时能跑通”。
