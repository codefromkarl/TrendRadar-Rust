# `app` 实施骨架

## 目标

在各核心模块最小闭环完成后，负责薄编排和系统性测试挂载。
当前已实现两条 pipeline 路径：基于 fixture 的最小闭环和基于 config 驱动的 HTTP pipeline。

## 前置依赖

- `config`
- `schedule`
- `analyze`
- `fetch`
- `storage`
- `report`

## 输入与输出

- 输入：各模块公开接口
- 输出：最小 pipeline、系统 fixture、编排入口
- 当前入口：`run_fixture_pipeline`（fixture 路径）、`run_config_pipeline`（HTTP 路径）、`trendradar` binary

## 本轮范围

- 串接模块接口
- 保持 `app` 薄层
- 添加系统性测试挂载点
- 用固定 fixture 验证端到端闭环

## 暂不处理

- 复杂 CLI 交互
- 把业务逻辑下沉失败后反向塞入 `app`

## 建议子任务

- bootstrap 扩展
- pipeline 编排
- 系统测试

## 完成定义

- 至少一条完整系统链路可跑通
- `app` 不承载核心业务规则
- 系统 fixture 可复查

## 当前进展

- 已存在一条从 `config` 到结构化输出的最小 fixture pipeline
- 已补 `collect-only` 系统 fixture，验证 `app` 会按 `schedule` 决策跳过 analyze / report 阶段
- 已补 `disabled-all` 系统 fixture，验证 `app` 在全部阶段关闭时返回空 pipeline 状态
- 已补 `report-only-empty` 系统 fixture，验证 `app` 在不采集数据时仍可按阶段开关输出空报告
- 已补 `analyze-without-report` 系统 fixture，验证 `app` 会保留分析结果但跳过报告渲染
- 已补 `collect-and-report-no-analyze` 系统 fixture，验证 `app` 会输出报告但不会生成分析结果
- 已补空来源全链路系统测试，验证 `app` 在 `collect=true` 但无 fixture source 时仍稳定返回空集合与空报告
- 已补单来源全链路系统测试，验证 `app` 不依赖“双来源固定组合”，只要存在任一 source 就能稳定产出部分结果
- 已补 RSS-only 全链路系统测试，验证未使用热榜 source 时 `app` 不依赖 `platforms` 中存在任何热榜平台
- 已补 hotlist-only 全链路系统测试，验证未使用 RSS source 时 `app` 也不依赖固定的双来源组合
- 已补 `collect=false` 时跳过损坏 source 的系统测试，验证阶段门控优先于 source 解析，`app` 不会在已禁用抓取时提前触碰上游 fixture
- 已补窗口阻断时跳过损坏 source 的系统测试，验证动态 `schedule.window` 门控同样先于 source 解析生效
- 已补 `collect-only` 时传播损坏 source 错误的系统测试，验证 source 解析是否发生只由 `collect` 决定，而不是由后续 `analyze` / `push` 状态决定
- 已补窗口放行时传播损坏 source 错误的系统测试，验证动态窗口在放行路径上也不会吞掉上游解析错误
- 已补跨午夜窗口内放行的完整 pipeline 系统测试，验证 `started_at + timezone` 在 overnight 窗口下同样能驱动整条编排链路
- 已补跨午夜窗口外阻断的完整 pipeline 系统测试，验证 overnight 窗口在 `app` 层同样具备“窗口内允许 / 窗口外禁止”的对称行为
- 已补窗口外阻断样例，验证 `app` 会在存在 `schedule.window` 时按 `started_at + timezone` 计算本地小时并阻断全链路
- `app` 继续只承担编排与系统测试挂载
- 已提取 `run_pipeline_with_fetchers` 作为 fixture 和 HTTP pipeline 共享的核心编排逻辑
- 已实现 `run_config_pipeline`，从 `AppConfig` 的 `rss_feeds` 和 `hotlist_apis` 构建 HTTP fetcher 并运行全链路
- 已添加 `trendradar` binary 入口（`src/main.rs`），支持从配置文件运行 HTTP pipeline
- 已补 3 条 mockito 隔离的 HTTP pipeline 集成测试（正常抓取、空来源、HTTP 错误传播）

## 验证命令

```bash
cargo test -p trendradar-app
cargo test --workspace wave2_minimal_pipeline_end_to_end -- --exact
```
