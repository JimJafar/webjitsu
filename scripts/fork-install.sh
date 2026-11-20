#!/usr/bin/env bash
# ── fork-install.sh: build this fork and install it outside npm ────────
# Run from project root:  ./scripts/fork-install.sh [--dest <dir>]
#
# Why a script instead of `npm install -g`: npm serves UPSTREAM's build, which
# still has the real-browser lane this fork removes (FORK.md). The binary has to
# come from this source tree, and your MCP client has to point at that path.
set -euo pipefail

DEST=""
if [[ "${1:-}" == "--dest" ]]; then
  DEST="${2:?--dest needs a directory}"
elif [[ -n "${XDG_BIN_HOME:-}" ]]; then
  DEST="$XDG_BIN_HOME"
else
  DEST="$HOME/.local/bin"
fi

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "› cargo build --release"
cargo build --release

mkdir -p "$DEST"
install -m 0755 target/release/bladebro "$DEST/bladebro"
BIN="$DEST/bladebro"

echo "› installed $BIN"

# Prove the gates rather than assuming them: a fork that silently reverted would
# otherwise install looking exactly like upstream.
if ! "$BIN" -v 2>&1 | grep -q 'no-rb'; then
  echo "✖ $BIN -v does not report the fork build — is this upstream's binary?" >&2
  exit 1
fi
if "$BIN" rb use >/dev/null 2>&1; then
  echo "✖ '$BIN rb use' succeeded — the real-browser lane is NOT closed here." >&2
  exit 1
fi
echo "› gates verified: -v reports fork no-rb, and 'rb use' is refused"

cat <<EOF

Point your MCP client at this path (not 'bladebro' from npm, not npx):

  {
    "mcpServers": {
      "bladebro": { "command": "$BIN", "args": ["mcp"] }
    }
  }

If a shell alias, an npm global install, or /usr/local/bin/bladebro is earlier
on your PATH, that copy still has the lane — check with:  command -v bladebro
EOF
