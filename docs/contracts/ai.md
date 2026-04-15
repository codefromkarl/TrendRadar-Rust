# `ai` 契约骨架

## 模块目标

定义可选 AI 分析旁路的最小输入输出契约。

当前重点是：

- 不阻塞主抓取与存储链路
- provider 可切换
- 输出结果可复查

## 当前实现基线

- `AnalysisProvider` trait
- `MockAnalysisProvider`
- 最小 `openai-compatible` provider
- `AiAnalysis` 结构化结果
- Markdown 渲染入口

## 输入与输出契约

### 输入

- `NewsItem[]`
- `RunContext`
- `ProviderConfig`
- `provider`
- `timeout_secs` / `retry_attempts`
- `max_items`
- 可选 `prompt`
- 可选 `model` / `base_url` / `api_key` / `api_key_env`

### 输出

- `summary`
- `key_topics`
- `source_highlights`

### 旁路语义

- AI 分析失败不应中断主 pipeline
- provider 不可用时应返回可断言错误或在 `app` 侧降级为 warn + skip

## 当前 provider 契约

- 已实现 provider：
  `mock`
  `openai-compatible`
- 当前仍未实现 provider：
  更完整的 provider 矩阵与流式输出

## 当前验证入口

- `cargo test -p trendradar-ai`
- `cargo test -p trendradar-app ai_analysis -- --nocapture`
