# `mcp` 实施骨架

## 目标

提供一个与现有 CLI 路径分离的最小查询型工具服务入口。

## 本轮范围

- 新增独立 `trendradar-mcp` crate
- 提供 `tools/list`
- 提供 `tools/call`
- 先实现查询类工具：
  - `storage.list_news`
  - `report.render_json`
  - `ai.analyze`

## 本轮不做

- 完整 MCP 协议兼容层
- 写操作工具
- 权限体系
- 长连接会话管理

## 当前实现

- `trendradar-mcp` 通过 stdin 读取 JSON 请求、stdout 返回 JSON 响应
- 工具入口不复用现有 CLI 输出
- 当前更接近“最小查询型工具服务”，后续可继续向完整 MCP 协议靠拢

## 后续建议

后续如继续推进，可补：

1. 更贴近 MCP 的协议字段和错误模型
2. tool schema 描述增强
3. 独立 query service 文档
