# 运行稳定性说明

## 目标

这份文档用于把“测试通过”转换成更接近运行层的稳定性证据。

当前不声称已经完成长期线上观测，而是先固定第 1 轮可复查的稳定性依据。

## 当前验证面

### 1. 工作区回归

当前工作区测试通过口径：

- `cargo test --workspace`
- 当前结果：`238 tests passed`

### 2. 系统层恢复能力

当前已覆盖：

- 慢源 / 失败源混合场景
- 多失败源并发抓取
- 保留成功结果并跳过失败源
- 重复执行输出一致性

关键入口：

- [http_resilient_recovery.rs](../tests/system/http_resilient_recovery.rs)
- [large_input_stability.rs](../tests/system/large_input_stability.rs)
- [large_output_consistency.rs](../tests/system/large_output_consistency.rs)

### 3. 重复执行稳定性

当前重点验证的是：

- 同一组 HTTP mock 输入
- 慢源 / 失败源同时存在
- 连续多轮执行
- 输出结构完全一致

这类测试的目标不是压测吞吐，而是确认：

- pipeline 没有随机抖动
- 并发抓取不会破坏结果顺序
- 容错模式下的保留结果语义稳定

## 建议的部署前检查

每次准备部署前，至少执行：

```bash
cargo test --workspace
trendradar --dry-run --config /path/to/config.json
```

若使用 Docker，再补：

```bash
docker build -t trendradar:local .
docker run --rm -v "$(pwd)/deploy/runtime/config.json:/config/config.json:ro" trendradar:local --config /config/config.json --dry-run
```

## 当前结论

第 1 轮当前已经具备下面这类稳定性证据：

- 开发回归稳定
- 复杂失败恢复路径稳定
- 多轮重复执行输出稳定
- 大输入与多格式输出路径稳定

下一阶段若要继续提高可信度，更合适的方向是：

- 增加真实环境定时运行记录
- 增加资源占用观测
- 增加长时段运行报告
