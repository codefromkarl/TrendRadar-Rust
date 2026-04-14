# Benchmark Baseline

## 目标

这份文档用于收口 TrendRadar Rust 当前可复用的 benchmark 基线，避免后续优化只停留在“感觉更快”。

当前基线只覆盖 Rust 内部测量结果，不直接声称已经完成与 Python 版本的严格对比。

如需查看 Rust / Python 对比的记录模板与对齐要求，见 [benchmark-python-baseline.md](./benchmark-python-baseline.md)。

## Benchmark 入口

当前统一入口：

```bash
cargo bench --package trendradar-app --bench pipeline_bench
```

对应代码位置：

- `crates/app/benches/pipeline_bench.rs`

## 当前保留的 Rust 基线

### 当前主基线

| Benchmark | 当前基线 |
| --- | --- |
| `pipeline_total/fixture_pipeline_minimal` | `147.17 µs ~ 166.30 µs` |
| `pipeline_stage/fetch_fixture_sources` | `9.0537 µs ~ 9.4556 µs` |
| `pipeline_stage/analyze_filter_rank_group` | `1.1192 µs ~ 1.1954 µs` |
| `pipeline_stage/storage_in_memory_roundtrip` | `75.258 µs ~ 84.820 µs` |
| `pipeline_stage/report_render_all_formats` | `30.268 µs ~ 31.643 µs` |

### 历史初始基线

| Benchmark | 初始基线 |
| --- | --- |
| `pipeline_total/fixture_pipeline_minimal` | `194.20 µs ~ 207.51 µs` |
| `pipeline_stage/storage_in_memory_roundtrip` | `99.968 µs ~ 108.28 µs` |

## 当前可解释的对比

目前仓库内可以直接复用的对比结论主要有：

- `pipeline_total/fixture_pipeline_minimal` 已从 `194.20 µs ~ 207.51 µs` 降到 `147.17 µs ~ 166.30 µs`
- `pipeline_stage/storage_in_memory_roundtrip` 已从 `99.968 µs ~ 108.28 µs` 降到 `75.258 µs ~ 84.820 µs`

这些对比主要反映两类优化收益：

- 批量写入替代逐条写入
- 主链路阶段职责收口后，pipeline 总体开销下降

## 为什么当前还没有 Python 对比值

当前 benchmark 结果是 Rust 内部基线，不是 Rust / Python 的一一对照。

还未直接写入 Python 对比值的原因是：

- Python 侧还没有在当前仓库内固化统一的测量脚本
- 两边的输入、运行方式和环境还没有完全对齐
- 直接写未经对齐的对比值，容易造成误导

## E6-2 后续建议

Python 对比基线的模板、对齐规则和落表方式，已经单独整理到 [benchmark-python-baseline.md](./benchmark-python-baseline.md)。

后续如果继续推进 E6，建议直接在那份文档中补真实测量入口和结果，而不是继续把对比说明堆到本页。
