#!/usr/bin/env sh
set -eu

REPO="${BYI_INSTALL_REPO:-wbytts/byi}"
INSTALL_DIR="${BYI_INSTALL_DIR:-$HOME/.local/bin}"
BINARY_NAME="byi"

log() {
  printf '%s\n' "$1"
}

detect_target() {
  os="$(uname -s)"
  arch="$(uname -m)"

  # 平台名必须和 GitHub Release workflow 产物后缀保持一致。
  case "$os" in
    Darwin) platform="apple-darwin" ;;
    Linux) platform="unknown-linux-gnu" ;;
    *) log "不支持的系统: $os"; exit 1 ;;
  esac

  case "$arch" in
    x86_64 | amd64) cpu="x86_64" ;;
    arm64 | aarch64) cpu="aarch64" ;;
    *) log "不支持的架构: $arch"; exit 1 ;;
  esac

  printf '%s-%s' "$cpu" "$platform"
}

download_file() {
  url="$1"
  output="$2"

  if command -v curl >/dev/null 2>&1; then
    curl --fail --location --show-error --silent "$url" --output "$output"
    return
  fi

  if command -v wget >/dev/null 2>&1; then
    wget --quiet "$url" --output-document "$output"
    return
  fi

  log "需要安装 curl 或 wget 后再执行安装。"
  exit 1
}

target="$(detect_target)"
asset="${BINARY_NAME}-${target}.tar.gz"
url="https://github.com/${REPO}/releases/latest/download/${asset}"
tmp_dir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT INT TERM

log "下载 ${url}"
download_file "$url" "$tmp_dir/$asset"

# 解压后只安装单个 CLI 二进制，不修改 shell 配置。
mkdir -p "$INSTALL_DIR"
tar -xzf "$tmp_dir/$asset" -C "$tmp_dir"
install -m 0755 "$tmp_dir/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"

log "安装完成: $INSTALL_DIR/$BINARY_NAME"
log "如果命令不可用，请确认 $INSTALL_DIR 已加入 PATH。"
