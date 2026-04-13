# Contributing

感谢关注 TrendRadar Rust。

当前仓库已经具备稳定的 Rust workspace、验证入口、系统测试骨架和迁移基线文档。这个文件的目标是让协作者可以快速判断：

- 现在适合改什么
- 改动前后应该看哪些文档
- 提交前至少要完成哪些验证
- 怎样避免把无关改动混到一起

## 开始之前先看

建议先阅读：

- `README.md`
- `AGENTS.md`
- `docs/git-workflow.md`
- `docs/acceptance-matrix.md`
- `docs/extension-execution-plan.md`

如果你的改动涉及系统性测试、fixture 或阶段性边界，建议再看：

- `tests/README.md`
- `tests/system/README.md`
- `docs/system-test-template.md`

## 当前适合贡献的范围

当前最适合继续推进的方向：

- 文档修正、状态校准和路线图同步
- 环境脚本、安装入口和验证命令优化
- 系统性测试、fixture 和回归保护增强
- 与现有 crate 边界一致的低风险能力补齐

当前不建议直接进入的大范围主题：

- 远程对象存储
- AI 分析 / AI 翻译
- MCP Server
- 大规模跨 crate 架构重写

## 工作方式

### 1. 保持单主题改动

一个分支或一个提交只表达一个清晰意图。

推荐：

- 一个测试批次
- 一个文档同步批次
- 一个 crate 内的一个能力扩展

避免：

- 同时改代码、脚本、README、无关测试
- 顺手清理大量不相关文件

### 2. 先验证边界，再改实现

对于非平凡改动，优先顺序应是：

1. 确认受影响 crate 与文档边界
2. 补 fixture / 测试或确认已有测试入口
3. 再补实现
4. 跑验证命令
5. 同步文档

### 3. 让 `app` 保持薄编排

如果一段逻辑更适合放在 `schedule`、`fetch`、`analyze`、`storage` 或 `report`，就不要把它塞进 `app`。

## 提交前最低要求

提交前请至少执行：

```bash
just env-check
just verify-basic
```

如改动涉及更深路径，建议额外执行对应入口：

```bash
cargo test -p trendradar-app
cargo test --workspace
cargo check --workspace --all-targets
```

## 文档同步规则

出现下面情况时，应该同步更新文档：

- CLI 参数变化
- 验证命令变化
- crate 边界变化
- 新增或删除 fixture / 系统测试入口
- roadmap / execution plan 状态变化

常见同步目标：

- `README.md`
- `docs/roadmap.md`
- `docs/extension-execution-plan.md`
- `docs/acceptance-matrix.md`
- `docs/dev-journal/`

## 分支与提交规范

分支命名和提交规范以 `docs/git-workflow.md` 为准。

提交标题格式：

```text
<type>(<scope>): <summary>
```

例如：

- `test(app): extend http resilient integration coverage`
- `docs(status): calibrate roadmap and active docs`
- `migration(fetch): add toutiao and baidu hotlist parsers`

## 测试与 fixture 约定

- crate 内测试优先放在各 crate 自己的 `src/` 或 `tests/`
- 根级 `tests/system/` 只放跨 crate 或系统层测试
- fixture 优先放在 `fixtures/system/`
- 输出顺序、去重规则、错误路径都优先做结构化断言，少依赖人工比对

## 编译与调试建议

当前仓库已配置编译性能优化工具链，详见 `docs/environment-setup.md`。

推荐用法：

- `just sweep`：清理旧缓存
- `just watch-test`：自动监听并运行测试
- `cargo test`：日常 Debug 测试

不建议默认做法：

- 习惯性使用 `cargo clean`
- 为了“更真实”而默认用 `cargo test --release`

## 开发日志建议

当一次工作满足下面任一条件时，建议补一篇 `docs/dev-journal/`：

- 形成阶段性结论
- 完成一个成组任务
- 做了重要取舍
- 需要给后续开发留明确提醒

## 交流方式

- 提交 Issue
- 发起 Pull Request
- 针对迁移策略、验证规则和 Rust 工程结构发起讨论
