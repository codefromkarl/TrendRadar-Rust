# Rust 参考与版本固定策略

## 当前固定版本

当前项目将 Rust toolchain 固定为 `1.94.1`。

固定方式：

- `rust-toolchain.toml` 中将 `channel` 设为 `1.94.1`
- 同时纳入 `rustfmt`、`clippy`、`rust-analyzer`、`rust-src`、`rust-docs`

这样做的目的不是追求最新，而是保证：

- 本地开发、AI 参考、CI 约束指向同一版本
- Rust 文档引用和项目代码所面对的版本一致
- 不因为在线 `stable` 文档前进而产生版本漂移

## 本地 Rust 文档

当前项目选择将 `rust-docs` 作为固定 toolchain 的组成部分。

这意味着本地会存在与 `1.94.1` 对应的一套离线官方文档，而不是完全依赖在线 `stable` 页面。

项目内额外补充了一个辅助脚本：

- `scripts/rust_doc_paths.sh`

它的作用不是安装文档，而是输出当前固定版本下本地文档的位置，用于确认本地可参考的官方文档入口。

## 参考优先级

当前项目约定的 Rust 参考优先级如下：

1. 当前固定版本的本地官方文档
2. 官方语言参考和 edition 相关文档
3. 在线官方文档
4. `docs.rs`
5. 社区文章、博客、问答

这样安排的原因是：

- 项目代码必须先对齐固定版本
- 在线 `stable` 文档可能领先于当前固定版本
- `docs.rs` 适合查第三方 crate，不适合作为 Rust 语言本身的唯一权威来源

## SessionStart 提醒

当前仓库已经增加了一个 repo 级 SessionStart 提醒脚本：

- `scripts/session_start_reminder.py`

它的作用是：

- 在进入仓库会话时提醒当前固定 Rust 版本
- 提醒中大变更先改文档，小 bug 可先修代码
- 提醒优先参考本地固定版本文档，而不是直接看在线 `stable`

这不是最终校验，而是开发动作发生前的前置提醒。
