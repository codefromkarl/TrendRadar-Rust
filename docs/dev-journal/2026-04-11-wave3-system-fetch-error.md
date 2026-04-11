# Wave 3 system fetch error

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> domain` 错误路径
- 目标：把非法 RSS fixture 的解析失败从 crate 单测提升到根级系统测试，验证错误定位信息在跨 crate 验证面中仍然清晰

## 本次完成内容

- 在 `tests/system/fetch_to_domain.rs` 中新增非法 RSS fixture 系统测试
- 复用 `fixtures/system/fetch/invalid-rss.json`
- 固定错误消息包含 `failed to parse fetch fixture` 与 fixture 路径
- 同步更新 `tests/README.md`

## 阶段结论

Wave 3 的根级系统测试现在已经覆盖 `fetch -> domain` 的正常输入、合法空输入和非法输入三条基本路径。
