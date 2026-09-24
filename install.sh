#!/bin/sh
# Solyn installer for Linux.
# Usage: curl -fsSL https://raw.githubusercontent.com/Keeferf/Solyn/main/install.sh | sh
set -eu

REPO="Keeferf/Solyn"

case "$(uname -m)" in
  x86_64 | amd64) ARCH="amd64" ;;
  aarch64 | arm64) ARCH="arm64" ;;
  *)
    echo "solyn: unsupported architecture: $(uname -m)" >&2
    exit 1
    ;;
esac

echo "Fetching latest release..."
release_json=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")

# Print the first asset download URL ending in $1 (e.g. _amd64.deb).
asset_url() {
  printf '%s' "$release_json" |
    grep -oE 'https://[^"[:space:]]+' |
    grep -E "$1\$" |
    head -n1
}

install_deb() {
  url=$(asset_url "_${ARCH}.deb")
  [ -n "$url" ] || return 1
  tmp=$(mktemp /tmp/solyn.XXXXXX.deb)
  echo "Downloading $url"
  curl -fL "$url" -o "$tmp"
  sudo dpkg -i "$tmp" || sudo apt-get -f install -y
  rm -f "$tmp"
}

install_appimage() {
  url=$(asset_url "_${ARCH}.AppImage")
  if [ -z "$url" ]; then
    echo "solyn: no release asset found for ${ARCH}" >&2
    exit 1
  fi
  dir="${HOME}/.local/bin"
  mkdir -p "$dir"
  echo "Downloading $url"
  curl -fL "$url" -o "${dir}/solyn"
  chmod +x "${dir}/solyn"
  echo "Installed solyn to ${dir}/solyn"
  case ":${PATH}:" in
    *":${dir}:"*) ;;
    *) echo "Add ${dir} to your PATH to run 'solyn'." ;;
  esac
}

if command -v dpkg >/dev/null 2>&1 && install_deb; then
  :
else
  install_appimage
fi

echo "Done. Solyn installs Ollama on first launch."
