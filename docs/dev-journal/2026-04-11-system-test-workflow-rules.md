# 开发记录：系统测试流程规则与工作区 harness 落地

## 基本信息

- 日期：2026-04-11
- 阶段：Wave 1 收口后的测试基线加强
- 主题：把“先补测试再实现”的流程落成仓库规则
- 目标：避免测试继续停留在收尾补写，而是变成默认工作流

## 本次完成内容

- 在根 `Cargo.toml` 中补了工作区级测试 harness 入口
- 新增 `src/lib.rs` 作为根工作区测试承载点
- 新增 `tests/common/mod.rs` 作为共享 fixture loader
- 新增 `tests/system.rs` 挂载系统性测试模块
- 新增 3 条工作区系统测试：
  - `tests/system/fetch_to_domain.rs`
  - `tests/system/analyze_pipeline.rs`
  - `tests/system/storage_to_report.rs`
- 接入 `insta`，用于稳定 JSON 输出的 inline snapshot
- 更新下面这些文档，把测试前置写成仓库规则：
  - `docs/system-test-template.md`
  - `docs/acceptance-matrix.md`
  - `tests/README.md`
  - `tests/system/README.md`
  - `fixtures/README.md`
  - `fixtures/system/README.md`
- 将 autoresearch 工件加入 `.gitignore`

## 为什么这次要先改流程

Wave 1 虽然已经补了一批 crate 级测试，但暴露了一个很典型的问题：

- 大多数测试还是跟着实现走
- 系统性测试模板已经存在，但默认入口还没有真正跑起来
- fixture、测试入口、验收矩阵三者之间，仍然缺少“实现前绑定”的约束

这会带来两个后果：

1. 测试容易在“功能做完后再补”，失去对设计和边界的约束力
2. 系统测试目录虽然存在，但不一定会被团队自然使用

所以这次没有继续补功能，而是先把测试流程从“推荐做法”推进成“仓库规则”。

## 这次具体解决了什么

### 1. 让根目录系统测试真正可执行

之前 `tests/system/` 只是目录约定和文档入口。

这次通过：

- 根工作区 package
- `tests/system.rs`
- `tests/common/mod.rs`

把它变成了会被 `cargo test --workspace` 真正执行的测试挂载点。

### 2. 把 fixture loader 统一起来

之前各个测试直接各自 `read_to_string`，写法分散，也不利于后续统一处理 fixture 路径、解析错误和固定时间。

这次把读取逻辑收到了 `tests/common/mod.rs`，后续新增系统测试时默认复用它。

### 3. 把 snapshot 机制最小接入

当前仓库最需要 snapshot 的不是所有模块，而是：

- JSON 顶层输出
- 系统链路的稳定结果

所以这次只引入了 `insta`，并先在 `storage -> report` 链路上使用 inline snapshot，保持引入成本可控。

### 4. 把“测试前置”写进规则文件

这次最重要的不是多了几条测试，而是多了明确流程：

1. 先写契约
2. 再写验收矩阵
3. 再补 fixture
4. 再写失败测试
5. 最后才进入实现

这样后续无论是人工还是 AI，都不再把测试视为收尾动作。

## 本次结果

- `tests/system/*.rs` 已经成为真实可执行入口
- 工作区系统测试现在覆盖了：
  - `config -> fetch -> domain`
  - `config -> schedule -> analyze`
  - `storage -> report`
- `cargo test --workspace` 通过
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` 通过
- `.gitignore` 已忽略 autoresearch 运行工件，工作树更干净

## 还没解决的部分

- 当前系统测试还是偏最小样例，覆盖的是“能跑通”，不是“边界全覆盖”
- `fetch` 还没有真正配置驱动的 adapter 构造测试
- `storage` 还缺失败路径测试
- `report` 当前只接入了 inline snapshot，还没有独立快照目录治理

这些都属于后续在这套流程下继续自然补齐的内容，而不需要再重建测试框架。

## 学到的点

- 只写测试模板不够，必须让模板有真实挂载点，否则团队默认还是会回到“先写功能”
- fixture 目录、验收矩阵和系统测试入口，三者必须一起设计，单独推进其中一个都容易失效
- 对当前仓库来说，新增一套大测试框架收益不高；真正缺的是更明确的默认流程

## 下一步

- 把新增系统测试入口继续扩到 Wave 2 的最小 `app` pipeline
- 为 `fetch`、`storage`、`report` 补失败路径与更完整快照
- 视输出规模决定是否把 `insta` 从 inline snapshot 扩到 `tests/snapshots/`
