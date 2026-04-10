# 开发记录：环境约束、命令分层与实施计划

## 基本信息

- 日期：2026-04-10
- 阶段：环境准备收口
- 主题：补仓库级 AGENTS 规则、实施计划与环境检查入口
- 目标：在不进入功能迁移的前提下，把工程环境准备成可审查、可验证、可持续收敛的状态

## 本次完成内容

- 新增仓库级 `AGENTS.md`
- 新增 `docs/implementation-plan.md`
- 新增 `scripts/check_environment.sh`
- 在 `justfile` 中补 `bootstrap`、`env-check`、`test-basic`、`test-doc`、`verify-basic`
- 在 `README.md` 和 `docs/environment-setup.md` 中补环境准备入口和命令分层说明
- 把 `scripts/bootstrap.sh` 收口为与 `Rust 1.85.0` 兼容的工具安装脚本
- 在本机实际安装 `just`、`cargo-nextest`、`cargo-deny`、`cargo-llvm-cov`
- 新增仓库本地 `pre-commit` 和 `pre-push` hooks 以及安装脚本
- 新增 Git 分支与提交规范文档、提交模板和 `commit-msg` hook

## 关键决策

### 决策 1

- 决策内容：当前阶段只做环境准备，不进入功能迁移
- 原因：如果先把工程边界和验证入口收口清楚，后续迁移时返工会更少

### 决策 2

- 决策内容：把 AI 规则落成仓库内文件，而不是只靠会话约束
- 原因：会话规则会丢失，仓库规则更适合作为长期协作入口

### 决策 3

- 决策内容：把基础验证和完整验证拆开
- 原因：环境尚未完全装齐时，不能让完整门禁误导为“仓库不可开发”

## 阶段结论

到这一轮为止，仓库已经具备下面这些环境层能力：

- Git 工作树可用
- AI 协作规则可在仓库内直接读取
- 环境准备工作有明确实施计划
- 环境检查、本地基础验证和完整验证三类命令已经分层
- `bootstrap.sh` 已经能在固定 `Rust 1.85.0` 下安装兼容版本的扩展工具
- `just env-check` 和 `just verify-basic` 已经通过
- 当前仓库已支持终端工作流下的本地 Git 自动检查
- 当前仓库已支持并行迁移场景下的提交标题校验与提交模板约束

## 安装与验证记录

### 本机安装结果

- `just 1.49.0`
- `cargo-nextest 0.9.100`
- `cargo-deny 0.18.3`
- `cargo-llvm-cov 0.6.21`

### 为什么锁这些版本

当前仓库固定在 `Rust 1.85.0`。

如果直接安装这些工具的最新版，会碰到它们各自更高的 MSRV 要求，因此本轮把 `bootstrap.sh` 改成安装兼容版本，而不是强行抬高仓库 toolchain。

### 本轮验证命令

```bash
just env-check
just install-githooks
just verify-basic
```

### 本轮验证结果

- `just env-check` 通过
- `just install-githooks` 可启用仓库自带 hooks
- `just verify-basic` 通过
- 工作区 `fmt`、`check`、`test` 在当前固定工具链下可重复执行
- Git 提交标题可由 `commit-msg` hook 校验

## 下一步

- 在首个基线提交后开始按分支流程工作
- 保持当前阶段只做环境准备，不提前进入功能迁移
