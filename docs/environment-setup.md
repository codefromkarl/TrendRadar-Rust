# 环境准备

## 文档目标

这份文档用于记录 TrendRadar Rust 在正式进入业务开发前，围绕环境、工具链、工程约束和 AI 协作所做的准备工作。

核心目标有两个：

- 让所有参与者在同一套工具链和验证规则下开发
- 为后续 AI 辅助开发提供稳定、可重复、可验证的工程环境

配套规则和实施路径还见：

- [../AGENTS.md](../AGENTS.md)
- [implementation-plan.md](./implementation-plan.md)
- [git-workflow.md](./git-workflow.md)

## 项目背景

TrendRadar Rust 不是一个全新项目，而是从原始 Python 项目 TrendRadar 迁移而来。

因此在环境准备阶段，需要同时满足两类要求：

- Rust 工程自身的开发与验证要求
- 迁移项目对结构清晰、行为可验证、文档完整的要求

## 本阶段完成的环境准备

### Git 工作树准备

当前仓库已经完成 `git init`，具备了后续在隔离分支中开发、审查 diff 和执行文档同步提醒脚本的基本前提。

后续应继续坚持：

- 所有 AI 改动都在 Git 工作树内完成
- 结构性变更优先通过分支和 diff 审查
- 文档提醒脚本依赖 Git 变更范围，不应脱离工作树使用

建议后续采用下面的分支约定：

- `main`：环境准备和后续迁移的主分支
- `env/*`：环境与工程约束调整
- `docs/*`：文档与规则收敛
- `migration/*`：未来进入功能迁移后的工作分支

更完整的命名和提交规范见：

- [git-workflow.md](./git-workflow.md)

### Rust 工具链约束

本阶段已经通过 `rust-toolchain.toml` 固定了工作区使用的 Rust 工具链和组件：

```toml
[toolchain]
channel = "1.94.1"
components = ["rustfmt", "clippy", "rust-analyzer", "rust-src", "rust-docs"]
```

这意味着：

- 本地开发默认使用 `1.94.1`
- 格式化、lint、编辑器补全使用统一组件
- 本地离线 Rust 文档被纳入固定工具链
- 不依赖开发者手动记忆要安装哪些 Rust 组件

### 扩展工具准备

本阶段已经把以下扩展工具纳入项目准备范围：

**质量门禁工具：**

- `just`
- `cargo-nextest`
- `cargo-llvm-cov`
- `cargo-deny`

**编译性能工具：**

- `cargo-sweep` — 清理旧编译缓存，替代 `cargo clean`
- `sccache` — 跨项目共享依赖编译缓存
- `cargo-watch` — 监听文件变化，自动增量编译和测试
- `mold` — 高性能链接器（系统级安装），Debug 模式链接提速 50%+

同时补充了 `scripts/bootstrap.sh`，用于统一记录这些工具的准备方式。当前脚本内容如下：

```bash
#!/usr/bin/env bash
set -euo pipefail

rustup set profile default
rustup toolchain install 1.94.1 --component rustfmt --component clippy --component rust-analyzer --component rust-src --component rust-docs
rustup default 1.94.1

cargo install just
cargo install cargo-nextest --version 0.9.100 --locked
cargo install cargo-llvm-cov --version 0.6.21 --locked
cargo install cargo-deny --version 0.18.3 --locked

# Build performance tools
cargo install cargo-sweep
cargo install sccache
cargo install cargo-watch

bash ./scripts/install_githooks.sh
```

这里显式锁定兼容版本，是因为当前仓库固定在 `Rust 1.94.1`。如果直接安装这些工具的最新版，仍可能碰到它们各自额外的 MSRV 或兼容性变化。

如果本机只想先完成环境准备，而不立即安装全部扩展工具，可以先通过环境检查脚本确认缺口：

```bash
./scripts/check_environment.sh
```

### Git hooks 自动检查

考虑到当前主要通过终端工作，而不是依赖编辑器，本仓库补充了本地 Git hooks：

- `.githooks/pre-commit`
  提交前执行 `cargo fmt --all --check`
- `.githooks/commit-msg`
  检查提交标题是否符合 `<type>(<scope>): <summary>`
- `.githooks/pre-push`
  推送前执行 `just verify-basic`

同时补充：

- `.gitmessage`
  作为提交消息模板，统一 Why / What / Verify 结构

启用方式：

```bash
./scripts/install_githooks.sh
```

或者：

```bash
just install-githooks
```

