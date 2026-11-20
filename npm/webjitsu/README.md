# webjitsu

The npm/pi package for **[WebJitsu](https://github.com/JimJafar/webjitsu)** — a fork of [Bladebro](https://github.com/dondai44423/bladebro) with the real-browser lane removed, so an agent you point at the web can never act as *you*.

Five MCP tools (`act`, `see`, `state`, `run`, `vision`) in one Rust binary. Why the fork exists and exactly what was changed: **[FORK.md](../FORK.md)** · [README](../README.md).

## This package is not on npm

The fork is source-only, and upstream's npm names are deliberately not claimed:

```sh
git clone https://github.com/JimJafar/webjitsu.git
cd webjitsu
cargo build --release          # the binary this package needs
./scripts/fork-install.sh      # or: install to ~/.local/bin/webjitsu and verify the gates
```

`bin.js` → `binary.cjs` resolves the executable in this order:

1. `WEBJITSU_BIN` (absolute path; a bad value is an error, not a silent fallback)
2. a build in this source tree — `target/release/webjitsu`, then `target/gate/webjitsu`
3. `~/.local/bin/webjitsu` (what `scripts/fork-install.sh` writes)
4. this platform's optional package (`webjitsu-linux-x64`, …) — for anyone who packs it locally

`pi-extension.ts` uses the same order, so pi and the CLI cannot end up running
different builds.

## Use as an MCP server

```json
{
  "mcpServers": {
    "webjitsu": { "command": "/home/you/.local/bin/webjitsu", "args": ["mcp"] }
  }
}
```

## Use as a Pi extension

```sh
pi install ./npm/webjitsu
```

Loaded from this path in place, the extension finds `../../target/release/webjitsu` (then `target/gate`) automatically. Installed elsewhere, set `WEBJITSU_BIN`.

## What is removed in this build

`webjitsu rb` (drive your own browser, enumerate its profiles, clone its cookies and logins, attach to a running one), plus `webjitsu update` / `--rollback` — because updating would install upstream's binary over this one and restore the lane. Both answer `blocked:`.
