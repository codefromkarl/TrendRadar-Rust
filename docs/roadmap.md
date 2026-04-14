# TrendRadar Rust 路线图

> 首版闭合后（Wave 6），记录后续任务方向与优先级。
> 最后更新：Wave 8 / B4 完成后

## 当前状态

- v1.2 功能补齐已进入文档校准阶段
- 221 tests passed, 0 clippy issues
- Release binary: 9.6MB
- CLI: `--config/--db/--output/--verbose/--dry-run/--help/--version`
- 输出格式: `json / html / both / table / markdown`
- 热榜解析: `generic / weibo / zhihu / bilibili / toutiao / baidu / pengpai / cls`
- CI/CD: fmt + clippy + nextest + release binary (Linux/macOS/Windows)

---

## 待办清单

### 🔴 高优先级（建议立即或在 v1.1 中完成）

#### O1: CI/CD Pipeline

- **目标**: GitHub Actions 自动化测试与发布
- **内容**: `cargo fmt --check` + `cargo clippy` + `cargo test --workspace` + release binary 构建
- **产出**: `.github/workflows/ci.yml`
- **状态**: ✅ 已完成（Wave 7）
- **备注**: ci.yml 补齐 release build 步骤 + 新建 release.yml 三平台自动发布

#### E1: 多平台热榜适配

- **目标**: 支持微博/知乎/B站等差异化 JSON 格式
- **当前**: `HotlistParser` trait + 8 实现（Generic/Weibo/Zhihu/Bilibili/Toutiao/Baidu/Pengpai/Cls）+ 工厂函数
- **产出**: 各平台解析器 + `source_type` 配置字段
- **状态**: ✅ 已完成（Wave 7）
- **依赖**: 无

---

### 🟡 中优先级（v1.1 ~ v1.2 迭代）

#### E2: 终端彩色表格输出

- **目标**: `--output table` 彩色终端友好输出
- **依赖**: `comfy-table` crate（7.1.1，兼容当前 toolchain）
- **状态**: ✅ 已完成（Wave 7）

#### E3: Markdown 输出格式

- **目标**: `--output markdown` 生成 Markdown 表格
- **依赖**: 无
- **状态**: ✅ 已完成（Wave 7）

#### E4: 更多通知渠道

- **目标**: 飞书/钉钉/企业微信通知适配
- **当前**: Webhook + Console + 飞书 + 钉钉 + 企业微信
- **架构**: `Notifier` trait 已就位，新增实现即可
- **状态**: ✅ 已完成

#### O2: 跨平台构建

- **目标**: Linux/macOS/Windows release binary
- **状态**: ✅ 已完成（Wave 7，合并到 release.yml）

#### O3: 安装脚本

- **目标**: `install.sh` / Homebrew formula / `cargo install`
- **状态**: ✅ 已完成（最小 install.sh + README 安装说明，Homebrew 后续按需补）

#### O4: 错误码规范

- **目标**: 统一 exit code（0 成功 / 1 配置错误 / 2 网络错误 / 3 存储错误 / 4 未知错误）
- **状态**: ✅ 已完成

---

### 🟢 低优先级（v1.2+ 或按需）

#### E5: 工作日调度/冷却周期

- **目标**: schedule 支持工作日/周末区分、冷却时间间隔
- **当前**: 已支持时间窗口 + weekday/weekend 覆盖 + cooldown_minutes
- **状态**: ✅ 已完成

#### E6: 性能 Benchmark

- **目标**: 对比 Python 版本，量化 Rust 性能优势
- **依赖**: `criterion` crate
- **当前**: 已有 `cargo bench --package trendradar-app --bench pipeline_bench` 入口、README 基线表、`docs/benchmark-baseline.md`，并新增 `scripts/benchmark_python_baseline.py`；`fixture_pipeline_minimal` 已补同机真实 Rust/Python 主对比值，详见 `docs/benchmark-python-baseline.md`
- **状态**: ✅ 已完成

#### O5: 集成测试覆盖增强

- **目标**: 更多边界场景（大数据量、并发、网络异常恢复）
- **当前**: 已补 HTTP mixed success/failure、并发多失败源 retained results、大输入稳定性、慢源/多失败源恢复，以及新增“复杂并发慢源/失败源组合下连续多轮输出完全一致”的根级系统测试；按当前收口口径，O5 不再继续横向扩测试组合
- **状态**: ✅ 已完成

#### O6: 贡献者文档

- **目标**: CONTRIBUTING.md、开发环境搭建、PR 流程
- **当前**: `CONTRIBUTING.md` 已补齐贡献范围、提交流程、文档同步规则、PR 建议流程和自查清单
- **状态**: ✅ 已完成

---

### Phase 5: 生态扩展（v2.0+，内核稳定后按需启动）

#### P1: MCP Server

- **目标**: 基于 Rust 内核构建 MCP tool 接口
- **前置**: 内核 API 稳定
- **当前**: 已落最小查询型工具服务入口 `trendradar-mcp`，提供 `tools/list` 与查询类 `tools/call`
- **状态**: ✅ 最小版本已完成

#### P2: AI 分析接入

- **目标**: LLM 驱动的新闻摘要/分析
- **前置**: P1 或独立 HTTP 服务
- **当前**: 已落独立 `trendradar-ai` crate、`mock` provider、配置字段和 app 旁路集成；真实远程 provider 仍待补
- **状态**: ✅ 最小版本已完成

#### P3: AI 翻译

- **目标**: 多语言翻译能力
- **前置**: P2
- **状态**: ⬜ 待评估

#### P4: 远程对象存储

- **目标**: S3/OSS adapter
- **架构**: `NewsRepository` trait 已就位
- **当前**: 已落 `storage.backend = "s3" + provider = "mock-s3"` 的 file-backed object store prototype，真实云 provider 仍待补
- **状态**: ✅ 最小版本已完成

#### P5: 可扩展通知 Sink

- **目标**: 统一 Sink trait + 更多渠道（Telegram/Discord/Slack）
- **架构**: `Notifier` trait 已就位
- **状态**: ⬜ 待办

---

## 版本规划

```
v1.0.0 — Release Candidate
└── 首版产品边界闭合，生产就绪

v1.1.0 — 首版增强（已完成 Wave 7）
├── ✅ O1: CI/CD pipeline + 三平台 release
├── ✅ E1: 多平台热榜适配（weibo/zhihu/bilibili/toutiao/baidu/pengpai/cls）
├── ✅ E2: 终端彩色表格输出
├── ✅ E3: Markdown 输出
└── ✅ O2: 跨平台构建（合并到 release.yml）

v1.2.0 — 通知与调度扩展
├── ✅ E4: 飞书/钉钉/企业微信通知
├── ✅ E5: 工作日调度/冷却周期
├── ✅ O4: 错误码规范
├── ✅ O3: 安装脚本
└── E6: 性能 benchmark

v2.0.0 — 生态扩展（Phase 5）
├── ✅ P4: 远程对象存储（最小版本）
├── P5: 可扩展通知 Sink
├── ✅ P1: MCP Server
├── ✅ P2: AI 分析（最小版本）
└── P3: AI 翻译
```

## 决策原则

1. **用户反馈驱动** — Phase 5 不预设需求，先验证 v1.0 满足核心场景
2. **架构已就位** — `Notifier`/`NewsRepository`/`Fetcher` trait 可直接扩展新实现
3. **O1（CI/CD）不阻塞版本** — 可在任何时间点补入
4. **E1（多平台）是替代 Python 的关键** — 决定 Rust 版本实际使用价值