这个安装过程只会把当前仓库的 `core.hooksPath` 指向 `.githooks`，不会修改全局 Git 配置。
同时会把当前仓库的 `commit.template` 指向 `.gitmessage`。

### 当前本机已知状态

在当前环境准备阶段，当前机器已经补齐仓库约定的基础工具和扩展工具：

- `just`
- `cargo-nextest 0.9.100`
- `cargo-deny 0.18.3`
- `cargo-llvm-cov 0.6.21`
- `cargo-sweep 0.8.0`
- `sccache 0.14.0`
- `cargo-watch 8.5.3`
- `mold`（系统级安装）

同时，`just env-check`、`just verify-basic` 与 `just doc` 已经在当前机器验证通过。

### 命令入口分层

为了避免“环境准备阶段”和“完整门禁阶段”混在一起，当前 `justfile` 已经分成两层入口：

- `just env-check`
  用于检查 Git、Rust 工具链、组件和扩展命令是否就绪
- `just install-githooks`
  用于启用当前仓库自带的 `pre-commit` 和 `pre-push`
- `just doc`
  用于生成当前 workspace 的本地 API 文档，不包含依赖 crate
- `just verify-basic`
  用于在初始化阶段跑基础格式化、检查和工作区测试
- `just verify`
  用于在扩展工具安装完成后跑完整门禁

这些入口分别对应：

- 环境是否齐备
- 本地提交和推送前是否自动拦截基础问题
- 当前 workspace 文档是否可以在本地生成并进入 `target/doc`
- 当前改动是否具备基础可验证性
- 是否达到完整门禁标准

### 系统性测试基线

当前仓库已经不再只有测试模板，也已经落下了第一条真实的系统性测试链路：

- fixture：`fixtures/system/config/minimal-valid.json`
- fixture：`fixtures/system/config/invalid-empty-timezone.json`
- 测试：`crates/app/tests/config_to_bootstrap.rs`

这条链路当前覆盖的是：

1. 从稳定 JSON fixture 读取配置
2. 通过 `trendradar-config` 解析并校验配置
3. 进入 `trendradar-app` 的基础 bootstrap 校验入口

这意味着系统性测试在当前仓库里已经从“约定”变成了“真实可执行入口”。

后续新增系统性测试时，应继续保持下面的模式：

- 先补 fixture
- 再补跨 crate 或跨函数链路测试
- 最后把它纳入统一验证命令

当前最小验证命令仍然是：

```bash
cargo test --workspace
```

如果后续安装了 `cargo-nextest` 和 `cargo-llvm-cov`，则应进一步纳入：

```bash
cargo nextest run --workspace --all-features
cargo llvm-cov nextest --workspace --all-features
```

## 编辑器建议

建议使用支持 `rust-analyzer` 的编辑器。

仓库当前已经提供了基础 VS Code 配置：

- [settings.json](../.vscode/settings.json)

当前配置的实际内容如下：

```json
{
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.check.allTargets": true,
  "rust-analyzer.cargo.targetDir": true
}
```

主要目标：

- 保存时使用统一检查逻辑
- 避免不同开发者本地行为不一致
- 提高 AI 与人工协作时的反馈速度

这三项配置分别表示：

- `check.command = "clippy"`
  编辑器内直接用 `clippy` 做静态检查，而不是只跑 `cargo check`
- `check.allTargets = true`
  单元测试、集成测试等目标也纳入检查范围
- `cargo.targetDir = true`
  让 `rust-analyzer` 使用独立的 target 目录，减少和手工命令抢锁

### 工程约束准备

当前根 `Cargo.toml` 中已经约束了 workspace 成员、Rust 版本和 lint 策略：

```toml
[workspace]
members = [
    "crates/domain",
    "crates/config",
    "crates/schedule",
    "crates/analyze",
    "crates/storage",
    "crates/fetch",
    "crates/report",
    "crates/notification",
    "crates/app",
]
resolver = "2"

[workspace.package]
edition = "2024"
rust-version = "1.94"
license = "GPL-3.0-only"

[profile.dev]
debug = 1

[profile.release]
lto = "thin"
codegen-units = 1
strip = "debuginfo"

[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[workspace.lints.clippy]
all = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "deny"
todo = "deny"
panic = "deny"
```

这些配置说明，本阶段已经把以下约束显式落地：

