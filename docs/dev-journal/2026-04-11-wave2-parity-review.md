# Wave 2 parity / 边界审查

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 2 收口
- 主题：`app` 最小 fixture pipeline 的 parity / 边界审查
- 目标：确认 `app` 仍是薄编排，并给出进入 Wave 3 的结论

## 审查范围

- `crates/app/src/lib.rs`
- `crates/app/tests/wave2_pipeline.rs`
- `docs/implementation/app.md`
- `docs/acceptance-matrix.md`
- `docs/module-map.md`
- `docs/parallel-migration-plan.md`

## 事实证据

- `run_fixture_pipeline` 先做配置校验，再调用 `schedule`、`fetch`、`analyze`、`storage`、`report` 的公开接口串起最小闭环
- `app` 当前没有自行实现排序规则、来源聚合规则、抓取解析、去重策略或 JSON 渲染
- `crates/app/tests/wave2_pipeline.rs` 已固定 fixture、固定时间，并对输出条目数、排序结果、来源聚合和 JSON 结构做断言
- `cargo test -p trendradar-app` 已通过，可作为本轮最小机械验证

## 审查判断

### parity 结论

- 当前实现与 `docs/parallel-migration-plan.md` 的 Wave 2 目标一致
- 仓库已经具备从配置到结构化输出的最小系统性 fixture pipeline
- 文档、fixture、测试入口和 crate 接口之间没有发现明显冲突

### 边界结论

- `app` 仍然是薄编排层
- 业务规则仍留在对应上游 crate：
  `schedule` 负责阶段决策，`fetch` 负责 fixture 解析，`analyze` 负责排序与来源聚合，`storage` 负责持久化与读取，`report` 负责 JSON 输出
- 本轮没有发现把业务规则偷偷吸进 `app` 的迹象

## 风险与提醒

- Wave 3 扩展系统 fixture 时，`app` 只应增加挂载、断言和错误穿透，不应新增规则判断
- 如果后续需要更多系统样例，应优先扩 fixture 和上游 crate 能力，不要把临时兼容逻辑塞进 `app`

## 阶段结论

`W2-parity-review` 通过。当前仓库可把 `app` 视为“已具备最小 fixture pipeline 和系统测试入口”的状态，并以前景方式推进 Wave 3。

## 下一步

- 扩展 `schedule`、`analyze` 的 richer cases 与 fixture
- 为 `fetch`、`storage`、`report` 增加更明确的系统级断言
- 保持 `app` 仅作为编排和系统测试挂载点
