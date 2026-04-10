#!/usr/bin/env bash
set -euo pipefail

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"

if [[ -z "$repo_root" ]]; then
    echo "[githooks] 当前目录不是 Git 仓库，无法安装 hooks。"
    exit 1
fi

git -C "$repo_root" config core.hooksPath .githooks
git -C "$repo_root" config commit.template .gitmessage

chmod +x "$repo_root/.githooks/pre-commit"
chmod +x "$repo_root/.githooks/pre-push"
chmod +x "$repo_root/.githooks/commit-msg"

echo "[githooks] 已将 core.hooksPath 设置为 .githooks"
echo "[githooks] 已将 commit.template 设置为 .gitmessage"
echo "[githooks] 已启用 pre-commit、commit-msg 和 pre-push hooks"
