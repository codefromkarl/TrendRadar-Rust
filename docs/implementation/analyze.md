# `analyze` 实施骨架

## 目标

实现过滤、聚合、排序、评分等纯计算逻辑。

## 前置依赖

- `domain` 输入模型稳定

## 输入与输出

- 输入：统一内容模型与过滤 / 排序参数
- 输出：稳定的分析结果与统计字段

## 本轮范围

- 已扩展基础评分逻辑到排序与来源聚合
- 已准备 analyze 输入 fixture
- 已补结构断言与 fixture 测试

## 暂不处理

- AI 分析
- 与报告层混写的格式化逻辑

## 建议子任务

- 评分规则
- 最小输入 fixture
- 排序与结构断言测试

## 完成定义

- 同一 fixture 输出稳定
- 结果字段可被快照比较
- 逻辑保持纯函数化

## 当前进展

- 已提供 `score_news`、`rank_news`、`group_news_by_source`
- 已用固定 fixture 覆盖排序结果与来源聚合结果
- 更高阶过滤与综合排序仍留待后续阶段

## 验证命令

```bash
cargo test -p trendradar-analyze
```
