# Fixtures

这里存放稳定的输入输出样例，用于：

- 调度测试
- 聚合测试
- 序列化测试
- 回归测试

建议统一使用 JSON 或 YAML，并为每个 fixture 标注来源与用途。

## 当前约束

- `fixtures/system/` 预留给系统性测试使用
- fixture 必须尽量稳定、可脱敏、可重复
- 如果样例来自旧系统真实数据，必须注明来源和裁剪方式

## 推荐分组

- `fixtures/system/config/`
- `fixtures/system/fetch/`
- `fixtures/system/analyze/`
- `fixtures/system/report/`
