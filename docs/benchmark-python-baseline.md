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
- Python 侧已在本仓库内固化测量脚本：`scripts/benchmark_python_baseline.py`
- `fixture_pipeline_minimal` 已补一条同机真实对比值
- 其余阶段拆分 benchmark 仍保持 `pending`

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

本次与 Python 对齐时，实际使用的同环境 Rust 主对比值如下（2026-04-14 本机重跑）：

| Benchmark | Rust 当前锚点 |
| --- | --- |
| `pipeline_total/fixture_pipeline_minimal` | `162.33 µs ~ 165.75 µs` |
| `pipeline_stage/fetch_fixture_sources` | `9.0537 µs ~ 9.4556 µs` |
| `pipeline_stage/analyze_filter_rank_group` | `1.1192 µs ~ 1.1954 µs` |
| `pipeline_stage/storage_in_memory_roundtrip` | `75.258 µs ~ 84.820 µs` |
| `pipeline_stage/report_render_all_formats` | `30.268 µs ~ 31.643 µs` |

### Python 侧

Python 真实入口已经固定为：

- 代码入口：`/home/yuanzhi/Develop/ai-research/TrendRadar/trendradar/__main__.py`
- CLI 入口：`python -m trendradar`

但 Python CLI 当前没有暴露可配置的热榜 API 基址，因此本仓库内新增了一个 benchmark bridge：

- 脚本：`scripts/benchmark_python_baseline.py`
- 实际测量调用：`trendradar.__main__.NewsAnalyzer.run()`
- 额外注入：本地 fixture HTTP server + `DataFetcher.DEFAULT_API_URL` monkeypatch

这样做的目的，是在**不修改旁边 Python 仓库代码**的前提下，仍然复用其真实主流程对象，并保持“无 HTTP 噪声、只对齐 fixture 语义”的约束。

Python 侧当前主对比命令：

```bash
/home/yuanzhi/Develop/ai-research/TrendRadar/.venv/bin/python \
  scripts/benchmark_python_baseline.py --warmups 5 --runs 10
```

脚本会把最近一次结果写到：

```bash
target/python-benchmark-baseline/fixture_pipeline_minimal.json
```

当前测量仍遵循这些原则：

- 优先测端到端 fixture pipeline，而不是 HTTP 路径
- 使用独立的 benchmark 配置，避免把生产配置噪声带进基线
- 至少记录 warm-up 后的稳定区间，不只记单次结果
- 建议使用同一台机器、同一时间段完成 Rust / Python 两组测量

## 本次环境记录

本次 `fixture_pipeline_minimal` 实测环境如下：

| 字段 | 值 |
| --- | --- |
| 日期 | `2026-04-14 10:35:23 +08:00` |
| 机器 | `yuanzhi-Legion-Y7000P-IRX9` |
| CPU | `Intel(R) Core(TM) i7-14700HX` |
| OS | `Linux-6.14.0-37-generic-x86_64-with-glibc2.39` |
| Rust 版本 | `rustc 1.94.1 (2026-03-25)` |
| Python 版本 | `3.13.9` |
| 测量工具 | `Criterion + time.perf_counter_ns` |
| 备注 | `Python 固定 CLI 入口为 python -m trendradar；实际 benchmark 通过 bridge 复用 NewsAnalyzer.run，并把热榜/RSS 重定向到本地 fixture HTTP server` |

没有环境记录的对比值，不应写入 README 主表。

## 对比结果

当前主对比结果如下：

| Profile | Rust 锚点 | Python 基线 | 状态 | 备注 |
| --- | --- | --- | --- | --- |
| `fixture_pipeline_minimal` | `162.33 µs ~ 165.75 µs` | `73.95 ms ~ 89.90 ms` | `done` | E6 主对比入口；Python 中位 `81.22 ms`，Rust 中位 `163.96 µs` |
| `fetch_fixture_sources` | `9.0537 µs ~ 9.4556 µs` | `待测` | `pending` | 仅在 Python 有可对齐拆分入口时填写 |
| `analyze_filter_rank_group` | `1.1192 µs ~ 1.1954 µs` | `待测` | `pending` | 同上 |
| `storage_in_memory_roundtrip` | `75.258 µs ~ 84.820 µs` | `待测` | `pending` | 仅在存储语义完全一致时填写 |
| `report_render_all_formats` | `30.268 µs ~ 31.643 µs` | `待测` | `pending` | 需要输出集合完全一致 |

### 粗略结论

- 仅就本次同机 `fixture_pipeline_minimal` 主对比看，**Rust 明显快于 Python**
- 以中位数粗略估算：`81.22 ms / 163.96 µs ≈ 495x`
- 这个倍数是基于当前本机、当前 bridge 测量方式的**推算值**，不应外推为所有输入、所有环境、所有配置下的固定结论

## 何时可以把 E6 视为完成

至少满足下面 4 条：

1. 已固定 1 条 Python 侧真实测量入口
2. `fixture_pipeline_minimal` 已有 Rust / Python 同环境对比值
3. README 中出现的 Python 对比值都能回链到本文件
4. 文档明确说明哪些值是主基线，哪些只是阶段性记录

## 当前下一步

如果继续推进 E6，建议按下面顺序执行：

1. 保持 `fixture_pipeline_minimal` 作为主对比入口，不轻易扩散范围
2. 视需要决定是否补 `fetch / analyze / storage / report` 阶段拆分对比
3. 如果要把 Python 对比值抬到 README 主表，先确认展示口径只引用本文件
4. 若后续 Python 仓库自身暴露了可配置 hotlist API 入口，可再把 bridge 简化为更贴近纯 CLI 的测量方式