- 工作区已拆分为多 crate 结构（含 9 个成员 crate）
- Rust edition 固定为 `2024`
- 最低 Rust 版本按 `1.94` 管理
- 默认不允许 `unsafe`
- `unwrap`、`expect`、`todo!`、`panic!` 被纳入默认限制
- Debug profile 仅保留最小调试信息（`debug = 1`）
- Release profile 使用 thin LTO、单 codegen unit、去除调试符号

## AI 协作准备

为了让 AI 在 Rust 迁移过程中输出更稳定、上下文更完整、搜索更准确，当前项目做了三类准备：

除此之外，当前仓库还把 AI 协作规则显式落到了根目录 `AGENTS.md`，用于定义：

- 当前阶段允许做什么
- 默认允许执行哪些命令
- 哪些目录和改动属于禁止范围
- 一个环境准备任务的完成标准是什么

### 1. 针对 Rust 迁移的 skills 准备

当前优先使用的能力主要包括：

- `migration-context`
  用于先判断旧 Python 项目的能力应该如何迁移、保留还是重设计
- `repo-search`
  用于基于语义检索理解当前仓库结构、模块职责和入口位置
- `test-driven-development`
  用于强制先建立测试再写实现，减少迁移时的行为漂移
- `verification-before-completion`
  用于在声称“完成”前，强制执行格式化、lint、测试等验证步骤

这几类能力对应的是迁移场景中最容易失控的四个问题：

- 迁移边界不清
- 代码定位不准
- 改动没有测试保护
- 没验证就误判完成

### 2. 当前上下文管理方式

当前项目使用 ContextAtlas 作为主要的上下文管理基础。

当前做法是：

- 每次会话开始，先执行一次语义代码检索
- 优先使用 `codebase-retrieval` 理解仓库结构、入口、模块职责
- 仅在语义检索不足时，再使用 `rg` 等精确搜索补充
- 长期只记录“不可从代码直接推导”的信息，避免把路径、接口列表、临时搜索结果写成长期记忆

这套策略的目标是：

- 避免 AI 每轮都从零开始理解项目
- 避免把临时结构误当成稳定知识
- 让上下文既可持续，又不过度污染

### 3. 当前代码索引方式

当前代码索引以 ContextAtlas 的混合检索为主：

- 语义检索：理解“这段代码是做什么的”
- 词法检索：理解“这个符号或关键字出现在哪里”

实际使用顺序是：

1. 先用 `codebase-retrieval` 做语义定位
2. 再用 `rg` 做精确匹配和补充确认
3. 对关键入口、核心模块和重要文档进行人工复核

这种索引方式特别适合迁移项目，因为迁移不是单纯找文件，而是要回答下面三类问题：

- 旧模块在新系统里还保不保留
- 哪些逻辑是核心，哪些是冗余
- 迁移时应该先动哪一层

## AI 协作原则

当前 AI 协作遵循以下原则：

- 先分析边界，再动手实现
- 先明确迁移取舍，再进入编码
- 优先让验证命令说话，而不是主观判断
- 让搜索、文档、测试形成闭环

这些原则对 Rust 项目尤其重要，因为 Rust 的类型系统虽然能暴露很多问题，但并不能替代架构边界、迁移判断和行为验证。

## 开发过程中的保障策略

### 1. 如何保证文档与代码一致

当前项目把“文档与代码一致”视为工程约束，而不是写完代码后的补充动作。

本阶段已经确立的做法是：

- 结构性调整发生时，同时更新 `README.md`、架构文档、迁移文档或开发日志
- 开发日志不记录抽象口号，而记录本阶段真实发生的变更、决策和问题
- 文档优先描述“当前系统实际是什么”，而不是“理想上应该是什么”
- 与代码强绑定的信息尽量写在最接近代码边界的文档中，例如环境约束写在环境文档、迁移取舍写在迁移文档

后续应继续坚持两条规则：

- 代码结构变了，文档必须同轮更新
- 如果某段文档无法从当前仓库结构得到印证，就说明它需要被重写或删除

这意味着文档不是独立产物，而是开发过程的一部分。

### 2. 如何保证代码风格检查尽量在编译前暴露

Rust 里很多检查天然会走编译流程，但当前项目已经尽量把“能前置的检查”前置了。

本阶段已经准备好的约束包括：

- `rustfmt` 作为统一格式来源
- `clippy` 作为默认静态检查入口
- `rust-analyzer` 配置成优先使用 `clippy`
- `justfile` 和 CI 都围绕统一的检查入口组织

这里的核心不是追求“绝对不编译”，而是追求：

