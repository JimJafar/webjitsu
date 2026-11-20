# WebJitsu

**A fork of [Bladebro](https://github.com/dondai44423/bladebro) with the real-browser lane removed**, so an agent you point at the web can never act as *you*.

Everything else is upstream's: a Rust binary that drives a Chromium-family browser over CDP and exposes it as five MCP tools — `act`, `see`, `state`, `run`, `vision` — with a persistent Live Page Model, delta-first output for token efficiency, and the stealth layer.

- Fork of `dondai44423/bladebro` v4.8.2 (`bfdb5c3`), Apache-2.0. Upstream's authorship and license are unchanged.
- Full gate-by-gate record: **[FORK.md](FORK.md)**.
- Not published to npm. This fork is built from source (see [Install](#install)).

## Why it's forked

Upstream's `bladebro rb` ("real browser") lane lets the agent drive **your own** Chromium:

| `rb` mechanism | What upstream does |
|---|---|
| `clone` (default) | Copies your profile into a Bladebro-owned clone, then "automatically refreshes whenever your real browser writes newer logins, cookies, or storage" |
| `profile` | Launches your browser on your **live** profile directory |
| `attach` | Connects to a browser you already started, through its debug endpoint |

Upstream is candid about it: *"Real-browser mode can act in authenticated sessions. Choose the profile and account you want the agent to use before handing it control."*

That is the whole problem for an agent-driven browser. A real profile is the union of every session you are logged into — mail, banking, source control, your employer's internal tools. Meanwhile the agent reads untrusted page content as a matter of course, and page content is exactly what steers an agent. Put those in one process and any page you visit is in a position to make the agent act as you: send the message, change the setting, export the data, buy the thing. No prompt or policy inside the tool can arbitrate that, because the capability and the exposure are the same feature.

The clone has a second, quieter cost. Chrome encrypts its cookie store at rest; a copied profile does not inherit that, so a clone — kept current by auto-refresh — is a bag of plaintext session tokens on disk, and the OS account is the only thing guarding it.

So the lane is deleted rather than defaulted off: the tool cannot enter it, and you cannot misconfigure your way back in.

## What has been changed

| Change | Effect |
|---|---|
| **Real-browser lane removed** | Every `webjitsu rb` subcommand returns `blocked:`. The lane cannot be selected by config or by `BLADE_LANE=real`; a profile cannot be enumerated, copied, launched, or attached. `rb status` and `rb off` still exit 0 and report the truth, so probes and setup scripts keep working |
| **Self-update and rollback removed** | `update`, `-u`, `--rollback` return `blocked:`, and the release registry is never contacted. Left in place, `bladebro update` would download upstream's binary, swap it into this path, and silently restore the lane — the fork would have lasted until the next update. `webjitsu -v` prints `fork no-rb` instead of nagging |
| **Renamed to WebJitsu** | Repo, crate, binary, npm package names (`webjitsu`, `webjitsu-linux-x64`, …), CLI text and docs. Upstream's npm names are not claimed: nothing here is published, and a different build under the same name is how supply-chain confusion starts |
| **npm publishing workflow deleted** | `.github/workflows/release.yml` went with it: a fork has no business publishing these packages, and its CI cannot attest to upstream's provenance |
| **Source-only install** | `cargo build --release`, or `scripts/fork-install.sh`, which installs outside npm and then verifies its own gates |
| **Pi extension resolves a local build** | Upstream found the binary only through optional npm platform packages. `pi install ./npm/webjitsu` loads the package in place without installing dependencies, so that route is empty here; resolution is now `WEBJITSU_BIN` → a build in this source tree → `~/.local/bin/webjitsu` → the platform package |

**Unchanged:** the isolated agent browser, all five tools and their schemas, the stealth and anti-detection layer, site adapters (Reddit/X/HN), `state save|load` for the agent browser's own sessions, the daemon, downloads, artifacts, `doctor`, and every upstream behaviour that isn't the lane.

**Not fixed by this fork** — read [FORK.md](FORK.md#known-gaps) before assuming otherwise: `--host`/`--port` can still attach to a browser you armed with a debug endpoint; `logins.json` stores the *agent* browser's cookies as plaintext; the stealth layer's ToS and account-ban exposure is yours; and nothing here stops a page from prompt-injecting an agent into leaking what the agent can see.

## Install

Chrome or Chromium must be installed on the machine running WebJitsu.

```sh
git clone https://github.com/JimJafar/webjitsu.git
cd webjitsu
cargo build --release            # binary: target/release/webjitsu
./scripts/fork-install.sh        # optional: installs to ~/.local/bin/webjitsu
```

`scripts/fork-install.sh` builds, installs, then **proves the gates**: it fails if `-v` doesn't report `fork no-rb`, or if `rb use` succeeds. So a fork that had silently reverted cannot install looking like upstream.

### From the terminal

```sh
webjitsu nav https://example.com
webjitsu see content
webjitsu stop
```

The first browser command starts a local daemon; later calls reuse the same tabs and page state until you stop it. `webjitsu doctor` diagnoses Chrome and the display.

### Connect an agent

Point your MCP client at the binary you built — not at `npx webjitsu`, which does not exist:

```json
{
  "mcpServers": {
    "webjitsu": { "command": "/home/you/.local/bin/webjitsu", "args": ["mcp"] }
  }
}
```

### Install as a Pi extension

```sh
pi install ./npm/webjitsu
```

That registers the five tools in pi. The extension finds the binary in this order: `WEBJITSU_BIN`, a build in this source tree (`target/release`, then `target/gate`), `~/.local/bin/webjitsu`, then the optional platform package. If you run it from somewhere else, set the override:

```sh
export WEBJITSU_BIN=$HOME/.local/bin/webjitsu
```

The browser is found from `CHROME_PATH`, then `PATH`; pi passes its own
environment to the server it spawns, so if your Chrome lives somewhere unusual:

```sh
export CHROME_PATH=$(ls -d "$HOME"/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort -V | tail -1)
```

## The five tools

| Tool | What it does |
|---|---|
| `act` | Navigate and interact: click, type, fill, select, press, scroll, upload, download, wait, dialog, eval, extract, collect, paginate, batch, pdf — returns a verified verdict plus a delta |
| `see` | Read without acting: `model` (elements with refs), `content` (Markdown), `outline`, `extract` (auto/links/forms/json), `find`, `changes`, console and network logs, paged artifacts |
| `state` | Cookies, localStorage, sessionStorage, tabs, and the agent browser's own `save` / `load` / `compress` / `block` |
| `run` | Batch steps with `if`/`while` branching and inline reads |
| `vision` | Screenshot to a file, with optional element marks |

`webjitsu help --json` emits the same schemas the MCP server serves.

## How it works

One Rust binary owns a daemon that keeps Chrome alive between calls, so tabs, refs and page state persist. `see` builds a Live Page Model with stable element refs that self-heal on the page; `act` verifies its own result by reading the page back, and marks an unknown mutation `degraded:true` instead of claiming success. Stealth is layered: CDP protocol choices, User-Agent and Client Hints, native-`toString` masking, canvas/audio noise seeded per install, WebRTC leak prevention, Bezier mouse paths and log-normal typing cadence, headful on a virtual display. [Architecture](docs/architecture.md) · [Browser modes](docs/browser-modes.md) · [Stealth bench](STEALTH_BENCH.md).

The browser it drives is a dedicated profile under the WebJitsu data directory (`~/.blade`, or `BLADE_HOME`). It has no view of your real browser, and your real browser has no view of it.

## Updating from upstream

```sh
git fetch upstream --tags
git merge upstream/<tag>
grep -rn "fork::" src/     # every gate must survive
cargo test --profile gate
```

Details, the gate table, and a host-dependent test caveat: [FORK.md](FORK.md).

## Docs

| Guide | Go here for |
|---|---|
| [FORK.md](FORK.md) | Why it's forked, every gate, re-enabling a removed feature |
| [Setup](docs/setup.md) | Source builds, MCP, Pi, DSH, Hermes, displays |
| [Tools](docs/tools.md) | Syntax, addressing, tabs and files, plans, JSON output |
| [Browser modes](docs/browser-modes.md) | Profiles, storage options, pause/idle (its `rb` sections describe upstream, not this fork) |
| [Configuration](docs/configuration.md) | Discovery, proxies, Linux displays, troubleshooting |
| [Architecture](docs/architecture.md) | Live Page Model, stable refs, recovery, source layout |
| [Support matrix](SUPPORT-MATRIX.md) | What was tested, on which systems |

## Upstream and license

WebJitsu is a fork of **[Bladebro](https://github.com/dondai44423/bladebro)** by Bishesh Bhandari, used under Apache-2.0 ([LICENSE](LICENSE)); the modified files are recorded in [FORK.md](FORK.md) per §4(b).

Upstream is a well-built project and worth supporting if you use its code: SLSA provenance on its npm packages, fail-closed checksum verification, no install-time scripts, no telemetry endpoint in the shipped binary, owner-only daemon socket and sidecar. Its [`rb` lane](https://github.com/dondai44423/bladebro#browser-modes) is a genuinely capable feature for a human holding the wheel, and its [sponsorship](https://github.com/dondai44423/bladebro#sponsors) funds the maintenance. This fork exists because of one threat model, not a criticism.
