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
- 已落最小真实远程对象存储接入与本地验证原型

## 暂不处理

- 复杂迁移框架
- 远程对象存储的凭证治理与更多 provider 兼容细节

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
- `AppConfig.storage.backend` 已预留 `sqlite / s3` 切换位
- 已新增 `OpendalObjectStoreNewsRepository`，用统一对象布局打通真实 `s3/oss` provider
- 已保留 `FileObjectStoreNewsRepository` 与 `mock-s3` 路由，用于本地布局验证与无需云环境的回归测试
- `app` 当前已把 `storage.backend = "s3"` 路由到 `s3/aws-s3/oss/aliyun-oss/mock-s3`，并在缺失 bucket / endpoint 或 provider 不支持时显式报错，避免默默退回本地 SQLite
- 真实文件数据库路径与 schema 迁移框架仍留待后续阶段

## 验证命令

```bash
cargo test -p trendradar-storage
```
