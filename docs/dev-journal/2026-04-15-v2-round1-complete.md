# 开发记录：第 1 轮 `v2.0.0-beta` 执行完成

## 基本信息

- 日期：2026-04-15
- 阶段：增量演进 / 生态扩展
- 主题：完成第 1 轮“对外可用化”
- 目标：收口部署、对外文档、平台扩展、通知扩展和运行稳定性口径

## 本轮完成内容

- `D1` 部署标准化
  - 新增 `Dockerfile`
  - 新增 `deploy/docker-compose.yml`
  - 新增 `deploy/systemd/trendradar.service`
  - 新增 `deploy/systemd/trendradar.timer`
  - 新增 `docs/deployment.md`
- `D2` 对外展示与交付文档
  - 新增 `docs/public-capability-overview.md`
  - 新增 `docs/runtime-stability.md`
  - README 新增文档入口
- `B3` 热榜平台扩展
  - `GenericHotlistParser` 兼容 `newsnow` 包装响应
  - 把 `douyin` / `wallstreetcn-hot` / `ifeng` / `tieba` 纳入显式支持
- `B1` 通知渠道扩展
  - 新增 `DiscordNotifier`
  - 新增 `NtfyNotifier`
  - 配置层新增 `discord_webhook_url` 与 `ntfy_topic_url`
- `D3` 长稳运行验证
  - 复用并整理系统层恢复与多轮一致性测试
  - 文档化部署前检查入口

## 验证结果

- `cargo test --workspace` 通过
- 当前工作区总数：`235 tests passed`
- `cargo build --release -p trendradar-app` 通过
- `docker run --rm trendradar:local --version` 通过
- `docker run --rm -v <config>:/config/config.json:ro trendradar:local --config /config/config.json --dry-run` 通过

## 阶段结论

第 1 轮已经把仓库从“核心迁移完成”推进到“可对外说明、可部署、可验证”的状态。

后续默认进入第 2 轮：

- `C1` 真实远程对象存储
- `C2` 真实 AI Provider
- `C3` MCP 协议补强
