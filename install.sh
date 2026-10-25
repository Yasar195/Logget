#!/usr/bin/env bash
set -euo pipefail

REPO="${LOGGET_REPO:-${1:-}}"
ASSET="logget-linux-amd64.tar.gz"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [[ -z "$REPO" ]]; then
  echo "Usage: curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/main/install.sh | sudo bash -s -- <owner>/<repo>" >&2
  echo "Or set LOGGET_REPO=<owner>/<repo>" >&2
  exit 1
fi

URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"

echo "Downloading logget from ${URL}"
curl -fsSL "$URL" -o "$TMP_DIR/${ASSET}"

mkdir -p "$INSTALL_DIR"
cd "$TMP_DIR"
tar -xzf "$ASSET"
install -m 0755 logget-linux-amd64 "$INSTALL_DIR/logget"

echo "logget installed to ${INSTALL_DIR}/logget"
echo "Run it with: logget"
