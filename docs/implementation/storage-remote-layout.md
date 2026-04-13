# `storage` 远程对象布局实施说明

## 目标

把远程对象存储的第一步从“实现远程写入”调整为“固定对象布局和测试骨架”。

## 本轮范围

本轮只做下面这些内容：

- 固定索引对象与 shard 对象的最小布局
- 固定读取顺序
- 补 fixture
- 补一个只验证布局形状的系统测试骨架

## 本轮不做

- 真实 S3/OSS 读写
- provider SDK 接入
- 凭证加载
- 重试与回退逻辑

## 关键取舍

当前先做“对象布局设计”，而不是先做 mock adapter。

原因：

- 先设计布局，后续 mock 和真实实现都能复用同一契约
- 如果直接做 mock adapter，容易把错误的 key 设计过早固化
- 当前仓库更缺的是可复查的远程存储边界，而不是一个看似可跑但契约不稳的假实现

## 本轮产出

- `docs/contracts/storage-remote-layout.md`
- `fixtures/system/storage/remote-layout-s3.json`
- `tests/system/remote_storage_contract.rs`

## 后续建议

完成这一步后，下一轮再做：

1. mock adapter 或内存远程仓储原型
2. 读取 / 写入错误路径测试
3. 再评估真实 provider 接入
