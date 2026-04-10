# Contributing

感谢关注 TrendRadar Rust。

当前仓库主要用于个人学习 Rust 与迁移实践，但欢迎围绕工程结构、验证闭环、Git 规范和迁移策略进行交流与改进。

## 当前贡献范围

在明确进入功能迁移阶段之前，优先接受下面这些类型的改动：

- 文档修正与结构优化
- 环境脚本与验证入口完善
- Git 工作流和提交规范收口
- 系统性测试模板和 fixture 规范补充

## 提交前要求

提交前请至少完成：

```bash
just env-check
just verify-basic
```

并确保：

- 分支命名符合 `docs/git-workflow.md`
- 提交标题符合 `<type>(<scope>): <summary>`
- 结构性变更同步更新 `README.md` 或 `docs/`

## 交流方式

- 提交 Issue
- 发起 Pull Request
- 针对迁移策略、验证规则和 Rust 工程结构发起讨论
