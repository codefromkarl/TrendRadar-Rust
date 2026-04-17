# AGENTS

## 目标

这个文件定义 TrendRadar Rust 仓库内的 AI 协作边界。

目标不是限制正常开发，而是确保 AI 参与时：

- 能读懂当前工程约束
- 能执行固定命令
- 能验证自己的改动
- 不能绕过仓库边界失控修改

## 工作范围

当前阶段口径统一为：`v1.x` 迁移收口完成，项目已进入 `v2.x` 增量演进 / 生态扩展阶段。默认工作聚焦验证门禁维护、状态校准和后续增量任务准备。

当前进入增量演进阶段后，默认不做下面这些事情：

- 不再以“迁移尚未完成”为理由继续扩大收尾范围
- 不把真实远程对象存储 provider、真实远程 LLM provider、完整 MCP 协议兼容层和更大生态接入混入当前收尾任务
- 不经说明直接改写已经收口的核心 crate 边界

## 默认工作方式

- 先检索和理解，再编辑
- 优先复用现有文档结构和命令入口
- 结构性改动必须同步更新 `README.md`、`docs/` 或开发日志
- 所有改动都应留在当前 Git 工作树内
- 分支与提交命名应遵循 `docs/git-workflow.md`

## 允许执行的命令

默认允许：

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace --all-targets`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace`
- `cargo test --doc --workspace`
- `cargo nextest run --workspace --all-features`
- `cargo deny check`
- `cargo llvm-cov nextest --workspace --all-features`
- `cargo sweep --time <days>`
- `cargo sweep --all`
- `cargo watch -x test`
- `cargo watch -x check`
- `sccache --show-stats`
- `sccache --start-server`
- `just *`
- `./scripts/bootstrap.sh`
- `./scripts/check_environment.sh`
- `./scripts/doc_sync_reminder.sh`
- `git status`
- `git diff`
- `git switch`
- `git branch`

## 禁止范围

默认禁止：

- 修改生产 secrets 或本机敏感凭据
- 修改部署、账务、权限或外部系统管理脚本
- 绕过当前仓库去写无关目录
- 不经说明直接重写大量已有文档或代码
- 在没有验证的情况下声称迁移完成

## 文档同步要求

出现下面任一情况时，必须同步更新文档：

- 调整 workspace 或 crate 边界
- 调整验证命令、脚本或 CI 入口
- 新增或删除迁移基线规则
- 修改 AI 协作方式、任务切分方式或完成标准

优先更新的文档包括：

- `README.md`
- `docs/environment-setup.md`
- `docs/migration-strategy.md`
- `docs/module-map.md`
- `docs/invariants.md`
- `docs/api-contracts.md`
- `docs/dev-journal/`

## 任务切分协议

默认任务单位应为下面之一：

- 一份状态校准或收尾文档
- 一个验证入口或一组回归测试补强
- 一个 crate 的轻量边界修正
- 一个独立的增量演进或生态扩展任务

当前阶段仍不使用“整仓业务重写”作为任务单位。

## 完成标准

对当前收尾阶段，一个任务要被视为完成，至少应满足：

- 改动范围清楚
- 对应文档已同步
- 至少运行一条相关验证命令
- 最终结果可以通过 Git diff 审查
- 分支名和提交标题符合仓库 Git 规范

## 收尾后的默认边界

当前仓库已经具备并应继续保持：

- Git 工作树和基础分支流程
- 已收口的主链路迁移文档与模块边界
- 稳定的环境检查、验证和发布入口
- 系统性测试与 fixture 基线
- 后续任务默认按“增量演进”或“生态扩展”命名