- 尽早在编辑器内看到风格和静态问题
- 尽早在本地门禁里暴露问题
- 不把明显的风格或低级问题拖到长时间构建之后才发现

因此当前项目采用的思路是：

- 编辑器层先暴露
- 本地检查层再收口
- CI 层做最终兜底

### 3. 如何保证 Rust 编译速度

当前项目并不把"最快编译"当作唯一目标，而是把"结构清晰 + 编译反馈可接受"作为平衡点。

本阶段已经在结构上做了几件对编译速度有帮助的准备：

- 采用 workspace + 多 crate，而不是所有逻辑塞进一个 crate
- 使用 `resolver = "2"`，减少不必要的特性传播
- 在编辑器侧启用独立 `target`，降低索引和手工构建互相抢占
- 把纯逻辑模块和编排模块分开，方便后续只重编译局部 crate
- `[profile.dev] debug = 1` — 最小调试信息，加速 Debug 编译
- `[profile.release] lto = "thin"` — 平衡 Release 编译时间与二进制体积

后续要继续保证编译速度，应坚持这些方向：

- 保持 crate 边界清晰，避免大而全的单体模块
- 控制依赖数量，尤其是编译重的大型依赖
- 优先用小而稳定的抽象，不提前引入复杂宏系统和过度泛型设计
- 把高频变动代码和低频基础模块分层

也就是说，Rust 编译速度首先是架构问题，其次才是工具参数问题。

### 4. 编译性能优化工具链

在架构优化之外，当前项目已经配置了以下编译性能工具链：

#### sccache — 跨项目依赖缓存共享

全局配置（`~/.cargo/config.toml`）：

```toml
[build]
rustc-wrapper = "sccache"
```

- 所有 Rust 项目的依赖编译结果缓存在 `~/.cache/sccache`
- 缓存上限 10 GiB，自动淘汰旧条目
- 首次编译时缓存未命中，后续编译相同依赖时直接命中缓存
- 多项目共享 tokio/serde 等重型依赖时效果显著

#### mold — 高性能链接器

项目级配置（`.cargo/config.toml`）：

```toml
[target.x86_64-unknown-linux-gnu]
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

- Debug 模式链接提速 50%+
- 仅影响链接阶段，不影响编译阶段
- Release 构建同样受益

#### cargo-sweep — 编译缓存清理

- 替代 `cargo clean`，只清理无用缓存，保留最新增量编译产物
- 使用方式：`just sweep`（清理 7 天前的缓存）或 `just sweep-all`（全量清理）

#### cargo-watch — 文件监听自动测试

- 监听源码变化，自动执行增量编译和测试
- 使用方式：`just watch-test`（自动测试）或 `just watch-check`（自动检查）

#### 核心禁忌

日常开发中严禁以下操作：

- ❌ 使用 `cargo clean`（会清空所有编译缓存，导致全量重编译）
- ❌ 使用 `cargo test --release`（Release 编译慢 10~100 倍，且生成独立缓存）
- ❌ 随意修改 `Cargo.toml` 依赖（会触发依赖全量重编译）
- ❌ 删除 `target/` 目录（Rust 增量编译的核心）

如需清理缓存，使用 `cargo sweep --time 7` 或 `just sweep`。

### 4. 如何保证 Rust 代码保持最新

这里的“最新”至少有三层含义：

- 与当前迁移目标保持一致
- 与当前文档和约束保持一致
- 与当前依赖和工具链策略保持一致

当前项目已经做的准备包括：

- 通过 `rust-toolchain.toml` 固定当前 Rust toolchain 口径
- 通过 workspace 配置固定 edition、`rust-version` 和 lint 基线
- 通过开发日志记录每一阶段的真实状态，减少“文档还是旧的、代码已经变了”的情况
- 通过迁移文档明确当前保留、删除、延后和重设计的边界，避免代码朝旧系统无限回归

后续要继续保证代码“最新”，重点不是盲目追新，而是保持下面三种同步：

- 代码与迁移目标同步
- 代码与文档同步
- 代码与工具链和质量门禁同步

如果未来更新依赖、切换 Rust 版本或调整迁移策略，也应该被视为一次明确的项目变更，而不是悄悄发生的背景变化。

### 命令入口准备

本阶段还补充了 `justfile`，把常用质量门禁和后续扩展命令收敛到统一入口。当前内容如下：

```just
set shell := ["bash", "-uc"]

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

test:
    cargo nextest run --workspace --all-features
    cargo test --doc --workspace

