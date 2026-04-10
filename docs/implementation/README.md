# 实施文档目录

## 目标

这一组文档用于把总方案拆成可直接分配给实现者或子代理的工作单。

每份文档只回答五个问题：

- 这个模块要做什么
- 依赖谁
- 本轮做什么，不做什么
- 怎样算完成
- 用什么命令验证

## 文档索引

- [domain](./domain.md)
- [config](./config.md)
- [schedule](./schedule.md)
- [analyze](./analyze.md)
- [fetch](./fetch.md)
- [storage](./storage.md)
- [report](./report.md)
- [app](./app.md)

## 使用规则

- 先看 `../parallel-migration-plan.md` 确定波次，再看对应模块实施文档
- 先补契约，再开实现
- 每个模块完成后，同轮更新验收矩阵
