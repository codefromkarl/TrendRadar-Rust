# `storage` 远程对象布局契约

## 文档目标

这份文档用于定义远程对象存储在 Rust 版中的最小可观察契约。

当前不实现真实远程 adapter，只先固定：

- 对象 key 如何组织
- 哪些对象是必须存在的
- 读取时先读什么、后读什么
- 去重与排序语义如何与本地 SQLite 对齐

## 适用范围

当前范围只覆盖未来 `storage.backend = "s3"` 一类对象存储后端的最小布局。

本轮不覆盖：

- 真实网络读写实现
- 凭证管理
- 增量同步优化
- 多 provider 差异细节

## 最小布局

远程对象布局暂定为“两层对象”：

1. 索引对象
2. 数据分片对象

### 1. 索引对象

- 作用：给读取方提供当前有哪些分片可读
- 建议 key：`<prefix>/index/latest.json`
- 内容：
  - schema version
  - backend type
  - shard 列表
  - 最新更新时间

### 2. 数据分片对象

- 作用：真正保存 `NewsItem` 列表
- 建议 key：`<prefix>/shards/<date>/<source>.json`
- 分片粒度：按日期和来源拆分
- 内容：
  - shard metadata
  - `NewsItem[]`

## 读取契约

读取最小顺序固定为：

1. 读取 `index/latest.json`
2. 根据索引列出的 shard keys 逐个读取 shard
3. 合并所有 `NewsItem`
4. 应用与 SQLite 一致的去重与排序规则

如果索引对象缺失，应直接报错，而不是静默返回空集合。

## 写入契约

未来真实实现时，最小写入顺序固定为：

1. 生成 shard 对象内容
2. 写入 shard 对象
3. 更新并写入 `index/latest.json`

不允许先改索引、后写 shard，否则读取方会看到不完整状态。

## 去重与排序契约

远程后端必须保持与本地 SQLite 一致：

- 去重键：`(source_id, title)`
- rank 冲突：保留更优 `rank`
- 最终排序：`rank ASC, source_id ASC, title ASC`

## 当前 fixture 约定

当前远程布局 fixture：

- `fixtures/system/storage/remote-layout-s3.json`

这个 fixture 的作用不是驱动真实远程 IO，而是先固定布局形状和字段命名。

## 当前测试入口

当前对应系统测试骨架：

- `tests/system/remote_storage_contract.rs`

## 当前结论

在真实远程 adapter 落地前，优先固定对象布局和读取顺序，比先写 mock adapter 更重要。

原因：

- mock adapter 会被布局设计反向约束
- 如果 key 设计先错，后续实现和 fixture 都会返工
- 先固定 layout，可以让 mock、真实实现和文档共用同一套契约
