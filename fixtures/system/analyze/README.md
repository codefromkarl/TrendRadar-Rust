# Analyze Fixtures

这个目录存放 `analyze` 起步阶段的最小输入样例。

## 当前样例

- `news-ranking-input.json`

## 样例说明

这个样例只覆盖当前 `score_news` 的最小行为：

- 输入类型是 `domain::NewsItem`
- `rank` 越小，分数越高
- 当前不测试过滤、聚合或复杂排序

## 预期结果

把样例按 `rank` 传入 `score_news` 后，结果应为：

- `rank = 1` -> `100`
- `rank = 12` -> `89`
- `rank = 100` -> `1`

## 维护要求

- 保持样例尽量小
- 每次修改样例都要同步更新契约文档
- 如果后续新增聚合或排序规则，再补对应 fixture
