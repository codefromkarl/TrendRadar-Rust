# Wave 3 system fetch to analyze error

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `fetch -> analyze` 错误路径
- 目标：把非法 RSS fixture 的失败提升到 `fetch -> analyze` 根级系统测试，验证组合链路在错误输入下能及时停止并暴露错误

## 本次完成内容

- 在 `tests/system/fetch_to_analyze.rs` 中新增错误路径系统测试
- 复用 `fixtures/system/fetch/invalid-rss.json`
- 固定错误消息包含 `failed to parse fetch fixture` 与 fixture 路径
- 同步更新 `tests/README.md`

## 阶段结论

根级 `fetch -> analyze` 系统测试现在同时覆盖成功、空输入和错误三类路径。
