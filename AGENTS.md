# AGENTS

## 目标

这个文件定义 TrendRadar Rust 仓库内的 AI 协作边界。

目标不是限制正常开发，而是确保 AI 参与时：

- 能读懂当前工程约束
- 能执行固定命令
- 能验证自己的改动
- 不能绕过仓库边界失控修改

## 工作范围

当前阶段只允许进行环境准备、文档完善、测试骨架和工程约束收敛。

在明确进入功能迁移阶段之前，默认不做下面这些事情：

- 不迁移旧系统业务逻辑
- 不扩真实抓取、存储、报告和通知功能
- 不引入与当前迁移阶段无关的外部服务接入

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

- 一个环境脚本
- 一份规则文档
- 一个验证入口
- 一条系统测试骨架
- 一个 crate 的非业务性工程整理

当前阶段不使用“整仓业务重写”作为任务单位。

## 完成标准

对当前环境准备阶段，一个任务要被视为完成，至少应满足：

- 改动范围清楚
- 对应文档已同步
- 至少运行一条相关验证命令
- 最终结果可以通过 Git diff 审查
- 分支名和提交标题符合仓库 Git 规范

## 进入功能迁移前的门槛

在进入具体功能迁移前，仓库应至少具备：

- Git 工作树和基础分支流程
- 明确的迁移基线文档
- 明确的 AI 协作规则
- 稳定的环境检查和验证命令入口
- 最小系统性测试模板和样例目录
