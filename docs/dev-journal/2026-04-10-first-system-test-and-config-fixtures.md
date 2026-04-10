# 开发记录：第一条真实系统测试与配置样例

## 基本信息

- 日期：2026-04-10
- 阶段：迁移基线收敛
- 主题：补最小配置 fixture 和第一条系统性测试
- 目标：把系统测试从模板推进到真实可执行链路

## 本次完成内容

- 新增 `fixtures/system/config/minimal-valid.json`
- 新增 `fixtures/system/config/invalid-empty-timezone.json`
- 新增 `crates/app/tests/config_to_bootstrap.rs`
- 在 `trendradar-config` 中补了 JSON 配置加载与校验入口
- 在 `trendradar-app` 中补了基于配置对象的 `bootstrap_with_config`
- 同步更新环境准备文档和测试模板文档

## 为什么先做这条链路

当前仓库最早需要稳定下来的，不是复杂抓取逻辑，而是：

- fixture 如何进入代码
- config 如何被解析和校验
- app 如何承接一个最小可验证的编排入口

这条链路足够小，但已经跨过了多个模块边界，适合作为系统性测试的起点。

## 结果

- 已能通过稳定 fixture 验证成功配置路径
- 已能通过稳定 fixture 验证空时区配置的失败路径
- `cargo test --workspace` 通过

## 下一步

- 为 `domain + analyze` 补第一组固定输入输出样例
- 逐步把 `fetch` 和 `report` 接到同一套 fixture 驱动测试里
