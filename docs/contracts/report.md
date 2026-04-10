# `report` 契约骨架

## 模块目标

定义结构化 JSON 输出、运行元数据和后续报告层的稳定边界。

## 当前实现基线

- `render_news_json`
- `meta + items` 顶层结构

## 需要固化的契约

### 1. JSON 输出结构

- 顶层结构：
  `{"meta": ..., "items": [...] }`
- 结果数组字段：
  当前直接输出 `NewsItem` 数组
- 元数据字段：
  `started_at`、`timezone`、`item_count`

### 2. 运行上下文

- 是否输出 `RunContext`：
  当前输出其中的 `started_at` 与 `timezone`
- 版本信息：
  Wave 1 暂不引入
- 错误输出格式：
  当前未在 `report` crate 内实现

### 3. 报告边界

- 首版保留：
  结构化 JSON 输出
- 首版延后：
  HTML 报告与错误渲染

## 兼容要求

- 是否要求兼容旧 JSON / HTML 报告结构：
- 不兼容时的替代输出说明：

## 验证方式

- fixture：
  `fixtures/system/report/news-report-input.json`
- 测试：
  `cargo test -p trendradar-report`
- 快照：
  当前不需要，顶层结构由 JSON 字段断言固定

## 待补充决策

- `report` 是否同时负责错误渲染
- HTML 报告何时从“延后”进入实现
