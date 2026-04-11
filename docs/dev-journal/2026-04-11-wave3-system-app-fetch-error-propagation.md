# Wave 3 system app fetch error propagation

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的抓取错误透传
- 目标：把 `invalid-rss.json` 的解析失败提升到根级 `app` 系统测试，验证 `app` 不会吞掉上游 fixture 解析错误

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增错误透传系统测试
- 复用 `fixtures/system/fetch/invalid-rss.json`
- 固定错误消息包含 `failed to parse fetch fixture` 与 fixture 路径
- 同步更新 `tests/README.md`

## 阶段结论

根级 `app` 系统测试现在不仅覆盖正向与阶段组合，也覆盖了上游 `fetch` 解析错误的透明传播。
