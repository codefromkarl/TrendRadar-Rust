# `config` 契约骨架

## 模块目标

定义应用最小配置结构、默认值、加载入口和校验规则。

## 当前实现基线

- `AppConfig`
- `load_default_config`
- `load_config_from_json_str`
- `validate_config`

## 需要固化的契约

### 1. 顶层配置结构

- 当前顶层字段：
  - `timezone: String`
  - `platforms: Vec<String>`
- 必填字段：
  - `timezone`
- 默认值字段：
  - `timezone` 默认值为 `Asia/Shanghai`
  - `platforms` 默认值为空数组
- 可延后字段：
  - RSS 订阅列表
  - 调度配置
  - 输出目标与输出开关
  - 存储配置

### 2. 调度配置

- 当前状态：
  - 调度字段尚未进入 `AppConfig`
  - Wave 0 只在契约层预留，不在本轮实现中扩字段
- 预留字段方向：
  - 时区相关调度窗口
  - 是否执行抓取 / 分析 / 推送的阶段开关
- 非法配置的错误语义：
  - 一旦调度字段落地，必须继续复用 `TrendRadarError::InvalidConfig`
  - 错误消息需要包含具体字段名和触发条件

### 3. 数据源配置

- 热榜平台列表：
  - `platforms` 表示热榜来源标识列表
  - 当前允许空数组，空数组不视为非法配置
- RSS 订阅列表：
  - 当前尚未进入实现
  - Wave 0 只在契约中确认它属于 `config` 负责范围
- 输出开关与输出目标：
  - 当前尚未进入实现
  - 在进入 `report` 与 `app` 集成前，不阻塞 `config` 最小闭环

## 错误契约

- 解析错误：
  - JSON 解析失败时返回 `TrendRadarError::InvalidConfig`
  - 错误消息前缀固定为 `failed to parse config json:`
- 校验错误：
  - `timezone` 为空时返回 `TrendRadarError::InvalidConfig`
  - 当前错误消息为 `timezone must not be empty`
- 错误消息要求：
  - 首版至少要包含字段名或解析失败原因
  - 不要求与旧系统错误文本完全一致

## 兼容要求

- 当前不兼容旧配置文件结构：
  - Rust 首版不追求完整兼容旧配置
  - 只保留主链路所需的最小配置子集
- 迁移策略：
  - 先在契约文档中显式声明“保留 / 延后”的字段范围
  - 后续如需兼容旧配置，应通过明确的映射规则扩展，而不是在 `AppConfig` 中一次性堆入所有历史字段

## 验证方式

- fixture：
  - [fixtures/system/config/minimal-valid.json](../../fixtures/system/config/minimal-valid.json)
  - [fixtures/system/config/invalid-empty-timezone.json](../../fixtures/system/config/invalid-empty-timezone.json)
- 测试：
  - `cargo test -p trendradar-config`
  - `cargo test -p trendradar-app --test config_to_bootstrap`
- 快照：
  - 当前不需要独立快照
  - Wave 0 通过 fixture 驱动测试锁定行为

## 待补充决策

- 顶层配置是否拆为模块化子结构体
- 配置加载是否只保留 JSON，还是预留 TOML / YAML
