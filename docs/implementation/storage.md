# `storage` 实施骨架

## 目标

建立本地持久化抽象，并补最小 SQLite 实现。

## 前置依赖

- `domain` 模型稳定

## 输入与输出

- 输入：统一内容模型
- 输出：稳定的写入、读取与去重行为

## 本轮范围

- 已收敛仓储 trait
- 已实现 SQLite 最小实现
- 已补固定 fixture 测试

## 暂不处理

- 远程存储
- 复杂迁移框架

## 建议子任务

- 去重策略
- schema 初始化
- 读写测试

## 完成定义

- 固定样例写入读取稳定
- 去重规则明确
- 错误语义可断言

## 当前进展

- 已提供 `SqliteNewsRepository::in_memory()`
- 已固定 `(source_id, title)` 去重并保留更优 `rank`
- 已补空仓库边界断言，固定初始读取结果为空集合
- 真实文件数据库路径与迁移框架仍留待后续阶段

## 验证命令

```bash
cargo test -p trendradar-storage
```
