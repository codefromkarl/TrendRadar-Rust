# Git 工作流与提交规范

## 文档目标

这份文档用于约束 TrendRadar Rust 在并行迁移阶段的 Git 分支、提交和审查方式。

目标不是追求复杂流程，而是解决三个实际问题：

- 多个并行任务的提交一眼可辨别
- 不同 agent 或开发者的提交风格一致
- 降低跨任务混改和冲突概率

## 核心原则

### 1. 一个分支只做一个主题

- 一个分支只承载一个环境改动、一个 crate、一个 adapter 或一组契约
- 不在同一分支里混入无关文档整理、脚本重写和功能迁移

### 2. 一个提交只表达一个意图

- 一个提交只做一类改动
- 如果同时改了代码、测试、文档，优先按“同一意图是否不可拆”判断
- 纯格式化清理不要混进功能提交

### 3. 并行优先按边界切分

并行迁移时，优先按下面这些边界拆开：

- crate
- 契约
- fixture
- 环境脚本
- 文档基线

不要按“想到哪里改哪里”的方式拆任务。

## 分支命名规范

默认命名模式：

```text
<track>/<scope>-<topic>
```

推荐 `track`：

- `env`
- `docs`
- `rules`
- `migration`
- `test`
- `refactor`

推荐示例：

- `env/githooks-setup`
- `docs/git-workflow`
- `rules/agents-guardrails`
- `migration/config-loader`
- `migration/analyze-ranking`
- `test/config-fixtures`

## 提交消息规范

默认提交标题格式：

```text
<type>(<scope>): <summary>
```

示例：

- `env(githooks): add commit-msg and pre-push hooks`
- `docs(workflow): define parallel migration commit rules`
- `rules(agents): tighten allowed git operations`
- `test(config): add fixture-driven bootstrap checks`
- `migration(analyze): scaffold ranking pipeline`
- `fix(config): reject empty timezone in loader`

### 允许的 `type`

- `env`
- `docs`
- `rules`
- `chore`
- `test`
- `refactor`
- `migration`
- `fix`

### `scope` 约束

`scope` 应尽量指向真实边界，而不是抽象词。

推荐写法：

- crate 名
- 文档模块名
- 脚本名
- 规则名

例如：

- `config`
- `app`
- `githooks`
- `workflow`
- `agents`

## 提交正文建议

如果提交不是极小改动，建议正文按下面顺序写：

```text
Why:
- 为什么要改

What:
- 改了什么

Verify:
- 跑了什么验证命令
```

## 并行迁移时如何降低冲突

### 1. 先锁边界，再开分支

并行任务开始前，先明确每个分支负责：

- 哪个 crate
- 哪个文档
- 哪个脚本
- 哪类 fixture

### 2. 避免共享写入热点

并行期尽量减少多人同时修改：

- `Cargo.toml`
- `justfile`
- `README.md`
- `docs/environment-setup.md`
- `AGENTS.md`

这些属于高冲突文件，应集中收口。

### 3. 文档与实现分层

如果多条迁移线并行推进：

- 功能分支优先只改自己负责的 crate
- 跨仓库规则与环境调整单独走 `env/*` 或 `rules/*`
- 大范围文档整理单独走 `docs/*`

### 4. 纯格式化单独提交

如果必须做大范围 `fmt` 或命名清理，应单独一个提交，避免污染真实迁移 diff。

## Definition of Done

一个并行迁移分支在准备合并前，至少应满足：

- 分支名称符合规范
- 提交标题符合规范
- 改动范围与分支主题一致
- 至少执行一条相关验证命令
- 如果命中文档影响范围，已同步更新文档

## 仓库内自动约束

当前仓库通过下面方式帮助保持一致：

- `.githooks/pre-commit`
- `.githooks/pre-push`
- `.githooks/commit-msg`
- `.gitmessage`

这些约束的目标是减少风格漂移，不是替代代码审查。
