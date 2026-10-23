#!/bin/sh
# adapted from comparegraph scripts/install.sh
# Installs the jurygraph release binary for this platform and checks it against the release's SHA256SUMS.
#   curl -fsSL https://raw.githubusercontent.com/massrelay-ai/jurygraph/main/scripts/install.sh | sh
# Environment:
#   JURYGRAPH_VERSION      a tag such as v0.1.0 (default: the latest release)
#   JURYGRAPH_INSTALL_DIR  where the binary goes (default: ~/.local/bin)
#   JURYGRAPH_BASE_URL     where the assets are read from (default: the GitHub release; tests point it at a file:// dir)
# Platforms: linux-x64, linux-arm64, darwin-arm64, darwin-x64. Windows: download the release asset by hand. Anything else: build from source.
# The whole script is one function called on the last line, so a download cut short runs nothing.
main() {
  set -eu
  repo="massrelay-ai/jurygraph"
  version="${JURYGRAPH_VERSION:-latest}"
  dir="${JURYGRAPH_INSTALL_DIR:-$HOME/.local/bin}"

  os="$(uname -s)"; arch="$(uname -m)"
  case "$os/$arch" in
    Linux/x86_64|Linux/amd64) platform=linux-x64 ;;
    Linux/aarch64|Linux/arm64) platform=linux-arm64 ;;
    Darwin/arm64|Darwin/aarch64) platform=darwin-arm64 ;;
    Darwin/x86_64) platform=darwin-x64 ;;
    *) echo "jurygraph: no prebuilt binary for $os/$arch; build from source:" >&2
       echo "  cargo install --locked --git https://github.com/$repo jurygraph" >&2; exit 2 ;;
  esac
  asset="jurygraph-$platform"

  if [ -n "${JURYGRAPH_BASE_URL:-}" ]; then base="$JURYGRAPH_BASE_URL"
  elif [ "$version" = latest ]; then base="https://github.com/$repo/releases/latest/download"
  else base="https://github.com/$repo/releases/download/$version"; fi

  if command -v curl >/dev/null 2>&1; then fetch() { curl -fsSL -o "$2" "$1"; }
  elif command -v wget >/dev/null 2>&1; then fetch() { wget -q -O "$2" "$1"; }
  else echo "jurygraph: curl or wget is needed" >&2; exit 2; fi
  if command -v sha256sum >/dev/null 2>&1; then sum() { sha256sum "$1" | cut -d' ' -f1; }
  elif command -v shasum >/dev/null 2>&1; then sum() { shasum -a 256 "$1" | cut -d' ' -f1; }
  else echo "jurygraph: sha256sum or shasum is needed" >&2; exit 2; fi

  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' EXIT
  echo "jurygraph: fetching $asset ($version)"
  fetch "$base/$asset" "$tmp/$asset"
  fetch "$base/SHA256SUMS" "$tmp/SHA256SUMS"
  expected="$(grep -E "[[:space:]]\*?$asset\$" "$tmp/SHA256SUMS" | head -1 | cut -d' ' -f1)"
  [ -n "$expected" ] || { echo "jurygraph: $asset is not listed in SHA256SUMS" >&2; exit 1; }
  actual="$(sum "$tmp/$asset")"
  [ "$actual" = "$expected" ] || {
    echo "jurygraph: checksum mismatch for $asset" >&2
    echo "  expected $expected" >&2; echo "  got      $actual" >&2
    echo "  nothing was installed" >&2; exit 1; }

  mkdir -p "$dir"
  cp "$tmp/$asset" "$dir/jurygraph.tmp" && chmod 755 "$dir/jurygraph.tmp" && mv "$dir/jurygraph.tmp" "$dir/jurygraph"
  echo "jurygraph: installed $dir/jurygraph ($("$dir/jurygraph" --version 2>/dev/null || echo "version unknown"))"
  case ":$PATH:" in *":$dir:"*) ;; *) echo "jurygraph: add $dir to your PATH" ;; esac
  echo "jurygraph: next, wire your agents: jurygraph setup --apply   (then: jurygraph doctor)"
}
main "$@"
