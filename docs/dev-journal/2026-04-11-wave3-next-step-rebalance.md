# Wave 3 next step rebalance

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：把下一步建议从 `app` 细化转向非 `app` richer cases
- 目标：根据当前 53 条根级系统测试覆盖，明确后续 autoresearch 更适合把增量投入到其他跨 crate 链路，而不是继续在 `app` 上做低价值重复

## 本次完成内容

- 更新 `docs/system-test-template.md` 的下一步建议
- 更新 `docs/acceptance-matrix.md` 和 `docs/parallel-migration-plan.md` 中的当前判断
- 明确 `app` 的阶段门控、来源形态、动态窗口和错误传播矩阵已经较完整
- 明确后续更适合优先扩 `fetch -> analyze`、`storage -> report` 等非 `app` 系统链路

## 阶段结论

Wave 3 现在不再只是“继续给 `app` 加样例”就能获得最好收益。文档已经把这个阶段判断显式写出来，后续继续前景推进时更容易把注意力转向仍有真实增量空间的跨 crate richer cases。