check:
    cargo check --workspace --all-targets

doc:
    cargo doc --workspace --no-deps

doc-open:
    cargo doc --workspace --no-deps --open

docs-remind:
    ./scripts/doc_sync_reminder.sh

cov:
    cargo llvm-cov nextest --workspace --all-features

deny:
    cargo deny check

# Clean build artifacts older than N days (default: 7)
sweep days="7":
    cargo sweep --time {{days}}

# Clean all unused build artifacts
sweep-all:
    cargo sweep --all

# Watch for file changes and auto-run tests
watch-test:
    cargo watch -x test

# Watch for file changes and auto-run check
watch-check:
    cargo watch -x check

verify: fmt-check lint test
```

注意：

- `just docs-remind` 会根据当前 Git 变更范围给出文档同步提醒
- `just doc` 只生成当前 workspace 的文档，产物位于 `target/doc`
- `just test` 依赖 `cargo-nextest`
- `just deny` 依赖 `cargo-deny`
- `just cov` 依赖 `cargo-llvm-cov`
- `just sweep` 依赖 `cargo-sweep`
- `just watch-test` / `just watch-check` 依赖 `cargo-watch`

### 文档同步提醒机制

仅写规则很容易被忽略，因此当前仓库补了两个工程入口：

- `.github/pull_request_template.md`
  用于在 PR 层显式要求说明“这次改动是否影响文档”
- `scripts/doc_sync_reminder.sh`
  用于根据当前 Git 变更范围做一次轻量提醒

这套机制的定位不是替代判断，而是把“记得检查文档”从口头要求变成工程上的固定步骤。

当前提醒策略是：

- 如果命中了 workspace 配置、crate 结构、CI、核心源码等变更范围，却没有同步修改 `README.md` 或 `docs/`，脚本会给出提醒
- 如果是跨多个 crate 的变更，会更倾向于判定为中大型变更
- 如果只是局部实现或小 bug，脚本会提醒你确认是否需要更新开发日志，而不是一律要求改正式文档

默认情况下这是提醒，不直接阻断；如果后续需要更严格执行，也可以把它切到严格模式。

## 当前仓库中的关键初始化文件

- [Cargo.toml](../Cargo.toml)
- [rust-toolchain.toml](../rust-toolchain.toml)
- [justfile](../justfile)
- [ci.yml](../.github/workflows/ci.yml)
- [migration-strategy.md](./migration-strategy.md)
- [architecture.md](./architecture.md)

### CI 中当前实际执行的检查

当前 `.github/workflows/ci.yml` 中已经接入了以下流程：

1. 安装 stable Rust
2. 安装 `rustfmt` 和 `clippy`
3. 安装 `cargo-nextest`
4. 执行 `cargo fmt --all --check`
5. 执行 `cargo clippy --workspace --all-targets --all-features -- -D warnings`
6. 执行 `cargo nextest run --workspace --all-features`
7. 执行 `cargo test --doc --workspace`

这说明，本阶段不仅准备了本地开发约束，也同步准备了 CI 侧的检查基线。

## 本阶段记录结论

从记录角度看，这一阶段已经完成了以下环境准备工作：

- 固定了 Rust stable toolchain 与核心组件
- 明确了扩展工具范围，并补充了统一 bootstrap 脚本
- 固定了编辑器侧 `rust-analyzer` 配置
- 固定了 workspace 结构、Rust 版本与 lint 约束
- 固定了 `justfile` 作为统一命令入口
- 固定了 CI 的检查基线
- 把 AI 协作所依赖的 skills、上下文管理和代码索引方式写入文档

但从实际机器状态看，仍有部分工具准备未完全落地，这部分属于后续需要继续补齐的工作。

## License 说明

本项目当前采用 `GPL-3.0-only`。

这是一个从 TrendRadar 迁移而来的 Rust 重构项目，因此在项目边界、文档、代码组织和后续实现上，都应明确其迁移来源，并保持与源项目许可策略一致，除非未来有明确的新结论。

## 当前阶段结论

当前环境准备阶段已经具备以下条件：

- Rust workspace 已建立
- 基础 crate 骨架已建立
- `fmt` / `clippy` / `test` 门禁已接入
- CI 已建立
- 迁移与架构文档已建立

但仍有后续工作需要补齐：

- 机器级安装 `cargo-nextest`、`cargo-deny`、`cargo-llvm-cov`
- 增加 fixture 和更强的测试基线
- 开始核心模块的真实迁移实现
