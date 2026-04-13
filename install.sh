#!/usr/bin/env bash
set -euo pipefail

REPO="${TRENDRADAR_REPO:-codefromkarl/TrendRadar-Rust}"
VERSION="${TRENDRADAR_VERSION:-latest}"
INSTALL_DIR="${TRENDRADAR_INSTALL_DIR:-$HOME/.local/bin}"
BIN_NAME="${TRENDRADAR_BIN_NAME:-trendradar}"

usage() {
    cat <<'EOF'
Usage: ./install.sh [--version <tag|latest>] [--dir <path>] [--repo <owner/repo>] [--help]

Install the prebuilt `trendradar` binary from GitHub Releases.

Options:
  --version <tag|latest>  Release tag to install. Default: latest
  --dir <path>            Install directory. Default: $HOME/.local/bin
  --repo <owner/repo>     GitHub repository. Default: codefromkarl/TrendRadar-Rust
  --help                  Show this help message

Environment overrides:
  TRENDRADAR_VERSION
  TRENDRADAR_INSTALL_DIR
  TRENDRADAR_REPO
  TRENDRADAR_BIN_NAME

Supported targets:
  - Linux x86_64
  - macOS arm64

Windows users should download the release asset manually.
EOF
}

log() {
    printf '[install] %s\n' "$1"
}

fail() {
    printf '[install] ERROR: %s\n' "$1" >&2
    exit 1
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --version)
            [[ $# -ge 2 ]] || fail "--version requires a value"
            VERSION="$2"
            shift 2
            ;;
        --dir)
            [[ $# -ge 2 ]] || fail "--dir requires a value"
            INSTALL_DIR="$2"
            shift 2
            ;;
        --repo)
            [[ $# -ge 2 ]] || fail "--repo requires a value"
            REPO="$2"
            shift 2
            ;;
        --help|-h)
            usage
            exit 0
            ;;
        *)
            fail "unknown argument: $1"
            ;;
    esac
done

detect_asset() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}:${arch}" in
        Linux:x86_64|Linux:amd64)
            printf 'trendradar-linux-x86_64'
            ;;
        Darwin:arm64|Darwin:aarch64)
            printf 'trendradar-macos-aarch64'
            ;;
        MINGW64_NT-*:x86_64|MSYS_NT-*:x86_64|CYGWIN_NT-*:x86_64)
            fail "Windows shell detected; please download trendradar-windows-x86_64.exe from Releases manually"
            ;;
        *)
            fail "unsupported platform: ${os} ${arch}"
            ;;
    esac
}

download_with() {
    local url="$1"
    local output="$2"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$output" "$url"
    else
        fail "curl or wget is required to download release assets"
    fi
}

normalize_version() {
    if [[ "$VERSION" == "latest" ]]; then
        printf 'latest'
    elif [[ "$VERSION" == v* ]]; then
        printf '%s' "$VERSION"
    else
        printf 'v%s' "$VERSION"
    fi
}

asset_name="$(detect_asset)"
normalized_version="$(normalize_version)"

if [[ "$normalized_version" == "latest" ]]; then
    download_url="https://github.com/${REPO}/releases/latest/download/${asset_name}"
else
    download_url="https://github.com/${REPO}/releases/download/${normalized_version}/${asset_name}"
fi

tmp_file="$(mktemp "${TMPDIR:-/tmp}/trendradar-install.XXXXXX")"
trap 'rm -f "$tmp_file"' EXIT

log "downloading ${download_url}"
download_with "$download_url" "$tmp_file"

mkdir -p "$INSTALL_DIR"
chmod +x "$tmp_file"
install -m 0755 "$tmp_file" "${INSTALL_DIR}/${BIN_NAME}"

log "installed ${BIN_NAME} to ${INSTALL_DIR}/${BIN_NAME}"
if command -v "$BIN_NAME" >/dev/null 2>&1; then
    log "binary is already on PATH"
else
    log "add ${INSTALL_DIR} to PATH if needed"
fi
log "run '${BIN_NAME} --help' to verify the installation"
