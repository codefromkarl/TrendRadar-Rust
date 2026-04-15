# 开发记录：v2 两轮执行与 D1 部署标准化启动

## 基本信息

- 日期：2026-04-15
- 阶段：增量演进 / 生态扩展
- 主题：两轮拓展计划定稿，并启动第 1 轮 `D1` 部署标准化
- 目标：把后续工作从“零散增强项”收束成两轮可执行主线，并落下官方部署入口

## 本次完成内容

- 更新 `docs/extension-execution-plan.md`，明确第 1 轮 `v2.0.0-beta` 与第 2 轮 `v2.1.0`
- 更新 `docs/roadmap.md` 与 `README.md`，同步当前执行轮次与部署文档入口
- 新增 `docs/deployment.md`
- 新增官方 `Dockerfile`
- 新增 `deploy/docker-compose.yml`
- 新增 `deploy/systemd/trendradar.service` 与 `deploy/systemd/trendradar.timer`
- 新增 `deploy/examples/config.rss.json` 作为最小可运行配置模板

## 关键决策

### 决策 1

- 决策内容：第 1 轮先做“对外可用化”，不先碰真实云存储和真实 LLM provider
- 原因：当前仓库最缺的是可部署、可展示、可替代，而不是更多内部扩展点

### 决策 2

- 决策内容：Docker 入口按 one-shot 任务设计
- 原因：当前 `trendradar` 是一次执行一轮 pipeline 的 CLI，不是常驻服务；把它包装成长期空转容器会误导部署模型

## 下一步

- 验证 Docker 镜像构建与 `--dry-run` 路径
- 继续推进第 1 轮剩余任务：`D2`、`B3`、`B1`、`D3`
