# Bladebro fork — the real-browser lane is removed

Fork of <https://github.com/dondai44423/bladebro> at `v4.8.2` (commit `bfdb5c3`),
modified by Jim Sangwine. Upstream license and copyright are unchanged
([Apache-2.0](LICENSE)); per §4(b) of that licence, **this file is the prominent
notice that the files listed below were changed**, and how.

Upstream's README describes the tool and all of its features. Read
[README.md](README.md) for the tool, and this file for the delta.

## What this build does not do

`bladebro rb` — the "real browser" lane — is closed. In upstream, that lane lets
the agent drive **your own** Chromium: it enumerates your installed browsers and
their profiles (`Local State`), copies a profile into a Bladebro-owned clone
("imports your profile … automatically refreshes whenever your real browser
writes newer logins, cookies, or storage"), launches your live profile, or
attaches to a browser you already started.

Two consequences made that the one thing worth removing for an agent-driven
browser:

1. **A real profile is the union of every session you are logged into.** Once the
   agent can act in it, a hostile page's text — which the agent reads as a matter
   of course — is in a position to make the agent act *as you*: send a message,
   change a setting, buy something, export data. The tool cannot arbitrate that,
   because the capability and the exposure are the same feature. Upstream warns
   about it in `docs/browser-modes.md`; a build that cannot enter the lane does
   not need the warning.
2. **A copied profile is a bag of plaintext session tokens on disk.** Chrome
   keeps its own cookie store encrypted at rest; a clone does not inherit that,
   and the auto-refresh keeps it current.

Everything else about Bladebro is intact: the isolated agent browser, the five
MCP tools (`act`, `see`, `state`, `run`, `vision`), the stealth layer, site
adapters, `state save|load` for the agent browser's own sessions, the daemon,
downloads and artifacts.

## The gates

One module, greppable in one pass:

```sh
grep -rn "fork::" src/
```

| Gate | Where | What it makes impossible |
|---|---|---|
| `fork::REAL_BROWSER_DISABLED` → `lane_from()` | `src/realbrowser.rs` | The lane decision. Every reader funnels through this one function (`init_lane`, `refresh_lane`, the launch fingerprint, the daemon's serving lane), so neither config nor `BLADE_LANE=real` selects it. |
| … → `config_from()` | `src/realbrowser/config.rs` | `enabled` is forced off on the way *in*, so no consumer anywhere can act on a `realbrowser.json` left behind by an upstream install. |
| … → `import_template()` | `src/realbrowser/template.rs` | The profile copy itself, refused at the function — defence in depth, independent of the lane. |
| … → `run_rb()` | `src/cli/rb.rs` | The whole CLI surface, before any config read or write. `rb status` and `rb off` still exit 0 and report the truth; everything else returns `blocked:`. |
| … → help surface | `src/cli/help.rs` | `rb` is out of `help --json` and out of the command-suggestion vocabulary, so an agent is never offered a command this build refuses. `help rb` explains the removal. |
| `fork::SELF_UPDATE_DISABLED` → `updater::run()` | `src/updater/mod.rs` | `update`, `-u`, `rollback`, `--rollback`. Without this gate `bladebro update` would download upstream's binary, swap it into this path, and silently put the lane back — the fork would last until the next update. |
| … → `fetch_latest_tag()` | `src/updater/version.rs` | All release-registry traffic (one chokepoint for `update`, `-v`, `doctor`), so the build never gets told to install an update it must not install. `-v` prints `fork no-rb` instead. |

`tests/fork_no_realbrowser.rs` pins each guarantee at both the process and the
library boundary. `src/fork.rs` explains the gate style: guards are written
`if fork::CONST { return … }`, so upstream code below stays reachable to the
compiler (this project lints with `clippy -D warnings`) and a re-enable is one
`false`.

## Build and install

```sh
git clone https://github.com/JimJafar/bladebro.git
cd bladebro
cargo build --release                    # binary: target/release/bladebro
./scripts/fork-install.sh                # optional: installs to ~/.local/bin/bladebro
```

Point your MCP client at **this** binary, not at `npx bladebro` / the npm
package — npm serves upstream's build, which has the lane:

```json
{ "mcpServers": { "bladebro": { "command": "/home/you/.local/bin/bladebro", "args": ["mcp"] } } }
```

Check it took: `bladebro -v` must show `fork no-rb`, and `bladebro rb use` must
print `blocked:`.

Do not install this fork as `bladebro` on npm: that name is upstream's, and a
different build under the same name is how supply-chain confusion starts. If you
publish one, rename it (e.g. `bladebro-no-rb`).

## Pulling from upstream

```sh
git fetch upstream --tags
git merge upstream/v<new>       # or rebase your commits on it
grep -rn "fork::" src/          # confirm every gate survived
cargo clippy --profile gate --all-targets -- -D warnings
cargo test --profile gate
```

A merge conflict in any of the seven gate sites above is the signal to look at.
The tests in `tests/rb_clone.rs` and `tests/rb_discovery.rs` are marked
`#[ignore]` here (they drive the removed lane) — they are kept so upstream's
coverage survives the rebase, and they will compile against a future re-enable.

## Known gaps — what this fork does *not* protect you from

* **`--host` / `--port` still attach to an already-running browser**
  (`src/cli.rs`). It survives because you have to arm that browser with a debug
  endpoint yourself; it is a deliberate operator action, not a config that drifted
  on. Say the word and it gets the same gate.
* **Cookie persistence is plaintext.** `logins.json` (0600) snapshots the live
  cookie store of the *agent* browser so logins survive restarts. Anything
  running as you can read those tokens; Chrome's own store is encrypted. Log into
  the agent browser only with accounts whose compromise you can tolerate.
* **The stealth layer is untouched** — fingerprint and behavioural spoofing,
  tracker-shaped page reads. Using it against sites that forbid automation is a
  ToS and account-ban risk you carry, not this fork's.
* **Nothing here defends the agent from a page that tells it to leak what it can
  see.** Removing the lane removes your identity from the blast radius; the page
  content still goes to a model that can act.

## Upstream

Bladebro is Bishesh Bhandari's project, and a well-built one: SLSA provenance on
the npm packages, checksums verified fail-closed, no install-time scripts, no
telemetry endpoint in the shipped binary, owner-only daemon socket and sidecar.
If any of that matters to you, [upstream](https://github.com/dondai44423/bladebro)
is worth a star, and the sponsor row in the README is how its maintenance is
funded. This fork exists because of a threat model, not a criticism.
