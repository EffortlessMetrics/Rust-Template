#!/usr/bin/env bash
set -euo pipefail

sha_check() {

# usage: sha_check <file> <key>
local file="$1" key="$2"
local ck="scripts/tools.sha256"
if [ -f "$ck" ]; then
  local expect
  expect=$(awk -v k="$key" '$1==k {print $2}' "$ck" | head -n1)
  if [ -n "${expect:-}" ]:
    if command -v sha256sum >/dev/null 2>&1; then
      echo "${expect}  ${file}" | sha256sum -c -
    else
      local got; got=$(shasum -a 256 "$file" | awk '{print $1}')
      test "$got" = "$expect"
    fi
    echo "Checksum OK for $key"
  else
    if [ "${ENFORCE_CHECKSUMS:-0}" = "1" ]; then
      echo "Checksum enforcement active but no entry for $key in scripts/tools.sha256" >&2
      exit 1
    else
      echo "No checksum entry for $key; skipping verification"
    fi
  fi
else
  if [ "${ENFORCE_CHECKSUMS:-0}" = "1" ]; then
    echo "Checksum enforcement active but scripts/tools.sha256 not present" >&2
    exit 1
  else
    echo "No scripts/tools.sha256 provided; skipping verification"
  fi
fi
}

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$ROOT/.tools/bin"
mkdir -p "$BIN"

OS="$(uname -s)"; ARCH="$(uname -m)"
case "$OS" in Linux) os="linux" ;; Darwin) os="darwin" ;; *) echo "Unsupported OS: $OS" >&2; exit 1 ;; esac
case "$ARCH" in x86_64|amd64) arch="amd64" ;; arm64|aarch64) arch="arm64" ;; *) echo "Unsupported ARCH: $ARCH" >&2; exit 1 ;; esac

install_oasdiff() {
  local v="2.19.1"; local tarball="oasdiff_${v}_${os}_${arch}.tar.gz"
  local url="https://github.com/Tufin/oasdiff/releases/download/v${v}/${tarball}"
  if ! [ -x "$BIN/oasdiff" ]; then
    curl -fsSL "$url" | tar -xz -C "$BIN" oasdiff
    chmod +x "$BIN/oasdiff"
    sha_check "$BIN/oasdiff" "oasdiff"
  fi
}
install_buf() {
  local v="1.45.0"
  local os_cap; os_cap="$(tr '[:lower:]' '[:upper:]' <<< "${os:0:1}")${os:1}"
  local buf_arch; case "$arch" in amd64) buf_arch="x86_64" ;; arm64) buf_arch="arm64" ;; esac
  local bin="buf-${os_cap}-${buf_arch}"
  local url="https://github.com/bufbuild/buf/releases/download/v${v}/${bin}"
  if ! [ -x "$BIN/buf" ]; then
    curl -sSL "$url" -o "$BIN/buf"; chmod +x "$BIN/buf"; sha_check "$BIN/buf" "buf"
  fi
}
install_atlas() {
  local v="0.21.1"; local bin="atlas-${os}-${arch}"
  local url="https://github.com/ariga/atlas/releases/download/v${v}/${bin}"
  if ! [ -x "$BIN/atlas" ]; then
    curl -sSL "$url" -o "$BIN/atlas"; chmod +x "$BIN/atlas"; sha_check "$BIN/atlas" "atlas"
  fi
}
install_oasdiff; install_buf; install_atlas
echo "Installed pinned CLIs into .tools/bin for ${os}/${arch}"
