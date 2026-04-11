# Wave 3 system app overnight window block

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `app pipeline` 的跨午夜窗口阻断
- 目标：补齐 `window-overnight.json` 在窗口外的完整链路行为，形成 overnight window 在 `app` 层的对称系统证据

## 本次完成内容

- 在 `tests/system/app_pipeline_modes.rs` 中新增跨午夜窗口外阻断系统测试
- 复用 `schedule/window-overnight.json`、`hotlist-weibo.json` 和 `rss-rust-blog.json`
- 固定本地时间落在窗口外时，pipeline 直接返回全空状态
- 同步更新 `README.md`、`tests/README.md`、`tests/system/README.md`、`docs/implementation/app.md`、`docs/acceptance-matrix.md`、`docs/parallel-migration-plan.md`、`docs/system-test-template.md`

## 阶段结论

现在 `app` 级系统测试已经对跨午夜窗口具备“窗口内允许 / 窗口外禁止”的对称样例。这样 `started_at + timezone` 的 overnight 行为不再只依赖 crate 级判断，而是通过完整编排链路得到验证。
