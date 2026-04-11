# Wave 3 system analyze empty input

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 3
- 主题：根级 `analyze` 空输入路径
- 目标：把空输入场景提升到根级系统测试，验证 `config -> schedule -> analyze` 在允许分析时也能稳定返回空结果

## 本次完成内容

- 新增 `fixtures/system/analyze/empty-input.json`
- 在 `tests/system/analyze_pipeline.rs` 中新增空输入系统测试
- 固定排序结果和来源聚合结果都为空
- 同步更新 `tests/README.md`

## 阶段结论

根级 `analyze` 系统测试现在已经覆盖成功、禁止、tie-break、零排名和空输入五类高信号行为。
