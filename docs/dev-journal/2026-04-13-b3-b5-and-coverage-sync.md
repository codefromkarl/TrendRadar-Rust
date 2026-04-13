# 开发记录：B3 收尾、B4 退出码、B5 安装入口与 O5 测试增强

## 基本信息

- 日期：2026-04-13
- 阶段：v1.2 工程收口
- 主题：热榜平台扩展闭环、CLI 退出码规范、安装分发入口、文档状态校准、HTTP 集成覆盖增强
- 目标：把前面已经具备实现基础的能力补齐到“代码 + 测试 + 文档 +验证”一致的状态

## 背景

此前仓库已经具备多通知渠道、调度增强、热榜 parser 架构和 release workflow，但存在几个明显缺口：

- B3 新平台 parser 已有扩展方向，但缺少完整闭环
- CLI 退出码仍只有统一 `1`
- release 存在，但新用户缺少最小安装入口
- 活文档对通知、调度、工具链版本和当前状态存在口径漂移
- HTTP 生产路径下“部分成功、部分失败”的回归保护还不够强

这些问题不需要大改架构，但会直接影响仓库的可交付性和可维护性。

## 本次完成内容

- 完成 B3 第一轮：新增 `toutiao` / `baidu` parser 与单测
- 完成 B3 第二轮：新增 `pengpai` / `cls` parser 与单测，并兼容 `thepaper` / `cls-hot` 路由
- 为 B3 补齐 app 级 HTTP 配置 fixture 和多平台 `source_type` 路由测试
- 完成 B4：在 CLI 入口新增集中式退出码分类，补配置错误、存储错误 smoke test 和分类单测
- 完成一轮活文档校准：同步 roadmap、README、migration guide、acceptance matrix、module map、toolchain 版本口径
- 完成 B5：新增根目录 `install.sh`、README 安装章节、release 安装说明，并同步 `scripts/bootstrap.sh`
- 作为 O5 第一轮补充，新增 2 条 HTTP resilient 集成测试，覆盖混合成功/失败恢复与新平台 payload 跳过场景

## 关键决策

### 决策 1

- 决策内容：B3 按两轮提交推进，每轮只覆盖 2 个平台
- 原因：和执行文档建议一致，便于审查，也能把 parser 风险隔离在小范围内
- 备选方案：一次性补 4 个平台
- 为什么没有选备选方案：单次 diff 过大，调试和回滚成本更高

### 决策 2

- 决策内容：B4 的退出码逻辑集中放在 `crates/app/src/main.rs`
- 原因：不把 exit code 语义散到各 crate，保持 `app` 作为 CLI 边界
- 备选方案：在各 crate 定义更细的退出码枚举
- 为什么没有选备选方案：会把 CLI 语义反向渗透到库层，边界不干净

### 决策 3

- 决策内容：B5 先交付最小 `install.sh` + README 安装说明，不等 Homebrew
- 原因：可以最短路径补齐新用户安装闭环
- 备选方案：等待 Homebrew formula 一起完成
- 为什么没有选备选方案：会把一个可独立交付的小任务拖成多平台分发设计任务

### 决策 4

- 决策内容：O5 先补 HTTP resilient 集成测试，而不是继续扩 benchmark
- 原因：E6 已有 benchmark 入口和基线，当前更缺的是高风险生产路径的回归保护
- 备选方案：继续做 Python 对比 benchmark
- 为什么没有选备选方案：短期收益不如先补真实错误恢复场景

## 遇到的问题

### 问题 1

- 现象：`newsnow.busiyi.world` 真实接口直接访问会遇到 Cloudflare 挑战
- 原因判断：上游聚合接口对自动化访问有限制
- 处理方式：改为读取 `ourongxing/newsnow` 上游源码确认真实字段形状
- 最终结果：基于源码而不是不稳定线上响应完成 `toutiao` / `baidu` / `pengpai` / `cls` parser

### 问题 2

- 现象：B5 验证时 `./install.sh --help` 首次执行返回权限错误
- 原因判断：新建脚本文件后没有可执行位
- 处理方式：补上 `chmod +x install.sh`
- 最终结果：帮助输出验证通过，提交时也保留了可执行权限

### 问题 3

- 现象：活文档里同时出现 Rust `1.85.x` 与 `1.94.x`
- 原因判断：早期环境文档、README 和参考策略没有跟着 toolchain 升级同步
- 处理方式：只校准活文档，不修改历史开发日志
- 最终结果：用户面文档已统一到 `1.94.1`

## 关键文件 / 关键操作记录

### 关键文件

- `crates/fetch/src/lib.rs`
- `crates/app/src/main.rs`
- `crates/app/tests/wave4_http_pipeline.rs`
- `crates/app/tests/binary_smoke.rs`
- `install.sh`
- `README.md`
- `docs/roadmap.md`
- `docs/extension-execution-plan.md`
- `docs/acceptance-matrix.md`

### 关键提交

- `5191ec3` `migration(fetch): add toutiao and baidu hotlist parsers`
- `546ec50` `migration(fetch): add pengpai and cls hotlist parsers`
- `39814ba` `test(app): close b3 with multi-platform http routing`
- `cb257ce` `migration(app): add stable cli exit codes`
- `ce3f7af` `docs(status): calibrate roadmap and active docs`
- `5da97f5` `docs(release): add install script and install guide`
- `c581af9` `test(app): extend http resilient integration coverage`

## 验证记录

本轮实际执行过的关键验证包括：

- `cargo fmt --all`
- `cargo test -p trendradar-fetch`
- `cargo test -p trendradar-app --test wave4_http_pipeline`
- `cargo test -p trendradar-app --test binary_smoke`
- `cargo test -p trendradar-app`
- `cargo test --workspace`
- `cargo check --workspace --all-targets`
- `bash -n ./install.sh`
- `./install.sh --help`
- `./scripts/check_environment.sh`

## 阶段结论

今天把几个原本分散的“半完成”能力收成了可交付闭环：

- B3 已从 parser 扩展完成到 app 路由和配置样例闭环
- B4 已具备稳定退出码语义
- B5 已形成最小安装与分发入口
- 活文档状态已基本和代码实现对齐
- O5 已开始补生产式 HTTP 恢复场景的集成覆盖

当前工作树只剩本地未跟踪文件 `config-ai-scan.json` 与 `trendradar.db`，未纳入本轮提交。

## 下一步

- 继续推进 O5，优先补并发抓取、多失败源和更大输入规模下的系统级回归保护
- 视需要决定是否把 E6 从“已有 benchmark 基线”进一步推进到 Python 对比或文档化展示
- 后续再看是否进入 `O6 CONTRIBUTING` 或 `C1` 之前的接口整理
