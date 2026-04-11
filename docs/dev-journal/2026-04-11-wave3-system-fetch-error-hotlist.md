# Wave 3 system fetch error hotlist

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> domain` 热榜错误路径
- 目标：把非法热榜 fixture 的解析失败提升到根级系统测试，补齐 `fetch -> domain` 在两类来源上的错误路径覆盖

## 本次完成内容

- 新增 `fixtures/system/fetch/invalid-hotlist.json`
- 在 `tests/system/fetch_to_domain.rs` 中新增非法热榜系统测试
- 固定错误消息包含 `failed to parse fetch fixture` 与 fixture 路径
- 同步更新 `tests/README.md`

## 阶段结论

根级 `fetch -> domain` 系统测试现在同时覆盖 RSS 与热榜两类来源的正常输入、空输入和非法输入路径。
