# Wave 3 system analyze tie-break

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `analyze` tie-break 系统测试
- 目标：把同排名排序规则从 crate 内测试提升到根级系统测试，证明跨 crate 门控后仍保持稳定标题排序

## 本次完成内容

- 在 `tests/system/analyze_pipeline.rs` 中新增同排名系统测试
- 复用 `fixtures/system/analyze/same-rank-input.json`
- 固定在 `config -> schedule -> analyze` 链路下，同排名条目最终按 `title` 升序输出
- 同步更新 `tests/README.md`

## 阶段结论

Wave 3 的 richer case 已开始从 crate 内部规则测试提升到根级跨 crate 测试，系统验证面进一步贴近真实组合行为。
