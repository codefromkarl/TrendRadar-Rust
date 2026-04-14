# Python Comparison Baseline

## 目标

这份文档用于收口 E6 中 Rust / Python 对比基线的最小契约，避免后续直接把未对齐的数字写进 README 或路线图。

当前文档的职责不是宣布“Rust 已经比 Python 快多少”，而是先固定：

- 对比范围
- 输入对齐方式
- 环境记录方式
- 结果落表格式

## 当前状态

- Rust 侧已有可复用 benchmark 入口与基线结果，详见 [benchmark-baseline.md](./benchmark-baseline.md)
- Python 侧当前尚未在本仓库内固化统一测量脚本
- 因此本文件目前提供的是“对比基线模板”，不是“已完成的对比结论”

## 对比边界

E6 的 Python 对比基线，当前只建议覆盖本地、可重复、无网络噪声的路径。

优先对比：

1. 最小 fixture pipeline 端到端耗时
2. 可选的本地阶段性对比，例如存储或报告渲染

当前不建议直接纳入：

- HTTP smoke benchmark
- 真实网络请求
- AI 分析旁路
- MCP 工具服务路径
- 远程对象存储

原因很直接：这些路径要么噪声过大，要么当前 Rust / Python 两边还没有完全对齐的稳定入口。

## 输入对齐要求

Rust 侧当前主 benchmark 使用的输入如下：

- 配置基线：`fixtures/system/config/minimal-valid.json`
- 热榜 fixture：`fixtures/system/fetch/hotlist-weibo.json`
- RSS fixture：`fixtures/system/fetch/rss-rust-blog.json`

Python 对比时，必须保证语义等价，而不是只看“差不多”：

- 同样的时区语义
- 同样的 1 个热榜源 + 1 个 RSS 源
- 同样的 fixture 内容
- 同样的输出目标
- 不引入真实网络请求

如果 Python 侧仍使用 YAML 配置或旧入口，可以做格式转换，但不能改 fixture 语义。

## 推荐测量方式

### Rust 侧

固定入口：

```bash
cargo bench --package trendradar-app --bench pipeline_bench
```

当前可直接引用的 Rust 锚点：

| Benchmark | Rust 当前锚点 |
| --- | --- |
| `pipeline_total/fixture_pipeline_minimal` | `147.17 µs ~ 166.30 µs` |
| `pipeline_stage/fetch_fixture_sources` | `9.0537 µs ~ 9.4556 µs` |
| `pipeline_stage/analyze_filter_rank_group` | `1.1192 µs ~ 1.1954 µs` |
| `pipeline_stage/storage_in_memory_roundtrip` | `75.258 µs ~ 84.820 µs` |
| `pipeline_stage/report_render_all_formats` | `30.268 µs ~ 31.643 µs` |

### Python 侧

Python 侧当前没有仓库内统一脚本，因此建议先固定“测量原则”，再填实际数值：

- 优先测端到端 fixture pipeline，而不是 HTTP 路径
- 使用独立的 benchmark 配置，避免把生产配置噪声带进基线
- 至少记录 warm-up 后的稳定区间，不只记单次结果
- 建议使用同一台机器、同一时间段完成 Rust / Python 两组测量

如果沿用原 Python 主程序入口，可以采用类似下面的测量方式：

```bash
python main.py --config benchmark-config.yaml
```

上面只是入口示意，不代表当前仓库已经内置该脚本或配置文件。真正落值前，需要先把 Python 侧实际测量入口记录到表格中。

## 环境记录模板

每次记录 Python 对比值时，至少补下面这些字段：

| 字段 | 值 |
| --- | --- |
| 日期 | `待填写` |
| 机器 | `待填写` |
| CPU | `待填写` |
| OS | `待填写` |
| Rust 版本 | `待填写` |
| Python 版本 | `待填写` |
| 测量工具 | `待填写` |
| 备注 | `待填写` |

没有环境记录的对比值，不应写入 README 主表。

## 对比结果模板

建议把 Rust 锚点与 Python 对比值并列表达，但保持“已测 / 未测”状态清晰可见。

| Profile | Rust 锚点 | Python 基线 | 状态 | 备注 |
| --- | --- | --- | --- | --- |
| `fixture_pipeline_minimal` | `147.17 µs ~ 166.30 µs` | `待测` | `pending` | E6 主对比入口 |
| `fetch_fixture_sources` | `9.0537 µs ~ 9.4556 µs` | `待测` | `pending` | 仅在 Python 有可对齐拆分入口时填写 |
| `analyze_filter_rank_group` | `1.1192 µs ~ 1.1954 µs` | `待测` | `pending` | 同上 |
| `storage_in_memory_roundtrip` | `75.258 µs ~ 84.820 µs` | `待测` | `pending` | 仅在存储语义完全一致时填写 |
| `report_render_all_formats` | `30.268 µs ~ 31.643 µs` | `待测` | `pending` | 需要输出集合完全一致 |

## 何时可以把 E6 视为完成

至少满足下面 4 条：

1. 已固定 1 条 Python 侧真实测量入口
2. `fixture_pipeline_minimal` 已有 Rust / Python 同环境对比值
3. README 中出现的 Python 对比值都能回链到本文件
4. 文档明确说明哪些值是主基线，哪些只是阶段性记录

## 当前下一步

如果继续推进 E6，建议按下面顺序执行：

1. 先在本文件填写 Python 实际测量入口
2. 只落 `fixture_pipeline_minimal` 一条主对比值
3. 再决定是否补阶段拆分对比
4. 最后把可公开展示的对比摘要同步到 README 和路线图
