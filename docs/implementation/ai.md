# `ai` 实施骨架

## 目标

把 AI 分析能力作为可选旁路接入当前内核，不污染 `analyze` 与 `report` 的纯逻辑边界。

## 本轮范围

- 新增独立 `trendradar-ai` crate
- 收口 provider trait
- 实现 `mock` provider
- 在 `app` 中按配置可选启用
- 将 AI 分析结果以 Markdown 旁路形式暴露

## 本轮不做

- 真实 OpenAI / 兼容 API 接入
- 流式输出
- provider 凭证管理
- 翻译能力

## 当前实现

- `config.ai_analysis` 已提供最小字段
- `app` 在不影响主链路的前提下生成 `ai_analysis_markdown`
- CLI 当前将 AI 分析输出打印到 stderr，避免破坏 JSON stdout

## 后续建议

后续如继续推进，可补：

1. 真实 provider
2. retry / timeout 细化
3. AI 输出的 JSON / HTML 集成
