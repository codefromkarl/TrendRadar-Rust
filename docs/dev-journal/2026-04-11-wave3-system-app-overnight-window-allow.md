# Wave 3 system app overnight window allow

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的跨午夜窗口放行
- 目标：把 `window-overnight.json` 提升到 `app` 全链路，验证 `started_at + timezone` 在跨午夜窗口内也能正确驱动完整编排

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增跨午夜窗口内放行系统测试
- 复用 `schedule/window-overnight.json`、`hotlist-weibo.json` 和 `rss-rust-blog.json`
- 固定本地时间落在跨午夜窗口内时，`app` 会执行采集、落库并输出报告，同时保持 `analyze=false`
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

`app` 级系统测试现在不只覆盖白天窗口的放行 / 阻断，也覆盖跨午夜窗口的真实放行链路。这说明 `app` 对时间窗口的处理没有停留在 crate 级判断，而是已经通过整条编排链路得到验证。
