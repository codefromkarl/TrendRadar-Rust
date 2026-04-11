# Wave 3 analyze tie-break

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：`analyze` 同排名排序稳定性
- 目标：把契约里“最终按 `title` 升序”的 tie-break 规则固定成真实 fixture 和测试

## 本次完成内容

- 新增 `fixtures/system/analyze/same-rank-input.json`
- 为 `rank_news` 新增同排名排序测试
- 固定同分同排名时按 `title` 升序输出
- 同步更新 analyze 契约、实施文档和验收矩阵

## 阶段结论

`analyze` 的排序规则现在不仅覆盖常规分数序列，也覆盖了同分 / 同排名时的稳定排序行为。
