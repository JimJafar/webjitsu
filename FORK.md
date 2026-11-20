# WebJitsu — a fork of Bladebro, with the real-browser lane removed

Fork of <https://github.com/dondai44423/bladebro> at `v4.8.2` (commit `bfdb5c3`),
modified by Jim Sangwine, and **renamed** Bladebro → WebJitsu. Upstream license
and copyright are unchanged ([Apache-2.0](LICENSE)); per §4(b) of that licence,
**this file is the prominent notice that the files listed here were changed**, and
how.

Repository: <https://github.com/JimJafar/webjitsu> · User-facing guide:
[README.md](README.md)

## Why the lane is gone

Upstream's `rb` ("real browser") lane lets the agent drive **your own** Chromium.
It enumerates your installed browsers and their profiles (`Local State`), copies a
profile into a fork-owned clone — upstream's own wording: *"Imports your profile
into a Bladebro-owned copy and automatically refreshes whenever your real browser
writes newer logins, cookies, or storage"* — launches your live profile, or
attaches to a browser you already started.

Two consequences make that the one feature worth deleting for an agent-driven
browser:

1. **A real profile is the union of every session you are logged into** — mail,
   banking, source control, your employer's internal tools. The agent reads
   untrusted page content as a matter of course, and page content is what steers
   an agent. Give it both and any page you visit is in a position to make the
   agent act *as you*: send the message, change the setting, export the data, buy
   the thing. No prompt or policy inside the tool can arbitrate this, because the
   capability and the exposure are the same feature. Upstream warns about it
   ("Real-browser mode can act in authenticated sessions. Choose the profile and
   account you want the agent to use before handing it control"); a build that
   cannot enter the lane does not need the warning.
2. **A copied profile is a bag of plaintext session tokens.** Chrome encrypts its
   cookie store at rest; a clone does not inherit that, and auto-refresh keeps the
   copy current.

The lane is deleted rather than defaulted off, so it cannot be re-enabled by a
config file that drifted, a stale `realbrowser.json` from an earlier install, or
an agent talking itself into it.

## What was changed

| Change | Where |
|---|---|
| **Real-browser lane removed** — `rb` refuses, the lane cannot be selected, a profile cannot be enumerated/copied/launched/attached | `src/fork.rs` + the gates below |
| **Self-update and rollback removed** — updating would install upstream's binary over this one and restore the lane silently | `src/updater/mod.rs`, `src/updater/version.rs`, `src/updater/doctor.rs` |
| **Renamed to WebJitsu** — repo, crate (`webjitsu`), lib, binary, npm packages (`webjitsu`, `webjitsu-linux-x64`, …), CLI text, help, docs, tests. Upstream URLs and CHANGELOG history are left as upstream's | 148 files |
| **npm publishing workflow deleted** (`.github/workflows/release.yml`) and `.github/FUNDING.yml` removed — a fork cannot publish upstream's packages or attest to its provenance | `.github/` |
| **Source-only install** with a script that verifies the gates | `scripts/fork-install.sh` |
| **Binary resolution for a source tree** — `WEBJITSU_BIN` → this tree's `target/{release,gate}` → `~/.local/bin/webjitsu` → optional platform package, identical in the CLI shim and the Pi extension | `npm/webjitsu/binary.cjs`, `npm/webjitsu/pi-extension.ts` |
| **Manifests repointed** at the fork, with `upstream`/`funding` fields crediting Bladebro | `npm/*/package.json` |

**Unchanged:** the isolated agent browser, all five tools and their schemas, the
stealth layer, site adapters, `state save|load`, the daemon, downloads,
artifacts, `doctor`, and every upstream behaviour that is not the lane.

## The gates

One module, greppable in one pass:

```sh
grep -rn "fork::" src/
```

| Gate | Where | What it makes impossible |
|---|---|---|
| `fork::REAL_BROWSER_DISABLED` → `lane_from()` | `src/realbrowser.rs` | The lane decision. Every reader funnels through this one function (`init_lane`, `refresh_lane`, the launch fingerprint, the daemon's serving lane), so neither config nor `BLADE_LANE=real` selects it |
| … → `config_from()` | `src/realbrowser/config.rs` | `enabled` is forced off on the way *in*, so no consumer anywhere can act on a `realbrowser.json` left behind by an upstream install |
| … → `import_template()` | `src/realbrowser/template.rs` | The profile copy itself, refused at the function — defence in depth, independent of the lane |
| … → `run_rb()` | `src/cli/rb.rs` | The whole CLI surface, before any config read or write. `rb status` and `rb off` still exit 0 and report the truth; everything else returns `blocked:` |
| … → help surface | `src/cli/help.rs` | `rb` is out of `help --json` and out of the command-suggestion vocabulary, so an agent is never offered a command this build refuses. `help rb` explains the removal |
| `fork::SELF_UPDATE_DISABLED` → `updater::run()` | `src/updater/mod.rs` | `update`, `-u`, `rollback`, `--rollback` |
| … → `fetch_latest_tag()` | `src/updater/version.rs` | All release-registry traffic (one chokepoint for `update`, `-v`, `doctor`), so the build never gets told to install an update it must not install. `-v` prints `fork no-rb` instead |

`tests/fork_no_realbrowser.rs` pins each guarantee at the process **and** library
boundary. `src/fork.rs` explains the gate style: guards are written
`if fork::CONST { return … }`, so upstream code below stays reachable to the
compiler (this project lints with `clippy -D warnings`) and re-enabling one
feature is a single `false`.

## Build, install, and the Pi extension

```sh
git clone https://github.com/JimJafar/webjitsu.git
cd webjitsu
cargo build --release
./scripts/fork-install.sh        # installs to ~/.local/bin/webjitsu + verifies the gates
pi install ./npm/webjitsu        # register the five tools in Pi
```

`pi install <path>` loads a local package **in place, without installing its
dependencies**, so the optional platform packages that upstream's extension
requires are never present. `pi-extension.ts` therefore searches
`WEBJITSU_BIN` → `../../target/release|gate/webjitsu` → `~/.local/bin/webjitsu` →
platform package, and the CLI shim (`binary.cjs`) uses the same order so pi and
the CLI cannot run different builds. Installed away from the source tree, set the
override:

```sh
export WEBJITSU_BIN=$HOME/.local/bin/webjitsu
```

Chrome must be reachable from the process that runs the driver: it searches
`CHROME_PATH` first, then `PATH`. Nothing persists the choice, so a machine whose
Chrome lives somewhere unusual (a Playwright download under
`~/.cache/ms-playwright`, for instance) needs it exported in the shell that starts
the agent — pi passes its own environment to the MCP server it spawns:

```sh
# a glob will not expand inside an assignment, so resolve it:
export CHROME_PATH=$(ls -d "$HOME"/.cache/ms-playwright/chromium-*/chrome-linux64/chrome | sort -V | tail -1)
```

`webjitsu doctor` reports what it found.

Check a running install:

```sh
webjitsu -v          # must report `fork no-rb`
webjitsu rb use      # must print `blocked:`
webjitsu rb status --json   # {"ok":true,"enabled":false,"disabled_in_this_build":true,…}
```

Nothing here is published to npm, and the fork deliberately does not claim
upstream's package names.

## Pulling from upstream

```sh
git fetch upstream --tags
git merge upstream/<tag>       # upstream = github.com/dondai44423/bladebro
grep -rn "fork::" src/         # every gate must survive
cargo fmt --check
cargo clippy --profile gate --all-targets -- -D warnings
cargo test --profile gate
```

A merge conflict in any of the gate sites above is the signal to look at. The
upstream tests in `tests/rb_clone.rs` and `tests/rb_discovery.rs` are marked
`#[ignore]` here (they drive the removed lane) and are kept so upstream's coverage
survives the rebase and a future re-enable.

On a host without Xvfb, `cargo test --profile gate` fails five of upstream's
`sigint_*` tests in `tests/launch_failure_cleanup.rs` (timing-sensitive
signal-during-launch assertions) while passing them in isolation:

```sh
cargo test --profile gate --test launch_failure_cleanup   # 10 passed
```

**This is upstream's behaviour, not the fork's** — verified by running upstream
`v4.8.2` (`bfdb5c3`) unmodified in a worktree on the same machine: the same five
fail in a full run and pass in isolation. Run the file on its own when a full run
turns red there.

## Known gaps — what this fork does not protect you from

* **`--host` / `--port` still attach to an already-running browser**
  (`src/cli.rs`). It survives because you have to arm that browser with a debug
  endpoint yourself: a deliberate operator action, not a config that drifted on.
  Say the word and it gets the same gate.
* **Cookie persistence is plaintext.** `logins.json` (0600) snapshots the live
  cookie store of the *agent* browser so logins survive restarts. Anything running
  as you can read those tokens; Chrome's own store is encrypted. Log into the
  agent browser only with accounts whose compromise you can tolerate.
* **The stealth layer is untouched** — fingerprint and behavioural spoofing, and
  the site adapters that work around bot walls. Automating sites that forbid it is
  a ToS and account-ban risk you carry, not this fork's.
* **Nothing here defends the agent from a page that tells it to leak what it can
  see.** Removing the lane removes *your identity* from the blast radius; page
  content still reaches a model that can act. Keep the agent browser's profile
  sparse and treat its sessions as secrets.

## Upstream

Bladebro is Bishesh Bhandari's project, and a well-built one: SLSA provenance on
the npm packages, checksums verified fail-closed, no install-time scripts, no
telemetry endpoint in the shipped binary, owner-only daemon socket and sidecar.
If any of that matters to you, [upstream](https://github.com/dondai44423/bladebro)
is worth a star, and its
[sponsorship](https://github.com/dondai44423/bladebro#sponsors) is how the
maintenance is funded. The `rb` lane is a genuinely capable feature for a human
holding the wheel; this fork exists because of one threat model, not a criticism.
