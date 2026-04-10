# `app` 实施骨架

## 目标

在各核心模块最小闭环完成后，负责薄编排和系统性测试挂载。

## 前置依赖

- `config`
- `schedule`
- `analyze`
- `fetch`
- `storage`
- `report`

## 输入与输出

- 输入：各模块公开接口
- 输出：最小 pipeline、系统 fixture、编排入口

## 本轮范围

- 串接模块接口
- 保持 `app` 薄层
- 添加系统性测试挂载点

## 暂不处理

- 复杂 CLI 交互
- 把业务逻辑下沉失败后反向塞入 `app`

## 建议子任务

- bootstrap 扩展
- pipeline 编排
- 系统测试

## 完成定义

- 至少一条完整系统链路可跑通
- `app` 不承载核心业务规则
- 系统 fixture 可复查

## 验证命令

```bash
cargo test -p trendradar-app
```
