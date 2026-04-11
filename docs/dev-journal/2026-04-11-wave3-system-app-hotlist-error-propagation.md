# Wave 3 system app hotlist error propagation

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的热榜错误透传
- 目标：补齐 `app` 在系统层对热榜 fixture 解析错误的透明传播，和 RSS 错误路径形成对称覆盖

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增非法热榜 fixture 系统测试
- 复用 `fixtures/system/fetch/invalid-hotlist.json`
- 固定错误消息包含 `failed to parse fetch fixture` 与 fixture 路径
- 同步更新 `tests/README.md`

## 阶段结论

根级 `app` 系统测试现在已经覆盖 RSS 与热榜两类来源的上游解析错误透传。
