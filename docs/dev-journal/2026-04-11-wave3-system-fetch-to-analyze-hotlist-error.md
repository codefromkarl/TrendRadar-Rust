# Wave 3 system fetch to analyze hotlist error

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 热榜错误路径
- 目标：补齐 `fetch -> analyze` 在热榜来源上的错误路径，和 RSS 错误形成对称覆盖

## 本次完成内容

- 在 `tests/system/fetch_to_analyze.rs` 中新增非法热榜系统测试
- 复用 `fixtures/system/fetch/invalid-hotlist.json`
- 固定错误消息包含 `failed to parse fetch fixture` 与 fixture 路径
- 同步更新 `tests/README.md`

## 阶段结论

根级 `fetch -> analyze` 系统测试现在同时覆盖成功、空输入、RSS 错误和热榜错误四类路径。
