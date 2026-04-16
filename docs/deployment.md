# 部署指南

## 目标

这份文档用于固定 TrendRadar Rust 第 1 轮拓展的官方部署入口。

当前默认支持两种方式：

- Docker one-shot 运行
- 二进制 + `systemd timer` 调度

## 部署模型

`trendradar` 当前是一次执行一轮 pipeline 的 CLI，不是常驻 Web 服务。

这意味着：

- 它适合被 `cron`、`systemd timer`、CI 或容器任务周期性调用
- 容器默认也应按 one-shot 方式运行，而不是包装成常驻空转进程

## 路径约定

为了避免与自动发现逻辑混淆，官方部署样例统一显式传入这两个路径：

- 配置文件：`/config/config.json`
- 数据库文件：`/data/trendradar.db`

对应 CLI 参数：

```bash
trendradar --config /config/config.json --db /data/trendradar.db
```

## 方式 1：Docker one-shot

### 1. 准备配置

先复制最小 RSS 示例配置：

```bash
cp deploy/examples/config.rss.json deploy/runtime/config.json
mkdir -p deploy/runtime/data
```

这个示例使用 Rust Blog RSS，可直接用于验证容器链路是否正常。

### 2. 构建镜像

```bash
cargo build --release -p trendradar-app
docker build -t trendradar:local .
```

### 3. Dry-run 自检

```bash
docker compose -f deploy/docker-compose.yml run --rm trendradar --dry-run
```

### 4. 执行一次真实抓取

```bash
docker compose -f deploy/docker-compose.yml run --rm trendradar
```

### 5. 说明

- `deploy/docker-compose.yml` 使用仓库根目录 `Dockerfile`
- 当前 Dockerfile 直接打包本机已构建的 release 二进制，因此首次构建前需要先执行 `cargo build --release -p trendradar-app`
- 镜像已内置 CA 证书，HTTPS RSS / API 请求可直接使用
- 镜像内 `trendradar` 用户固定为 `uid/gid 1000`，与常见本地工作目录权限更容易对齐
- `deploy/docker-compose.yml` 把 `--config /config/config.json --db /data/trendradar.db` 固定在 entrypoint 中，因此 `run --rm trendradar --dry-run` 不会再丢失配置路径
- 容器会把数据库写入 `deploy/runtime/data/trendradar.db`
- 如需改输出格式，可把 `json` 调整为 `html` / `both` / `table` / `markdown`

## 方式 2：二进制 + systemd timer

### 1. 安装二进制

任选一种：

```bash
curl -fsSL https://raw.githubusercontent.com/codefromkarl/TrendRadar-Rust/main/install.sh | bash
```

或：

```bash
cargo install --path crates/app --locked
```

### 2. 准备目录

```bash
sudo mkdir -p /etc/trendradar /var/lib/trendradar
sudo cp deploy/examples/config.rss.json /etc/trendradar/config.json
```

### 3. 安装 unit 文件

```bash
sudo cp deploy/systemd/trendradar.service /etc/systemd/system/
sudo cp deploy/systemd/trendradar.timer /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable --now trendradar.timer
```

### 4. 手动触发一次

```bash
sudo systemctl start trendradar.service
sudo journalctl -u trendradar.service -n 50 --no-pager
```

### 5. 调整频率

默认定时器为每 30 分钟执行一次：

```ini
OnUnitActiveSec=30min
```

如需修改，编辑 [trendradar.timer](/home/yuanzhi/Develop/ai-research/TrendRadar-Rust/deploy/systemd/trendradar.timer) 后重新加载：

```bash
sudo systemctl daemon-reload
sudo systemctl restart trendradar.timer
```

## 最小验证清单

部署完成后，至少确认下面 4 项：

1. `trendradar --version` 正常返回版本号
2. `--dry-run` 能正确加载配置
3. 首次执行后数据库文件已生成
4. 非法配置能返回非零退出码

## 当前边界

这份部署指南只覆盖第 1 轮的官方最小部署入口，不包含：

- 常驻 Web 服务模式
- Kubernetes / Helm
- Homebrew
- 完整云对象存储接入
- 完整 MCP 服务部署矩阵
