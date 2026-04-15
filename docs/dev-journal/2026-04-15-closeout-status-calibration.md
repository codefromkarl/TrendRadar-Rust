# 开发记录：迁移收尾状态校准

## 基本信息

- 日期：2026-04-15
- 阶段：迁移收尾
- 主题：统一收尾口径、活文档状态与验证数据
- 目标：把仓库当前“核心迁移已闭环”的事实同步到 AI 协作规则、README、路线图、验收矩阵和开发日志中，避免继续沿用环境准备期口径

## 背景

仓库代码、测试和路线图已经表明主链路迁移完成，可以进入收尾与后续演进阶段。

但活文档仍残留几类明显漂移：

- `AGENTS.md` 仍按环境准备期约束 AI 工作范围
- `README.md` 仍写仓库处于环境准备阶段
- 部分统计数据仍停留在旧的测试数量
- `module-map`、`api-contracts` 等文档还保留“首版不迁移”或“尚未完整实现”的早期表述

如果这些文档不一起校准，后续任务会持续在“收尾已完成”和“迁移尚未开始”两套口径之间来回摆动。

## 本次完成内容

- 更新 `AGENTS.md`，把默认阶段从环境准备改为迁移收尾校准与后续增量演进准备
- 更新 `README.md`，同步当前仓库阶段与最新测试数量
- 更新 `docs/roadmap.md`，把状态口径切到“收尾完成后的后续演进”
- 更新 `docs/acceptance-matrix.md`，同步 `app` 状态描述和根级系统测试数量
- 更新 `docs/module-map.md`，补上最小 AI / MCP 已落地的事实，并把 AI 翻译单独标为延后
- 更新 `docs/api-contracts.md` 与 `docs/environment-setup.md`，移除“当前尚未实现 / 当前仍在环境准备”的过期前提
- 把 `docs/project-closeout.md` 明确纳入本轮收尾文档集合，作为 README 和路线图口径的直接依据
- 修正 `crates/ai`、`crates/storage`、`crates/app` 与 `crates/app/tests/wave4_http_pipeline.rs` 中阻塞 `clippy` 的测试写法
- 对齐 `clippy.toml` 与工作区 `MSRV` 版本口径，移除 `clippy` 启动告警

## 关键决策

### 决策 1

- 决策内容：这轮收尾只做状态校准，不扩新功能
- 原因：当前问题主要是文档与代码口径漂移，而不是能力缺口
- 备选方案：借收尾顺手继续补生态能力
- 为什么没有选备选方案：这会再次扩大收尾范围，破坏本轮定稿边界

### 决策 2

- 决策内容：优先修正 AI 协作规则和对外状态文档
- 原因：`AGENTS.md`、`README.md` 和路线图会直接影响后续任务定义
- 备选方案：只改 README，不动仓库规则文档
- 为什么没有选备选方案：这样只能修表面口径，AI 与开发流程仍会继续按旧阶段工作

## 遇到的问题

### 问题 1

- 现象：测试总数和系统测试数量在不同文档中的表述不一致
- 原因判断：文档没有随着后续补测同步更新
- 处理方式：重新执行工作区测试，并单独核对根级系统测试条数
- 最终结果：当前工作树下确认全工作区为 232 tests、根级 `tests/system/` 为 68 条测试

### 问题 2

- 现象：部分文档还保留“当前还没有完整实现”“首版不迁移”这类早期表述
- 原因判断：这些文档建立得较早，后续没有随 Wave 6~8 和收尾阶段同步修订
- 处理方式：把“总入口文档”改成历史定位或抽象索引，并把已落地的最小实现补入映射
- 最终结果：活文档与当前代码状态更一致

## 关键文件 / 关键操作记录

### 关键文件

- `AGENTS.md`
- `README.md`
- `docs/roadmap.md`
- `docs/acceptance-matrix.md`
- `docs/project-closeout.md`
- `docs/module-map.md`
- `docs/api-contracts.md`
- `docs/environment-setup.md`

### 本阶段涉及的关键操作

- 重新核对工作树状态与活文档差异
- 重新统计工作区测试总数和根级系统测试数量
- 统一收尾、演进与生态扩展的命名边界
- 运行 `cargo fmt --all --check`
- 运行 `cargo check --workspace --all-targets`
- 运行 `cargo test --workspace`
- 运行 `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## AI 协作记录

### 本阶段用到的 skills

- `repo-search`

### 本阶段的上下文管理方式

- 先用 ContextAtlas 检索迁移进度相关文档
- 再用定点文件阅读和 `git diff` 校对活文档漂移

### 本阶段的代码索引方式

- 优先使用 ContextAtlas 混合检索
- 用 `rg` 补充阶段口径和统计数字的精确定位

### 对 AI 协作的观察

- 收尾阶段最容易失真的不是代码，而是文档里的阶段假设
- 如果不先统一“当前到底处在哪个阶段”，后续任务会持续错误切分

## 阶段结论

这一轮收尾校准之后，仓库的默认阶段口径已经从“环境准备”切换为“核心迁移已闭环，后续转增量演进 / 生态扩展”。

当前仍未完成的内容主要是更完整的生态能力，而不是这轮迁移收尾本身。

本轮收尾验证结果为：

- `cargo fmt --all --check` 通过
- `cargo check --workspace --all-targets` 通过
- `cargo test --workspace` 通过
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过

## 下一步

- 运行 `fmt/check/test/clippy` 形成收尾验证证据
- 视工作树边界决定是否整理成单独的收尾提交
- 后续新增任务按“增量演进”或“生态扩展”命名
