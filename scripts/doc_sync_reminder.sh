#!/usr/bin/env bash
set -euo pipefail

base_ref="${1:-HEAD}"

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "[doc-sync] 当前目录不是 Git 仓库，跳过文档同步提醒。"
    exit 0
fi

if ! git rev-parse --verify "$base_ref" >/dev/null 2>&1; then
    echo "[doc-sync] 基准引用 '$base_ref' 不存在，跳过文档同步提醒。"
    exit 0
fi

changed_files="$(git diff --name-only "$base_ref"...HEAD 2>/dev/null || git diff --name-only "$base_ref" HEAD)"

if [[ -z "$changed_files" ]]; then
    echo "[doc-sync] 没有检测到变更。"
    exit 0
fi

has_docs_change=0
has_scope_change=0
has_large_change=0
crate_count=0

while IFS= read -r file; do
    [[ -z "$file" ]] && continue

    case "$file" in
        README.md|docs/*|.github/pull_request_template.md)
            has_docs_change=1
            ;;
    esac

    case "$file" in
        Cargo.toml|rust-toolchain.toml|justfile|clippy.toml|deny.toml|rustfmt.toml|.github/workflows/*)
            has_scope_change=1
            has_large_change=1
            ;;
        crates/*/Cargo.toml)
            has_scope_change=1
            has_large_change=1
            ;;
        crates/*/src/*|crates/*/tests/*)
            has_scope_change=1
            ;;
    esac
done <<< "$changed_files"

crate_count="$(printf '%s\n' "$changed_files" | sed -n 's#^crates/\([^/]*\)/.*#\1#p' | sort -u | wc -l | tr -d ' ')"

if [[ "$crate_count" -ge 2 ]]; then
    has_large_change=1
fi

if [[ "$has_scope_change" -eq 0 ]]; then
    echo "[doc-sync] 本次变更未命中需要文档同步提醒的范围。"
    exit 0
fi

if [[ "$has_docs_change" -eq 1 ]]; then
    echo "[doc-sync] 已检测到文档同步变更。"
    exit 0
fi

echo "[doc-sync] 提醒：检测到代码/配置变更，但没有发现 README 或 docs 变更。"

if [[ "$has_large_change" -eq 1 ]]; then
    echo "[doc-sync] 本次更像中大型变更，建议先补或同步更新以下文档之一："
    echo "  - README.md"
    echo "  - docs/architecture.md"
    echo "  - docs/migration-strategy.md"
    echo "  - docs/environment-setup.md"
    echo "  - docs/dev-journal/"
else
    echo "[doc-sync] 如果这是小型 bug 修复，可以暂不改正式文档，但建议至少确认是否需要写开发日志。"
fi

if [[ "${DOC_SYNC_STRICT:-0}" == "1" ]]; then
    echo "[doc-sync] 当前为严格模式，提醒视为失败。"
    exit 1
fi

exit 0
