# `analyze` 契约骨架

## 模块目标

定义过滤、聚合、排序、评分和统计的纯计算接口。

## 当前实现基线

- `score_news`
- `rank_news`
- `group_news_by_source`

## 需要固化的契约

### 1. 输入契约

当前最小输入模型：

- 输入是一组 `domain::NewsItem`
- 每个条目必须包含 `title`、`source_id`、`rank`
- 首版分析只要求能读取并排序该集合，不要求先做复杂过滤或聚合

过滤条件：

- Wave 1 起步阶段暂不引入复杂过滤条件
- 只使用 fixture 内已有条目作为分析输入

聚合键：

- 当前不引入聚合键
- 后续如果新增聚合，优先按 `source_id` 或来源分组

排序规则输入：

- 首版由 `rank` 决定基础优先级
- `rank` 越小，条目优先级越高
- `score_news` 是当前唯一的显式评分函数

### 2. 输出契约

输出条目结构：

- 排序结果当前输出 `RankedNews`
- 聚合结果当前输出 `SourceSummary`
- 排序结果保留原始 `NewsItem` 与计算分数

统计字段：

- 当前先固化来源聚合统计
- `SourceSummary` 至少包含 `source_id`、`item_count`、`best_rank`

排序稳定性要求：

- 相同 `rank` 的条目保持输入顺序或由上游明确规定次序
- fixture 需要覆盖至少一个严格递增的 rank 序列，避免歧义

### 3. 评分契约

分数字段：

- 当前仅返回 `u32`
- 计算结果由 `score_news(&NewsItem)` 决定

排名与热度权重：

- 当前版本只看 `rank`
- 公式为 `101 - clamp(rank, 1, 100)`
- 这意味着 `rank = 1` 得到 `100`，`rank = 100` 得到 `1`
- `rank = 0` 当前按边界保护视为 `1` 处理，避免产生 `101` 分

同分时的决策规则：

- 当前排序先按 `score` 降序，再按 `rank` 升序，最后按 `title` 升序
- 后续如果引入综合排序，再在此处补更高阶同分处理

## 错误与边界

空输入行为：

- 空输入应返回空结果，不应报错

无匹配结果行为：

- 当前没有过滤条件，因此不存在“无匹配”分支

非法排序规则行为：

- 当前没有外部排序规则输入
- 后续若新增排序规则，必须在 fixture 中补非法输入样例
- `rank = 0` 不报错，但评分阶段会钳制到顶分边界

## 验证方式

fixture：

- [fixtures/system/analyze/news-ranking-input.json](../../fixtures/system/analyze/news-ranking-input.json)
- [fixtures/system/analyze/source-groups-input.json](../../fixtures/system/analyze/source-groups-input.json)
- [fixtures/system/analyze/zero-rank-input.json](../../fixtures/system/analyze/zero-rank-input.json)
- [fixtures/system/analyze/same-rank-input.json](../../fixtures/system/analyze/same-rank-input.json)

测试：

- `cargo test -p trendradar-analyze`
- 读取排序 fixture 后，检查 `score_news` 结果是否为 `100, 89, 1`
- 读取聚合 fixture 后，检查来源计数与最佳排名是否稳定
- 读取零排名 fixture 后，检查分数不会超过 `100`
- 读取同排名 fixture 后，检查最终按 `title` 升序稳定排序

快照：

- 当前不需要快照
- 如果未来引入聚合输出，再补结构快照

## 开放问题

- 分析结果是否需要保留来源明细
- 聚合层与排序层是否拆开暴露
