# TrendRadar Rust

TrendRadar 的 Rust 工作区重构版本。

这个仓库的目标不是把旧 Python 项目逐文件翻译成 Rust，而是先建立一个可验证、可维护、适合 AI 协作推进的 Rust 内核，再逐步迁移抓取、调度、分析、存储和输出链路。

## 当前定位

- 项目类型：Rust workspace 迁移仓库
- 当前阶段：环境准备和迁移基线收敛
- 主要目标：先把工程环境、验证闭环和 Git 协作规则收口，再进入功能迁移

如果你现在关心的是如何开始而不是实现细节，建议先看：

- [环境准备](./docs/environment-setup.md)
- [实施计划](./docs/implementation-plan.md)
- [AI 协作规则](./AGENTS.md)
- [Git 工作流与提交规范](./docs/git-workflow.md)

## 为什么单独做 Rust 重构

这个仓库明确基于原始 Python 版 TrendRadar 的迁移与重构工作展开。

迁移目标不是保留所有历史行为，而是：

- 保留真正构成产品核心价值的链路
- 删除历史包袱和高复杂度低收益的兼容层
- 用 Rust 重建更清晰的模块边界和验证入口
- 让 AI 和人工都能在同一套规则下推进迁移

迁移总体思路见：[迁移策略](./docs/migration-strategy.md)。

## 当前已经具备的基础

- 多 crate workspace 骨架
- Rust `1.85.0` 固定工具链
- `fmt`、`clippy`、`check`、`test` 的命令入口
- CI 基础验证
- 迁移基线文档
- 系统性测试模板和最小样例
- 终端工作流可用的 Git hooks
- 并行迁移可用的分支与提交规范

这意味着仓库已经从“能放代码的目录”进入了“可审查、可验证、可并行协作的迁移工地”状态。

## 快速开始

### 1. 安装工具链和本地工具

```bash
rustup set profile default
rustup toolchain install 1.85.0 --component rustfmt --component clippy --component rust-analyzer --component rust-src --component rust-docs
rustup default 1.85.0
cargo install just
cargo install cargo-nextest --version 0.9.100 --locked
cargo install cargo-llvm-cov --version 0.6.21 --locked
cargo install cargo-deny --version 0.18.3 --locked
bash ./scripts/install_githooks.sh
```

也可以直接执行：

```bash
bash ./scripts/bootstrap.sh
```

### 2. 检查本地环境

```bash
just env-check
```

### 3. 跑基础验证

```bash
just verify-basic
```

## 常用命令

```bash
just env-check
just install-githooks
just fmt
just fmt-check
just check
just lint
just test-basic
just test
just verify-basic
just verify
```

## 仓库结构

- `crates/domain`
  领域模型、共享错误、运行元数据
- `crates/config`
  配置模型、默认值和加载入口
- `crates/schedule`
  调度规则解析
- `crates/analyze`
  过滤、聚合、排序等纯逻辑
- `crates/storage`
  存储抽象和后续本地持久化实现
- `crates/fetch`
  热点源和 RSS 抓取适配
- `crates/report`
  结构化输出和后续报告层
- `crates/app`
  编排与 CLI 入口
- `docs/`
  迁移策略、环境准备、规则和架构文档
- `fixtures/`
  系统测试样例目录
- `.githooks/`
  本地 `pre-commit`、`commit-msg`、`pre-push` hooks

## 文档入口

- [架构说明](./docs/architecture.md)
- [迁移策略](./docs/migration-strategy.md)
- [环境准备](./docs/environment-setup.md)
- [模块映射基线](./docs/module-map.md)
- [迁移不变量](./docs/invariants.md)
- [契约基线](./docs/api-contracts.md)
- [系统性测试模板](./docs/system-test-template.md)
- [Git 工作流与提交规范](./docs/git-workflow.md)

## Git 与并行迁移

当前仓库已经约定：

- 使用仓库内 `.githooks/` 做本地自动检查
- 使用 `.gitmessage` 统一提交模板
- 使用 `<type>(<scope>): <summary>` 统一提交标题
- 使用 `<track>/<scope>-<topic>` 统一分支命名

如果后续要并行推进多个迁移任务，先按 crate、契约、fixture 或环境脚本切分，不要混写成“大分支、大提交”。

详细规则见：[Git 工作流与提交规范](./docs/git-workflow.md)。

## License

当前仓库沿用 `GPL-3.0-only` 许可策略。

原因是该项目明确属于从 TrendRadar 迁移而来，迁移、重构和后续实现都应在与源项目一致的许可约束下进行，直到后续有明确的法律和项目治理结论为止。
