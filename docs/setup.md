# Setup

[WebJitsu README](../README.md) · [Configuration](configuration.md) · [Tool guide](tools.md)

WebJitsu needs a Chromium-family browser on the host running the process. The npm launcher also requires Node.js 14 or later. The standalone Rust binary runs without the npm launcher.

## Install: source only

This fork is **not published to npm and publishes no release binaries** — the
upstream npm packages (`bladebro`, `bladebro-linux-x64`, …) and their GitHub
releases belong to [upstream](https://github.com/dondai44423/bladebro), and
installing them gets you a build with the real-browser lane this fork removes.

```sh
git clone https://github.com/JimJafar/webjitsu.git
cd webjitsu
cargo build --release                 # ./target/release/webjitsu
./scripts/fork-install.sh             # or: install to ~/.local/bin + verify the gates
./target/release/webjitsu doctor
```

`scripts/fork-install.sh` installs outside npm and then fails if `-v` does not
report the fork build or if `webjitsu rb use` succeeds — so a fork that had
silently reverted cannot install looking like upstream.

The package under `npm/webjitsu` exists so the CLI shim (`bin.js`), the DSH
plugin and the Pi extension can resolve that binary — from `WEBJITSU_BIN`, a
build in this tree, `~/.local/bin/webjitsu`, or an optional platform package you
packed yourself. On musl hosts, build from source (prebuilt Linux binaries expect
glibc 2.28+). See the [support matrix](../SUPPORT-MATRIX.md) for runtime coverage.

## MCP

Register the following entry in a client that accepts `mcpServers`.

```json
{
  "mcpServers": {
    "webjitsu": {
      "command": "webjitsu",
      "args": ["mcp"]
    }
  }
}
```

For a desktop client with a different PATH, put the absolute executable path in `command`. To launch through npm without a global install, use `"command": "npx"` with `"args": ["-y", "webjitsu", "mcp"]`. The server communicates over stdin and stdout. Client-side restart instructions depend on the host.

## Agent integrations

### Pi

```sh
pi install npm:webjitsu
```

Pi registers five direct MCP tools (`mcp__webjitsu__act`, `mcp__webjitsu__see`, `mcp__webjitsu__state`, `mcp__webjitsu__run`, `mcp__webjitsu__vision`) from the binary's MCP schema. Start a new session if the current one does not pick them up.

### DeepSeek Harness

In the web UI, use **Plugins**, or install into the web profile from the terminal.

```sh
dsh plugin --profile web add webjitsu
```

Restart the profile and start a new chat. DSH exposes the MCP tools under `mcp__webjitsu__`. Chrome must be installed on the DSH host. Keep a single server registered under the WebJitsu name.

For Desktop, use its Plugins page. For terminal setup, enable **Manage dsh Command**, fully quit Desktop, run its bundled command below, and reopen the app. The standalone npm DSH CLI cannot manage the app-owned profile.

```sh
dsh plugin --profile desktop add webjitsu
```

Use **Update** in Plugins or `dsh plugin --profile web update webjitsu`. Use `remove webjitsu` to uninstall from that profile. Package-manager release-age policies can delay version selection. You can request an exact published version with `webjitsu@<version>`, subject to that policy.

Optional profile configuration in `cordis.patch.yml`:

```yaml
- id: webjitsu
  config:
    binaryPath: /absolute/path/to/webjitsu
    env:
      CHROME_PATH: /absolute/path/to/chrome
      BLADE_HOME: /absolute/path/to/private/browser-data
    toolCallTimeoutMs: 180000
```

Omit `binaryPath` to use the package binary. Windows paths can use forward slashes. The default tool-call budget is 180 seconds. If you cancel after browser input has been dispatched, inspect the destination before issuing the action again.

### Hermes

```sh
webjitsu hermes on
```

Run it from whichever `webjitsu` you built (`scripts/fork-install.sh` puts one on
`~/.local/bin`).

Start a new Hermes chat and restart a running gateway or desktop client. Setup registers the standard MCP server and suppresses Hermes' built-in browser toolset. Web search and extraction keep their settings. Use a Hermes version with `config get --json --raw`, `config unset`, and `mcp test`.

```sh
webjitsu hermes status
webjitsu hermes off
```

Use the same `--profile` for setup and restore when choosing a named profile. `HERMES_HOME` and `--hermes /path/to/hermes` also select the target instance. Hermes may expose the tools through Tool Search. Interrupted changes can resume with `on` or restore with `off`. Avoid changing Hermes configuration concurrently with the switch.

## From source

The crate declares Rust 1.86 as its minimum and the checkout pins the toolchain used by CI. Use the repository pin for development.

```sh
git clone https://github.com/JimJafar/webjitsu.git
cd webjitsu
cargo build --release
./target/release/webjitsu doctor
```

To build upstream instead, clone
[the upstream repository](https://github.com/dondai44423/bladebro) — note that
its build re-adds the real-browser lane.

See [Contributing](../CONTRIBUTING.md) for the edit and verification workflow.

## Updating

There is no self-update in this fork: `webjitsu -u` and `webjitsu --rollback`
return `blocked:`, because installing upstream's release over this build would
restore the removed lane without telling you. Updating is a source operation:

```sh
git fetch upstream --tags
git merge upstream/<tag>
grep -rn "fork::" src/        # every gate must survive the merge
cargo build --release
```

Then restart the CLI daemon (`webjitsu stop`) and any MCP client process, so they
pick up the new executable. The full procedure, including what to do when a
merge hits a gate: [FORK.md](../FORK.md).
