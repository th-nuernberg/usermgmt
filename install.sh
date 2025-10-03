#!/usr/bin/env bash
set -euo pipefail

REPO="th-nuernberg/usermgmt"
DEFAULT_INSTALL_DIR="/usr/local/bin"

usage(){
  cat <<EOF
Usage: $0 [TAG] [INSTALL_DIR]
TAG: optional GitHub release tag (e.g. v0.6.3). If omitted, it uses latest.
INSTALL_DIR: optional install directory (default: ${DEFAULT_INSTALL_DIR})
EOF
}


if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [ "$#" -gt 2 ]; then
  usage
  exit 1
fi

TAG="${1:-latest}"
INSTALL_DIR="${2:-$DEFAULT_INSTALL_DIR}"

OS="$(uname -s)"
case "$OS" in
  Darwin) PLATFORM="mac" ;;
  Linux)  PLATFORM="linux" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 2
    ;;
esac

if [ "$PLATFORM" = "mac" ]; then
  ASSET="usermgmt-aarch64-apple-darwin.tar.gz"
elif [ "$PLATFORM" = "linux" ]; then
  ASSET="usermgmt-x86_64-unknown-linux-gnu.tar.gz"
fi

# Resolve tag (if latest, follow GitHub redirect to get actual tag)
if [ "$TAG" = "latest" ]; then
  echo "Resolving latest release tag..."
  final_url=$(curl -fsSL -o /dev/null -w '%{url_effective}' "https://github.com/${REPO}/releases/latest")
  TAG="$(basename "$final_url")"
  if [ -z "$TAG" ]; then
    echo "Failed to resolve latest release tag." >&2
    exit 3
  fi
  echo "Latest tag is: $TAG"
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${TAG}/${ASSET}"
echo "Will download: $DOWNLOAD_URL"

for cmd in curl tar; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Required tool '$cmd' not found. Install it and re-run." >&2
    exit 4
  fi
done

tmpdir="$(mktemp -d)"
cleanup(){ rm -rf "$tmpdir"; }
trap cleanup EXIT

outfile="$tmpdir/$ASSET"

echo "Downloading asset..."
if ! curl -fL --progress-bar -o "$outfile" "$DOWNLOAD_URL"; then
  echo "Download failed. Check tag ($TAG) and that asset exists: $ASSET" >&2
  exit 5
fi

echo "Extracting..."
tar -xzf "$outfile" -C "$tmpdir"

# locate executable: prefer file named 'usermgmt', otherwise first executable file found
found="$(find "$tmpdir" -type f -name 'usermgmt' -perm -111 -print -quit 2>/dev/null || true)"
if [ -z "$found" ]; then
  found="$(find "$tmpdir" -type f -perm -111 -print -quit 2>/dev/null || true)"
fi

if [ -z "$found" ]; then
  echo "Could not find an executable inside the archive." >&2
  echo "Archive contents:" >&2
  find "$tmpdir" -ls >&2 || true
  exit 6
fi

echo "Found executable: $found"

if [ ! -d "$INSTALL_DIR" ]; then
  echo "Creating install dir: $INSTALL_DIR"
  mkdir -p "$INSTALL_DIR"
fi

dst="$INSTALL_DIR/usermgmt"
echo "Installing to $dst"

if [ -w "$INSTALL_DIR" ]; then
  cp -f "$found" "$dst"
  chmod +x "$dst"
else
  echo "Need sudo to write to $INSTALL_DIR"
  sudo cp -f "$found" "$dst"
  sudo chmod +x "$dst"
fi

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
  echo "Warning: $INSTALL_DIR is not in PATH. You may need to add it to run 'usermgmt' directly."
fi

echo "Verifying installation..."
if ! "$dst" --version; then
  echo "Verification failed: installed binary did not run successfully." >&2
  exit 7
fi

echo "usermgmt installed successfully from release ${TAG} -> ${dst}"

