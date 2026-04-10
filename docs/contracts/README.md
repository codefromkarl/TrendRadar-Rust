# 契约文档目录

## 目标

这一组文档用于把 `docs/api-contracts.md` 中的模板级基线，下沉成可直接指导实现的模块级契约。

进入实现阶段后，所有可观察行为都应优先落到这里，再进入代码与测试。

## 使用方式

- 总体规则仍以 `../api-contracts.md` 为准
- 模块级字段、错误语义、fixture、快照和验证入口写入本目录
- 如果某个模块契约发生变化，应同轮更新对应实现文档和验收矩阵

## 文档索引

- [domain](./domain.md)
- [config](./config.md)
- [schedule](./schedule.md)
- [analyze](./analyze.md)
- [fetch](./fetch.md)
- [storage](./storage.md)
- [report](./report.md)

## 完成标准

一个模块的契约文档至少应补齐：

- 场景与边界
- 输入与输出
- 错误分类
- 兼容要求
- 对应 fixture、测试和快照入口
