# Wave 3 analyze boundary case

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`analyze` 零排名边界保护
- 目标：让评分规则在异常输入下仍保持稳定可比对，不产生超过顶分的结果

## 本次完成内容

- 新增 `fixtures/system/analyze/zero-rank-input.json`
- 新增 `zero_rank_is_clamped_to_top_score` 测试
- 将 `score_news` 从 `101 - min(rank, 100)` 收紧为 `101 - clamp(rank, 1, 100)`
- 同步更新 analyze 契约、实施文档和验收矩阵

## 阶段结论

`W3-analyze-rule-cases` 至少完成了一条明确的边界收口：异常 `rank = 0` 不再产生 `101` 分，分析输出上界稳定在 `100`。

## 下一步

- 继续补更高阶排序 / 过滤样例
- 如果需要引入综合评分，先写 fixture 再扩规则
