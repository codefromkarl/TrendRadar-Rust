# 环境准备实施计划

## 文档目标

这份文档只描述 TrendRadar Rust 在进入功能迁移前的环境准备工作。

目标不是安排业务开发，而是明确：

- 进入迁移前必须完成哪些工程准备
- 每一项准备的完成标准是什么
- 应通过哪些命令验证

## 阶段边界

本计划只覆盖下面这些内容：

- Git 工作树和分支工作流
- Rust 工具链和扩展工具
- 仓库级 AI 协作规则
- 迁移基线文档
- 测试和 fixture 骨架
- 本地与 CI 验证入口

本计划不覆盖：

- 旧系统具体模块迁移
- 新业务功能实现
- 外部服务和通知能力接入

## 阶段 1：版本控制与审查基础

### 目标

让所有后续 AI 与人工改动都能在可审查、可回滚的工作树内完成。

### 任务

- 完成 `git init`
- 约定主分支与临时分支命名
- 约定文档同步检查的使用方式
- 约定提交标题格式和提交模板

### 完成标准

- 当前目录是有效 Git 工作树
- 可以通过 `git status`、`git diff` 检查改动
- 文档提醒脚本可运行
- Git hooks 和提交模板可以启用

### 验证命令

```bash
git status --short
./scripts/doc_sync_reminder.sh
./scripts/install_githooks.sh
```

## 阶段 2：工具链与本机环境统一

### 目标

确保所有参与者在统一的 Rust 工具链和验证命令下工作。

### 任务

- 固定 `rust-toolchain.toml`
- 维护 `scripts/bootstrap.sh`
- 提供环境检查脚本
- 在 `justfile` 中收口环境命令

### 完成标准

- 工具链组件要求明确
- 本机可一键检查关键命令是否就绪
- 准备阶段和完整阶段的验证入口分开

### 验证命令

```bash
./scripts/check_environment.sh
just env-check
```

## 阶段 3：仓库级规则和迁移基线

### 目标

把 AI 原生协作所依赖的规则从“会话习惯”变成“仓库文件”。

### 任务

- 维护 `AGENTS.md`
- 维护 `docs/module-map.md`
- 维护 `docs/invariants.md`
- 维护 `docs/api-contracts.md`

### 完成标准

- 仓库内存在稳定的 AI 协作规则入口
- 迁移边界、约束和契约模板已经落地
- 文档之间没有明显冲突

### 验证方式

- 文档自查
- `./scripts/doc_sync_reminder.sh`

## 阶段 4：测试骨架与样例目录

### 目标

在功能迁移前，先建立稳定的测试模板、fixture 目录和最小样例。

### 任务

- 维护 `tests/README.md`
- 维护 `tests/system/README.md`
- 维护 `fixtures/README.md`
- 维护 `fixtures/system/README.md`
- 维护 `docs/system-test-template.md`

### 完成标准

- 系统性测试目录结构稳定
- fixture 目录约束清楚
- 至少有一个真实样例能证明模板不是空壳

### 验证命令

```bash
cargo test --workspace
```

## 阶段 5：验证门禁收口

### 目标

把“本地能做的检查”和“完整门禁检查”分开，避免环境未装齐时误报。

### 任务

- 维护 `justfile`
- 区分基础验证与完整验证命令
- 让 README 和环境文档引用这些入口

### 完成标准

- 基础命令适用于当前初始化阶段
- 完整命令适用于工具安装完成后的阶段
- 文档说明和命令行为一致

### 验证命令

```bash
just verify-basic
just verify
```

## 进入功能迁移前的 Definition of Ready

只有下面条件同时满足，才建议进入具体功能迁移：

- Git 工作树可用
- 工具链和关键扩展工具安装路径明确
- 环境检查脚本可运行
- AGENTS 规则已落地
- 迁移基线文档已成形
- 测试与 fixture 骨架已成形
- 基础验证命令持续可通过

## 当前状态

截至当前阶段，环境准备已经完成了大部分骨架，但仍应继续保持：

- 规则先行
- 文档同步
- 验证先于声称完成

## 后续衔接

当上面的 Definition of Ready 满足后，后续功能迁移不再继续写入本文件，而应转到：

- [并行迁移总方案](./parallel-migration-plan.md)

这样可以保持“环境准备计划”和“功能迁移计划”分离，避免不同阶段的目标混在同一份文档里。
