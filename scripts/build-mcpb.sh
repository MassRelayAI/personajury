#!/usr/bin/env bash
# Build one .mcpb bundle for Claude Desktop: the manifest (with this platform) and the jurygraph binary, zipped.
# Usage: scripts/build-mcpb.sh <platform> <binary> [out-dir]     platform: linux-x64 | darwin-arm64 | darwin-x64 | win32-x64
# Writes <out-dir>/jurygraph-<platform>.mcpb and its .sha256 (default out-dir: dist), prints the sha256. No network.
set -euo pipefail
platform=${1:?platform (linux-x64 | darwin-arm64 | darwin-x64 | win32-x64)}
binary=${2:?path of the jurygraph binary built for that platform}
out=${3:-dist}
here=$(cd "$(dirname "$0")/.." && pwd)
case "$platform" in
  linux-x64) os=linux; exe=jurygraph ;;
  darwin-arm64 | darwin-x64) os=darwin; exe=jurygraph ;;
  win32-x64) os=win32; exe=jurygraph.exe ;;
  *) echo "unknown platform: $platform" >&2; exit 2 ;;
esac
[ -f "$binary" ] || { echo "no such binary: $binary" >&2; exit 2; }
stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT
mkdir -p "$stage/bin" "$out"
# the manifest names this platform; on Windows the binary carries its .exe
sed -e "s#\"PLATFORM\"#\"$os\"#" -e "s#bin/jurygraph#bin/$exe#g" "$here/packaging/mcpb/manifest.json" > "$stage/manifest.json"
cp "$binary" "$stage/bin/$exe"
chmod +x "$stage/bin/$exe"
bundle="$(cd "$out" && pwd)/jurygraph-$platform.mcpb"
rm -f "$bundle"
# -X drops extra file attributes, so the same inputs zip to the same layout
(cd "$stage" && zip -q -X -r "$bundle" manifest.json bin)
dir=$(dirname "$bundle"); name=$(basename "$bundle")
(cd "$dir" && (sha256sum "$name" 2>/dev/null || shasum -a 256 "$name") > "$name.sha256")
cut -d' ' -f1 "$bundle.sha256"
