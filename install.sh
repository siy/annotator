#!/bin/sh
set -e

REPO="siy/annotator"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

detect_target() {
  os=$(uname -s)
  arch=$(uname -m)

  case "$os" in
    Linux)  os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *)      echo "Unsupported OS: $os" >&2; exit 1 ;;
  esac

  case "$arch" in
    x86_64|amd64)  arch_part="x86_64" ;;
    aarch64|arm64) arch_part="aarch64" ;;
    *)             echo "Unsupported architecture: $arch" >&2; exit 1 ;;
  esac

  echo "${arch_part}-${os_part}"
}

get_latest_version() {
  curl -sL "https://api.github.com/repos/${REPO}/releases/latest" \
    | grep '"tag_name"' \
    | head -1 \
    | sed 's/.*"tag_name": *"//;s/".*//'
}

main() {
  target=$(detect_target)
  version="${1:-$(get_latest_version)}"

  if [ -z "$version" ]; then
    echo "Could not determine latest version" >&2
    exit 1
  fi

  url="https://github.com/${REPO}/releases/download/${version}/annotator-${target}.tar.gz"
  echo "Downloading annotator ${version} for ${target}..."

  tmpdir=$(mktemp -d)
  trap 'rm -rf "$tmpdir"' EXIT

  curl -sL "$url" | tar xz -C "$tmpdir"

  if [ -w "$INSTALL_DIR" ]; then
    mv "$tmpdir/annotator" "$INSTALL_DIR/annotator"
  else
    sudo mv "$tmpdir/annotator" "$INSTALL_DIR/annotator"
  fi

  echo "Installed annotator to ${INSTALL_DIR}/annotator"
}

main "$@"
