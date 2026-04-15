# `storage` 契约骨架

## 模块目标

定义本地持久化接口、去重规则、查询最小集合和持久化错误语义。

## 当前实现基线

- `NewsRepository` trait
- `SqliteNewsRepository`

## 需要固化的契约

### 1. 写入契约

- 写入对象：
  `NewsItem`
- 主键或去重键：
  `(source_id, title)`
- 幂等要求：
  重复写入相同来源与标题时不新增记录，并保留更优 `rank`

### 2. 查询契约

- 最小读取接口：
  `list_news() -> Vec<NewsItem>`
- 查询过滤条件：
  Wave 1 暂不引入
- 排序与分页需求：
  当前固定按 `rank ASC, source_id ASC, title ASC`

### 3. 持久化后端

- 首版后端：
  SQLite
- 远程后端：
  已实现基于对象布局契约的最小 `S3/OSS` adapter，并保留 `mock-s3` 文件系统原型用于本地验证
- 连接与初始化方式：
  `SqliteNewsRepository::in_memory()` 用于测试与最小闭环
- 远程对象存储入口：
  `storage.backend = "s3"`，由 `provider` 路由到真实 `s3/aws-s3/oss/aliyun-oss` 或本地 `mock-s3`
- schema 管理方式：
  由仓储在初始化时建表

## 错误契约

- 建库失败：
  进入 `TrendRadarError::Storage`
- 写入失败：
  进入 `TrendRadarError::Storage`
- 去重冲突行为：
  不报错，更新为更优 `rank`

## 验证方式

- fixture：
  `fixtures/system/storage/news-roundtrip-input.json`
- 测试：
  `cargo test -p trendradar-storage`
- 空仓库边界：
  新建仓库时检查 `list_news()` 返回空集合
- 快照：
  当前不需要，行为由结构断言覆盖

## 待补充决策

- `NewsItem` 与 `RssItem` 是否共表或分表
- schema 迁移是否在首版纳入范围
- 远程对象存储的凭证治理、provider 兼容细项和失败回退策略
