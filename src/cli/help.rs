//! The CLI's self-teaching surfaces — `help`, `help --json`, `help <cmd>`,
//! `<cmd> --help`, and the typo suggester behind unknown-command errors.
//!
//! `help --json` is generated from the same `tools_to_json()` the MCP server
//! serves, so the two surfaces cannot drift.

use serde_json::{json, Value};

use crate::error::{BladeError, Result};

// ── Help ───────────────────────────────────────────────────────────────

/// The CLI's agent manual — served by `help --json`. This is the single
/// discovery surface: tool schemas (identical to MCP `tools/list`), the
/// command map, the output contract, and the workflow that gets the most
/// out of both.
const INSTRUCTIONS: &str = "\
WebJitsu CLI — drive a real, stealthy browser from shell commands.
SETUP: none. The first command auto-starts a background daemon (one Chrome for all later commands). --no-daemon forces isolated one-shot runs; 'webjitsu stop' cleans up.
DISCOVERY: 'webjitsu help' (human) — 'webjitsu help --json' (this document) — 'webjitsu help <command>' for one command in depth.
OUTPUT CONTRACT: --json on any command gives ONE JSON object on stdout: {\"ok\":bool,\"is_error\":bool,\"text\":string} (+ \"image_path\" for vision — the PNG is saved to a file, never inlined base64). Exit codes: 0 = success; 1 = the command ran but failed (read 'text' — it carries current page state so you can recover without an extra call); 2 = usage error (bad command/flag/argument — fix the command).
WORKFLOW: 1) Read-only URL? 'see content <url>' or 'see extract auto <url> --format json --block images,fonts' navigates and reads in ONE call. 2) Interactive flow? 'nav <url>' gives refs + a content preview; act by ref/text/label/selector ('act click \"Sign in\"' needs no see; refs self-heal within a page). 3) On list/search/product/profile pages, 'see extract auto' FIRST: one call returns structured items (site adapters cover Reddit/X/GitHub/HN/products; on Reddit POST pages it returns the full comment tree). 4) Collapse round-trips: 'act fill' for whole forms (supports nth/role per field); 'act batch' for linear sequences; 'run' for if/while branching or --format json step receipts. 5) Big JSON payloads: @file or - (stdin) — no shell-quoting games. 6) Keep budgets default; large outputs offload to artifact files read back via 'see --artifact <path>'.
RELIABILITY: real Chromium with the full stealth stack and human-like input; per-domain knowledge (settle timing, block config, bot risk) compounds across runs. Errors are loud, never silent: unknown flags and missing values fail with exit 2; navigation errors report exact HTTP status codes (e.g. HTTP 429 + Retry-After); failed actions return current page state for recovery. Vision is the last resort — refs and deltas are cheaper.
";

/// The human manual — printed by `webjitsu help` / `-h` / bare `webjitsu`.
pub fn help_text() -> String {
    crate::ui::style_help(&HELP_TEXT.replace("__VERSION__", env!("CARGO_PKG_VERSION")))
}

const HELP_TEXT: &str = r#"webjitsu __VERSION__ — agentic browser driver (CLI)

USAGE
  webjitsu <command> [args] [--json] [--no-daemon]
  webjitsu help [command]      this manual (start here)

The first command auto-starts a background daemon: ONE Chrome instance for
all later commands (no startup delay after the first launch). `webjitsu stop`
shuts it down. `--no-daemon` forces isolated one-shot runs.

COMMANDS
  nav <url> [--block <classes>]   navigate (refs + content preview; usually enough to act)
  see [mode] [url] [flags]        read without acting: model|content|outline,
                                  extract <auto|links|forms|json>, --find, --scope, --logs
  act <action> [args]             interact: click, type, fill, select, clear, press, scroll,
                                  hover, navigate, upload, download, wait, dialog, eval, collect,
                                  read, batch, pdf, back, forward, reload, save, load,
                                  open-tab, close-tab, switch-tab
  state <op> [args]               cookies, set-cookie, del-cookie, ls, ss, set-ls, set-ss,
                                  rm-ls, rm-ss, clear-ls, clear-ss, tabs, open-tab,
                                  close-tab, switch-tab, save, load, compress, block
  run '<json-steps>'              batch with if/while branching + inline see reads
  vision [--marks]                screenshot (saved to a file; path printed)
  daemon | stop                   manage the persistent Chrome session
  mcp                             MCP server on stdio — add to your agent's client config
  rb (removed in this build)      real-browser lane is deleted here: every rb command returns 'blocked:'
  audio [on|off|status]           browser sound output (muted by default; opt-in with 'audio on')
  hermes on|off|status             switch Hermes' browser to WebJitsu; reversible MCP setup
  audit                           stealth audit — vector suite + boot self-check + drift stamp
  update | -u [--check] [--force]  self-update; --rollback restores the previous binary
  doctor | -doc                   system diagnostics dossier (OS, browser, display, vault, release)
  -v | --version                  version + install method + update status
  help [command]                  this manual; 'help --json' for the machine version

QUICK START
  webjitsu nav example.com                      navigate
  webjitsu see model                            interactive elements with refs
  webjitsu see content                          page as clean markdown
  webjitsu see extract auto                     structured data (lists, posts, repos, products)
  webjitsu act click e5                         click by ref
  webjitsu act click "Sign in"                  click by text (no see needed)
  webjitsu act type e12 "hello world"           type into one field
  webjitsu act fill '{"e12":"John"}' --submit e20      fill a form in ONE call
  webjitsu act batch '[{"action":"navigate","url":"x.com"},{"action":"see","mode":"content"}]'
  webjitsu run @steps.json                      big payload from a file (- = stdin)
  webjitsu act eval "document.title"            evaluate JS

FLAGS
  --json         one JSON object on stdout: {"ok", "is_error", "text"} (+ "image_path" for vision)
  --no-daemon    one-shot mode: launch Chrome per command
  --host/--port  drive an already-running Chrome on this debug port (skips the daemon)
  --marks        vision: overlay numbered ref badges

EXIT CODES
  0  success
  1  the command ran but failed — read the output text (it carries page state for recovery)
  2  usage error — bad command/flag/argument; the message says how to fix it

ENVIRONMENT
  BLADE_HOME           isolate everything (daemon, Chrome, sessions, knowledge) in a directory
  BLADE_CMD_TIMEOUT    client wait for a daemon response, seconds (default 300)
  BLADE_IDLE_TIMEOUT   daemon idle before Chrome shuts down, seconds (default 600)
  BLADE_NO_COMPRESS=1  disable response compression
  BLADE_LANE           real|agent — force the real-browser lane for this process
  BLADE_RB_DEBUG=1     real lane: surface the browser's own stderr on launch failures
  BLADE_PLAIN          plain output — no ANSI even on a terminal
                       (NO_COLOR also disables; CLICOLOR_FORCE=1 forces color off-TTY)
  CHROME_PATH          override the Chrome/Chromium binary

Agents: `webjitsu help --json` returns the machine version of this manual —
tool schemas (same as MCP tools/list), per-command usage, exit codes, payload
conventions, and examples. Fetch it once, then drive the CLI with --json.
"#;

/// Closest known command for a mistyped/informal one, so the
/// unknown-command error gives a pointer instead of a dead end:
/// synonyms first (`browser` → `rb use`), then edit distance (≤2, or ≤3 for
/// inputs of six or more chars).
pub fn suggest_command(cmd: &str) -> Option<&'static str> {
    /// Words users reach for that map onto a real command.
    // FORK (FORK.md): upstream mapped "browser"/"browsers"/"lane"/"real" onto
    // the rb commands. Pointing an agent at a command this build refuses would
    // only waste its turn, so those aliases are gone.
    const ALIASES: &[(&str, &str)] = &[("mute", "audio"), ("unmute", "audio"), ("sound", "audio")];
    let lower = cmd.to_ascii_lowercase();
    if let Some((_, target)) = ALIASES.iter().find(|(a, _)| *a == lower) {
        return Some(target);
    }
    const COMMANDS: &[&str] = &[
        "nav", "see", "act", "state", "run", "vision", "daemon", "stop", "help",
        // FORK: "rb" / "realbrowser" removed from the suggestion vocabulary —
        // this build refuses them (see FORK.md).
        "audio", "hermes", "mcp", "audit", "probe", "targets", "update", "doctor", "rollback",
        "version",
    ];
    let mut best: Option<(usize, &'static str)> = None;
    for c in COMMANDS {
        let d = edit_distance(&lower, c);
        let limit = if lower.chars().count() >= 6 { 3 } else { 2 };
        if d <= limit && best.map(|(bd, _)| d < bd).unwrap_or(true) {
            best = Some((d, c));
        }
    }
    best.map(|(_, c)| c)
}

/// Plain Levenshtein over chars — short strings, no dependency.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// Canonical command name: resolve aliases to what the user actually types
/// on the CLI, so every help surface accepts the same spellings.
fn normalize_cmd(cmd: &str) -> &str {
    match cmd {
        "navigate" => "nav",
        "-u" => "update",
        "-doc" => "doctor",
        "--rollback" => "rollback",
        "-v" | "--version" => "version",
        _ => cmd,
    }
}

/// Per-command machine help (`help <cmd> --json` and the `commands` map in
/// the full JSON manual).
fn command_help_json(cmd: &str) -> Option<Value> {
    let cmd = normalize_cmd(cmd);
    let v = match cmd {
        "nav" => json!({
            "usage": "webjitsu nav <url> [--block <classes>]",
            "tool": "act",
            "args": { "url": "target URL; bare domains get https://" },
            "flags": { "--block": "images,fonts,media,trackers (remembered per domain)" },
            "examples": ["webjitsu nav example.com", "webjitsu nav https://x.com --block images,fonts --json"],
            "notes": ["returns refs + a content preview — often enough to act without a separate see"]
        }),
        "see" => json!({
            "usage": "webjitsu see [mode] [url] [extract <type>] [flags]",
            "tool": "see",
            "args": {
                "mode": "model (default) | content | outline",
                "url": "optional — navigates first",
                "extract": "auto | links | forms | json (json needs --template)"
            },
            "flags": {
                "--block": "with url: images,fonts,media,trackers to skip during load",
                "--filter": "role filter (model mode)",
                "--find": "search by text → refs (supports --format json)",
                "--scope": "ref id — read one element's subtree",
                "--content": "include page text in model mode",
                "--budget": "max chars (default 8000)",
                "--limit": "max extract items (default 50; Reddit post comments: all up to 1000 unless set)",
                "--logs": "console | network | xhr (all support --format json)",
                "--sort": "Reddit post auto-extract: confidence|top|new|controversial|old|qa",
                "--template": "JSON or @file — for extract=json",
                "--artifact": "read offloaded text chunks (concatenate data then parse); --items pages JSON rows; offset/limit count chars or rows",
                "--preview": "auto-extract container/schema without rows",
                "--changes": "model delta since last capture; --format json for structured fields",
                "--clear": "read then clear the selected log",
                "--format": "text (default) | json — pure JSON output for extract, artifact, changes, find, and logs reads"
            },
            "examples": [
                "webjitsu see model",
                "webjitsu see content example.com",
                "webjitsu see extract auto --limit 20",
                "webjitsu see extract auto --format json",
                "webjitsu see --find \"Submit\"",
                "webjitsu see --logs network"
            ],
            "notes": ["extract=auto is the first move on list/search/product/profile pages — one call returns structured items; on Reddit post pages it attempts the full comment tree; complete/partial disclose API denial and missing replies", "extract=auto envelope: container,count,items + page_total/partial/paged when the page claims more results than the DOM holds, sponsored (paid placements), duplicates_removed, redirects_unwrapped/redirects_unresolved — each explained by a note line"]
        }),
        "act" => json!({
            "usage": "webjitsu act <action> [target] [value] [--flags]",
            "tool": "act",
            "actions": crate::action::ACT_ACTIONS,
            "universal_flags": {
                "--ref": "element ref (self-heals; page-scoped — refresh after navigation; clear/read also take labels)",
                "--label": "field label",
                "--selector": "CSS selector (searches open shadow roots)",
                "--text": "value — text to type, file path (upload), wait match value",
                "--role": "role filter for text/label resolution",
                "--nth": "1-based pick among matches",
                "--key": "press key or chord (Control+a)",
                "--url": "navigate first (any action), or the URL for navigate/download/collect",
                "--option": "select option",
                "--condition": "wait condition",
                "--timeout": "seconds",
                "--dx/--dy": "scroll distance",
                "--js": "eval expression (@file or - for scripts)",
                "--submit": "fill: submit button ref or text",
                "--next": "paginate: CSS selector or text=Next (also label=/ref=)",
                "--extract": "paginate: read kind per page — auto (default) | links | forms | json (needs --template)",
                "--limit": "paginate: per-page extraction item limit",
                "--body": "download: HTTP body, <=32MiB/30s, no browser cookies; honors HTTP(S) BLADE_PROXY",
                "--sort": "Reddit post extract: confidence|top|new|controversial|old|qa",
                "--slim": "verdict-only successful actions; batch/run inherit, reads/errors retained",
                "--format": "read/extract/eval: text (default) | json — read returns live values/state/select choices; {data,note} for notices; {artifact,bytes,next_offset:0} when oversized"
            },
            "examples": [
                "webjitsu act click e5",
                "webjitsu act click \"Sign in\"",
                "webjitsu act type e12 \"hello\"",
                "webjitsu act fill '{\"e3\":\"John\"}' --submit e8",
                "webjitsu act click Submit --url example.com/login",
                "webjitsu act upload e5 /tmp/file.pdf",
                "webjitsu act batch @steps.json",
                "webjitsu act extract --format json",
                "webjitsu act wait settle",
                "webjitsu act dialog confirm --accept false",
                "webjitsu act eval \"document.title\""
            ],
            "notes": [
                "unquoted multi-word values join: act click Sign in == act click \"Sign in\"",
                "big JSON payloads: @file or - (stdin)",
                "eval result contract: `result: <compact JSON>` on ONE line — json.loads that line; a returned JSON string is flagged in a note; --format json returns the value itself",
                "batch steps may include {\"action\":\"see\"|\"extract\",...} to read inline; url= on any step navigates first"
            ]
        }),
        "state" => json!({
            "usage": "webjitsu state <op> [args] [flags]",
            "tool": "state",
            "ops": ["cookies","set-cookie","del-cookie","ls","ss","set-ls","set-ss","rm-ls","rm-ss","clear-ls","clear-ss","tabs","open-tab","close-tab","switch-tab","save","load","compress","block"],
            "flags": {
                "--url": "cookie scope / cookies filter / open-tab url",
                "--domain": "cookie domain",
                "--path": "cookie path",
                "--secure": "secure cookie",
                "--http-only": "httpOnly cookie",
                "--same-site": "Strict | Lax | None",
                "--target-id": "close-tab / switch-tab id",
                "--slim": "open-tab / switch-tab / close-tab: tab ID only, omit page model",
                "--clear": "block: stop blocking",
                "--mode": "compress: on | off | status"
            },
            "examples": [
                "webjitsu state cookies",
                "webjitsu state cookies example.com",
                "webjitsu state set-cookie token abc --domain example.com",
                "webjitsu state tabs",
                "webjitsu state save my-session",
                "webjitsu state block images,fonts",
                "webjitsu state compress off"
            ],
            "notes": ["save/load persists cookies+storage; navigate to the saved origin before load, then reload"]
        }),
        "run" => json!({
            "usage": "webjitsu run '<json-steps>' | @file | -",
            "tool": "run",
            "steps": "array; each {action:...} uses the same fields as act; plus {action:'if'|'while'|'see'}",
            "examples": [
                "webjitsu run '[{\"action\":\"navigate\",\"url\":\"example.com\"},{\"action\":\"see\",\"mode\":\"content\"}]'",
                "webjitsu run @steps.json",
                "cat steps.json | webjitsu run -"
            ],
            "notes": ["stops on first error with the step number + page state", "while+see reads across pages in ONE call", "--format json: status, results per top-level step, stopped_at/error/page/note on failure; nested observations remain output text; step formats are explicit; large results offload"]
        }),
        "vision" => json!({
            "usage": "webjitsu vision [--marks]",
            "tool": "vision",
            "flags": { "--marks": "numbered ref badges (Set-of-Marks)" },
            "examples": ["webjitsu vision", "webjitsu vision --marks --json | jq -r .image_path"],
            "notes": ["screenshot is saved to a file; path printed (json: image_path)", "last resort — see/act are cheaper and return actionable refs"]
        }),
        "daemon" => json!({
            "usage": "webjitsu daemon",
            "tool": null,
            "examples": [],
            "notes": ["auto-starts on the first command; run manually only to watch startup errors"]
        }),
        "stop" => json!({
            "usage": "webjitsu stop",
            "tool": null,
            "examples": [],
            "notes": ["graceful (flushes logins + knowledge); idempotent — exit 0 when nothing is running"]
        }),
        "help" => json!({
            "usage": "webjitsu help [command] [--json]",
            "tool": null,
            "examples": ["webjitsu help --json", "webjitsu help act"],
            "notes": ["the single self-teaching surface — fetch 'help --json' once"]
        }),
        "audio" => json!({
            "usage": "webjitsu audio [on|off|status] [--json]",
            "tool": null,
            "examples": [
                "webjitsu audio",
                "webjitsu audio on",
                "webjitsu audio off",
                "webjitsu audio status --json"
            ],
            "notes": [
                "browser audio is muted by default (`--mute-audio`) on both the isolated agent browser and owned real-browser launches so media/video pages never play unexpected sound",
                "opt in to audible playback with `webjitsu audio on` (persisted) or `BLADE_AUDIO=1` (per-process override); `webjitsu audio off` restores silent mode",
                "does not alter externally attached browsers (`rb mode attach` / `--port`), and does not affect Blink media APIs or WebAudio fingerprinting"
            ]
        }),
        // FORK (FORK.md): the real-browser lane is removed. This arm stays so
        // `help rb` explains the refusal instead of looking like a typo, and
        // `rb status` answers with `disabled_in_this_build: true`.
        "rb" | "realbrowser" => json!({
            "usage": "webjitsu rb status (reports disabled) — every other rb subcommand returns 'blocked:'",
            "tool": null,
            "removed_in_this_build": true,
            "examples": [
                "webjitsu rb status --json"
            ],
            "notes": [
                "REMOVED IN THIS FORK: the real-browser lane (drive your own browser, enumerate its profiles, clone its cookies/logins, or attach to a running one) does not exist in this build — see FORK.md",
                "the isolated agent browser under the blade data dir is the only lane; `state save|load` still persists ITS OWN state, which is all an agent needs",
                "a leftover realbrowser.json with `enabled: true`, or BLADE_LANE=real, cannot re-enable it: the lane decision and the profile import are both gated (src/fork.rs)"
            ]
        }),
        "hermes" => json!({
            "usage": "webjitsu hermes on|off|status [--profile <name>] [--hermes <path>] [--json]",
            "tool": null,
            "examples": ["webjitsu hermes on", "webjitsu hermes off", "webjitsu hermes status --json", "webjitsu hermes on --profile work"],
            "notes": ["Uses Hermes' public configuration CLI and the ordinary WebJitsu MCP server; no plugin or schema copies. Preserves other disabled toolsets and web search/extraction. off restores the prior MCP entry and browser policy, retaining unrelated settings. Start a new chat and restart gateways/desktop clients after switching. HERMES_HOME or --profile selects the Hermes profile. Chrome/Chromium is required; doctor checks prerequisites."]
        }),
        "mcp" => json!({
            "usage": "webjitsu mcp",
            "tool": null,
            "examples": ["webjitsu mcp"],
            "notes": [
                "MCP server on stdio — the integration surface for AI agents (Claude, Cursor, opencode, pi)",
                "client config: {\"command\":\"webjitsu\",\"args\":[\"mcp\"]}",
                "drives Chrome over WebSocket by default (navigator.webdriver stays false natively — zero page patches); BLADE_TRANSPORT=pipe opts into the zero-port pipe transport instead"
            ]
        }),
        "audit" => json!({
            "usage": "webjitsu audit",
            "tool": null,
            "examples": ["webjitsu audit"],
            "notes": ["stealth audit — local vector suite + boot self-check + cross-restart consistency stamp; every check must pass (the run prints its own total); any FAIL exits non-zero"]
        }),
        // FORK (`fork::SELF_UPDATE_DISABLED`, see FORK.md): self-update would
        // install upstream's unpatched binary and undo this fork, so the help
        // says so instead of advertising a working command.
        "update" => json!({
            "usage": "webjitsu update — REMOVED in this build (update by rebuilding from source)",
            "tool": null,
            "removed_in_this_build": true,
            "examples": [],
            "notes": [
                "self-update and --rollback are refused in this build: installing upstream's release over it would silently restore the real-browser lane this fork removes (FORK.md)",
                "to update: `git fetch upstream`, merge the new tag, keep the gates in src/fork.rs, then `cargo build --release`"
            ]
        }),
        "doctor" => json!({
            "usage": "webjitsu doctor | -doc",
            "tool": null,
            "examples": ["webjitsu doctor"],
            "notes": ["13 system checks: data root, Chrome, Xvfb, profile, logins, hygiene, locks, network, binary, disk, version"]
        }),
        "rollback" => json!({
            // FORK: refused — see the `update` arm above and FORK.md.
            "usage": "webjitsu --rollback — REMOVED in this build (a backup restore would put an unpatched binary back)",
            "tool": null,
            "removed_in_this_build": true,
            "examples": ["webjitsu --rollback"],
            "notes": ["restores the previous binary from <data dir>/backups (written by every self-update)"]
        }),
        "version" => json!({
            "usage": "webjitsu -v | --version",
            "tool": null,
            "examples": ["webjitsu -v"],
            "notes": ["version + install method + update status (behind / on latest / ahead — never a false 'up to date')"]
        }),
        _ => return None,
    };
    Some(v)
}

/// The MCP tool a command maps to (for schema lookup in help --json).
fn command_tool(cmd: &str) -> Option<&'static str> {
    match cmd {
        "nav" => Some("act"),
        "see" => Some("see"),
        "act" => Some("act"),
        "state" => Some("state"),
        "run" => Some("run"),
        "vision" => Some("vision"),
        _ => None,
    }
}

/// The machine manual — `webjitsu help --json` (optionally per command).
/// One call teaches an agent everything the CLI can do: the same depth as
/// MCP's tools + instructions, plus CLI-only details (exit codes, stdin/@file
/// payloads, universal flags, examples).
pub fn help_json(cmd: Option<&str>) -> Result<String> {
    let tools = crate::mcp::tools::tools_to_json();

    if let Some(name) = cmd {
        let norm = if name == "navigate" { "nav" } else { name };
        let detail = command_help_json(norm).ok_or_else(|| {
            BladeError::Usage(format!(
                "unknown command '{name}' — run 'webjitsu help' for the command list"
            ))
        })?;
        let mut out = json!({ "command": norm, "detail": detail });
        if let Some(t) = command_tool(norm) {
            if let Some(def) = tools
                .iter()
                .find(|v| v.get("name").and_then(|n| n.as_str()) == Some(t))
            {
                out["tool_definition"] = def.clone();
            }
        }
        return Ok(format!("{out}"));
    }

    let mut commands = serde_json::Map::new();
    // FORK (FORK.md): "rb" is dropped from the machine-readable command list so
    // an agent discovering the surface through `help --json` is never offered a
    // command this build refuses. `help rb` still documents the removal.
    for c in [
        "nav", "see", "act", "state", "run", "vision", "daemon", "stop", "help", "mcp", "audit",
        "update", "doctor", "rollback", "version", "hermes",
    ] {
        if let Some(d) = command_help_json(c) {
            commands.insert(c.to_string(), d);
        }
    }

    let out = json!({
        "webjitsu": { "version": env!("CARGO_PKG_VERSION"), "surface": "cli" },
        "instructions": INSTRUCTIONS,
        "quick_start": [
            "webjitsu nav example.com",
            "webjitsu see extract auto",
            "webjitsu act click \"Sign in\"",
            "webjitsu act fill '{\"e12\":\"John\"}' --submit e20",
            "webjitsu run '[{\"action\":\"navigate\",\"url\":\"x.com\"},{\"action\":\"see\",\"mode\":\"content\"}]'",
            "webjitsu stop"
        ],
        "commands": commands,
        "tools": tools,
        "output": {
            "human": "result text on stdout; diagnostics on stderr",
            "--json": "ONE JSON object on stdout: {\"ok\":bool,\"is_error\":bool,\"text\":string} (+ \"image_path\" for vision)",
            "payloads": {
                "@file": "read the argument from a file (steps, fields, templates, js)",
                "-": "read from stdin",
                "example": "webjitsu run @steps.json  |  cat steps.json | webjitsu run -"
            }
        },
        "exit_codes": {
            "0": "success",
            "1": "command ran but failed — details in text (includes page state)",
            "2": "usage error — bad command/flag/argument; the message says how to fix it"
        },
        "env": {
            "BLADE_HOME": "isolate daemon/Chrome/sessions/knowledge in a directory",
            "BLADE_CMD_TIMEOUT": "client wait for a daemon response, seconds (default 300)",
            "BLADE_IDLE_TIMEOUT": "daemon idle before Chrome shuts down, seconds (default 600)",
            "BLADE_NO_COMPRESS": "set 1 to disable response compression",
            "BLADE_LANE": "real|agent — force the real-browser lane for this process",
            "BLADE_RB_DEBUG": "real lane: set 1 to surface the browser's own stderr on launch failures",
            "BLADE_PLAIN": "plain CLI output — no ANSI even on a TTY (NO_COLOR also honored; CLICOLOR_FORCE=1 forces color off-TTY)",
            "BLADE_TRANSPORT": "mcp transport: WebSocket by default (native webdriver=false); set 'pipe' for the zero-port transport (its automation flag is masked)",
            "CHROME_PATH": "override the Chrome/Chromium binary",
            "RUST_LOG": "log filter (default warn,webjitsu=info)"
        }
    });
    Ok(format!("{out}"))
}

/// Detailed per-command human help (`help <cmd>`, `<cmd> --help`).
pub fn command_help_text(cmd: &str) -> Option<String> {
    let cmd = normalize_cmd(cmd);
    let text = match cmd {
        "nav" => {
            r#"webjitsu nav — navigate

USAGE
  webjitsu nav <url> [--block <classes>] [--json]

Returns the page title/URL, interactive element refs, and a content preview —
usually enough to act without a separate see call. Bare domains work:
`webjitsu nav example.com` → https://example.com. The first command
auto-starts the daemon (one Chrome shared by all later commands).

--block <classes>   block inert resources for this load AND remember the
                    choice for the domain: images,fonts,media,trackers

EXAMPLES
  webjitsu nav example.com
  webjitsu nav https://x.com --block images,fonts
  webjitsu nav example.com --json | jq .text
"#
        }
        "see" => {
            r#"webjitsu see — read the page without acting

USAGE
  webjitsu see [mode] [url] [extract <type>] [flags]

MODES
  model (default)   interactive elements with refs (e1, e2, …)
  content           page text as clean markdown (reading articles/docs)
  outline           heading hierarchy only (cheapest read)

EXTRACTION
  extract auto      structured items in ONE call — lists, search results,
                    products, posts. Site-aware: Reddit POST pages → the
                    full comment tree (every reply incl. collapsed, thread
                    order, complete flag); feeds/GitHub/products get fields.
                    Envelope: {container,count,items,…} + page_total/partial/paged
                    when the page claims more results than the DOM holds,
                    sponsored (paid placements), duplicates_removed,
                    redirects_unwrapped — each with a `note:` line.
  extract links     all links
  extract forms     all forms with fields
  extract json      custom template (--template '{"items":{…}}' or @file)

FLAGS
  --filter <role>     only elements of a role (button, link, textbox, …)
  --find <text>       search by text → refs
  --scope <ref>       read one element's subtree
  --content           include text in model mode
  --budget <N>        max response chars (default 8000)
  --limit <N>         max extract items (default 50; Reddit comments: all ≤1000)
  --logs console|network|xhr
  --artifact <path>   paged read (--offset/--limit count chars; --items counts JSON rows)
  --preview           extract auto: container/schema without rows
  --changes           model delta since last capture; --format json for structured fields
  --clear             read then clear --logs console|network|xhr
  --sort <sort>       Reddit post auto-extract: confidence|top|new|controversial|old|qa
  --format text|json  json: pure JSON out for extract, artifact, changes, find, and logs reads
  --block <classes>   with url: skip images,fonts,media,trackers during load
  --url <url>         navigate first (or pass the URL positionally)

EXAMPLES
  webjitsu see                            interactive elements
  webjitsu see content                    page as markdown
  webjitsu see extract auto --limit 20    structured items
  webjitsu see extract auto --format json  pure JSON output
  webjitsu see example.com content        navigate + read in one call
  webjitsu see --find "Submit"            refs by text
  webjitsu see --logs network             recent requests
"#
        }
        "act" => {
            r#"webjitsu act — interact with the page

USAGE
  webjitsu act <action> [target] [value] [--flags]

ACTIONS
  click <ref|label>        type <target> <text>      fill <fields> [--submit]
  select <target> <option> clear <ref|label>         press <key|chord>
  scroll <dx> <dy>         hover <target>            navigate <url> [--block]
  upload <target> <path>   download <url> [--path]  wait <condition> [value]
  eval <js> [--ref]        collect <url> [--max N]   read <ref|label>
  paginate <next> [--max N] [--extract auto|links|forms|json] [--limit N]
                           bounded read/advance, retains pages
  download <url> --body   save an HTTP body without browser cookies (HTTP(S) proxy honored)
  extract [auto|links|forms|json]   structured data (same as 'see extract …')
  dialog <alert|confirm|prompt|beforeunload|any|clear> [--accept true|false]
         [--prompt-text <t>] [--message <m>]   arm the NEXT dialog's answer (one use)
  batch <steps|@file|->    pdf [--path] [--landscape]
  back / forward / reload  save <name> / load <name>
  open-tab [url] / switch-tab <id> / close-tab <id>

UNIVERSAL FLAGS (mirror the MCP schema — accepted on every action)
  --ref --label --text --selector --role --nth --key --url --option --condition --timeout
  --next --body --sort --dx --dy --js --submit --block --slim --format  (per-action: --path, --max, --extract, --limit, --x/--y…)

Multi-word values don't need quotes: 'act click Sign in' works. Big JSON
payloads: @file or - (stdin). Unknown flags fail loudly (exit 2). Key chords: 'act press Control+a'.

RESULT SHAPES (stable — parse them without guessing)
  read          live text/control value, checked/indeterminate and ARIA state,
                select choices (first 80; options_total/options_truncated disclose bounds).
                --format json returns structured properties; large reads offload.
  eval          one line: `result: <compact JSON of the returned value>` — read
                that line, json.loads it. A returned string that CONTAINS JSON is
                called out in a `note:` line (parse the value once more — do not
                discard it); a DOM node/Window says what to return instead;
                undefined reads `result: undefined`. `--format json` returns the
                bare value, `{"data":…,"note":…}` when a note rides along, or an
                `{"artifact":…,"bytes":…}` envelope when oversized.
  extract auto  envelope {container,count,items,…} plus, when they apply:
                page_total/partial/paged (the page claims more results than the
                DOM holds), sponsored (paid placements — flagged per item, left in
                page order), duplicates_removed, redirects_unwrapped/redirects_unresolved. Each has a
                `note:` line explaining what to do about it.

EXAMPLES
  webjitsu act click e5                          click a ref
  webjitsu act click "Sign in"                   click by text
  webjitsu act click --selector "[role=menuitem]"   click by CSS (searches shadow DOM)
  webjitsu act type e12 "hello world"            type
  webjitsu act fill '{"e3":"John","e5":"Doe"}' --submit e8
  webjitsu act click Submit --url example.com/login   navigate + click in ONE call
  webjitsu act upload e5 /tmp/file.pdf           file upload
  webjitsu act wait settle                       wait for DOM quiet
  webjitsu act wait element --text "Results"     wait for an element
  webjitsu act eval "document.title"             evaluate JS
  webjitsu act batch '[{"action":"reload"}]'     sequential steps in ONE call
  webjitsu act collect https://x.com/list --max 100   infinite-scroll collect
  webjitsu act extract --format json             structured data as JSON
"#
        }
        "state" => {
            r#"webjitsu state — cookies, storage, tabs, sessions, blocking

USAGE
  webjitsu state <op> [args] [flags]

COOKIES
  cookies [url]                        list (filtered to url / current page)
  set-cookie <name> <value> [flags]    flags: --url --domain --path --secure
                                       --http-only --same-site Strict|Lax|None
  del-cookie <name> [--url|--domain]

STORAGE
  ls / ss                              list localStorage / sessionStorage
  set-ls <key> <value> · set-ss <key> <value>
  rm-ls <key> · rm-ss <key> · clear-ls · clear-ss

TABS
  tabs                                 list (* = current)
  open-tab <url>                       open + auto-switch
  switch-tab <id> · close-tab <id>     ids come from 'tabs'

SESSIONS / CONTROL
  save <name> · load <name>            persist/restore cookies+storage
  block [classes] · block clear        image/font/media/tracker blocking
  compress on|off|status               context pruning

EXAMPLES
  webjitsu state cookies
  webjitsu state set-cookie token abc --domain example.com --secure
  webjitsu state tabs
  webjitsu state save my-session
"#
        }
        "run" => {
            r#"webjitsu run — batch actions with branching and loops

USAGE
  webjitsu run '<json-steps>' [--slim] [--format text|json] (or @file / -)

Steps are action objects (same fields as act) plus:
  {"action":"if","condition":…,"then":[…],"else":[…]}   branch
  {"action":"while","condition":…,"steps":[…],"max":N}  loop
  {"action":"see",…}        read inline — while+see reads across pages in ONE call
  state ops (open-tab, save, load, …) work as steps too

Stops on the first error and returns the step number + page state.
Conditions: element, title, url, text, settle, js.
--format json returns status=completed|stopped and results with 1-based top-level
step IDs, status=ok|optional_failed|failed and output text. Stopped runs include
stopped_at/error/page/note. Nested observations stay in output; set each read's
format explicitly. Oversized results use the usual artifact paging envelope.

EXAMPLES
  webjitsu run '[{"action":"navigate","url":"example.com"},{"action":"see","mode":"content"}]'
  webjitsu run @steps.json
  cat steps.json | webjitsu run -
"#
        }
        "vision" => {
            r#"webjitsu vision — screenshot

USAGE
  webjitsu vision [--marks]

Always saves a PNG to a file and prints the path (--json: "image_path").
--marks overlays numbered ref badges (Set-of-Marks) so you can click by ref
after looking at the image. Vision is the LAST RESORT: the structural model
(see/act with refs) is cheaper and gives actionable refs.

EXAMPLES
  webjitsu vision
  webjitsu vision --marks --json | jq -r .image_path
"#
        }
        "daemon" => {
            r#"webjitsu daemon — persistent Chrome session

USAGE
  webjitsu daemon

Auto-starts on the first command — run it manually only to watch startup
errors. One Chrome instance serves every later command (Unix socket under the
data dir; socket + pid file are 0600). Idle timeout: BLADE_IDLE_TIMEOUT
seconds (default 600), then Chrome shuts down; the next command relaunches it.
"#
        }
        "audio" => {
            r#"webjitsu audio — browser sound output (muted by default)

USAGE
  webjitsu audio                  show current audio state (same as `audio status`)
  webjitsu audio on               opt in to audible browser playback
  webjitsu audio off              mute browser audio again (default)
  webjitsu audio status [--json]  show effective audio state + env override

Browser sound is muted by default (`--mute-audio`) on both the isolated agent
browser and owned real-browser launches so YouTube, video ads, and media
players never play unexpected sound while an agent browses.

Opt in persistently with `webjitsu audio on`, or for one run with
`BLADE_AUDIO=1` (`BLADE_AUDIO=0` forces muted). The CLI daemon restarts
immediately and running MCP sessions pick up the change on their next call.
Externally attached browsers (`rb mode attach` / `--port`) are untouched.
"#
        }
        "rb" | "realbrowser" => {
            r#"webjitsu rb — REMOVED in this build

USAGE
  webjitsu rb status              reports the disabled state (exit 0)
  webjitsu rb <anything else>     blocked: (exit non-zero)

This fork of WebJitsu deletes the real-browser lane: the agent never
drives your own browser, never enumerates your Chromium profiles, and
never copies the cookies, logins and storage out of one. The isolated
agent browser under the blade data dir is the only lane, so there is
nothing to switch, select, refresh, pause or forget.

WHY
  A real profile is the union of every session you are logged into.
  Handing it to an agent that reads untrusted pages puts page content in
  a position to act as you, and a copied profile sits on disk as plain
  session tokens. `webjitsu state save|load` still persists the agent
  browser's own state, which is all an agent needs.

  FORK.md records the gates and how to re-enable a removed feature.
"#
        }
        "stop" => {
            r#"webjitsu stop — shut the daemon down

USAGE
  webjitsu stop

Graceful: flushes logins + domain knowledge, kills Chrome, removes the
socket. Idempotent — exit 0 even when nothing is running. The daemon runs
one command at a time: if it does not answer within 10s (busy with a long
command, or wedged), stop sends SIGTERM to the pid from the pid file — the
running command is cancelled (outcome unknown) and Chrome still closes
gracefully; SIGKILL follows if it is still alive 10s later.
"#
        }
        "help" => {
            r#"webjitsu help — the single self-teaching surface

USAGE
  webjitsu help              this manual
  webjitsu help <command>    one command in depth
  webjitsu help --json       the machine manual: tool schemas (same as MCP
                             tools/list), per-command usage, exit codes,
                             payload conventions, examples — call ONCE and
                             you know the whole CLI
  webjitsu help <cmd> --json per-command machine help
  webjitsu <cmd> --help      same as help <cmd> (never runs the command)
"#
        }
        "hermes" => {
            r#"webjitsu hermes — reversible Hermes browser setup

USAGE
  webjitsu hermes on             connect MCP, then disable Hermes' built-in browser tools
  webjitsu hermes off            restore the previous browser policy and MCP entry
  webjitsu hermes status         inspect the managed setup
  --profile <name>               select a Hermes profile (forwarded as hermes -p)
  --hermes <path>                select the Hermes CLI executable
  --json                        one machine-readable result

Uses the ordinary MCP server and Hermes' public configuration commands.
Web search/extraction and unrelated settings are preserved. Configuration
recovery is stored privately beside Hermes' config.yaml in .webjitsu-browser/.
An interrupted switch can resume with on or restore with off. Changed MCP
entries are never overwritten during recovery. Use the same profile for off.
Start a new chat; restart running gateways/desktop clients after switching.
Chrome/Chromium must be installed; webjitsu doctor checks prerequisites.
Updates use your normal WebJitsu updater; no separate Hermes plugin to update.
"#
        }
        "mcp" => {
            r#"webjitsu mcp — MCP server (stdio JSON-RPC)

USAGE
  webjitsu mcp

The integration surface for AI agents. Client config:

  {"mcpServers": {"webjitsu": {"command": "webjitsu", "args": ["mcp"]}}}

Speaks MCP 2024-11-05 through 2026-07-28 (legacy initialize handshake +
the 2026-07-28 stateless dialect). Drives Chrome over WebSocket by default
(navigator.webdriver stays false natively — zero page patches);
BLADE_TRANSPORT=pipe opts into the zero-port pipe transport instead (its
automation flag is masked — lie-engine-style detectors can see the mask).
Chrome launches lazily on the first tool call.
"#
        }
        "audit" => {
            r#"webjitsu audit — stealth audit

USAGE
  webjitsu audit

Runs the local vector suite (tests/vectors.html: the core stealth
surface, the permissions native-parity battery, GL-coherence checks,
worker/iframe propagation) plus a boot self-check and a cross-restart
consistency stamp (canvas/audio/geometry/UA/GL must not drift between
runs). Run it after any stealth-affecting change; every check must pass —
the run prints its own total (it depends on platform and profile), and
any FAIL line exits non-zero. The suite is built into the binary; a
tests/vectors.html in the current directory overrides it.
"#
        }
        "update" => {
            r#"webjitsu -u / update — REMOVED in this build

USAGE
  webjitsu -u                     blocked: (exit non-zero)

Self-update is refused here. Downloading upstream's release and swapping
it in would replace this build with one that has the real-browser lane
back, which is the one thing this fork exists to prevent — so updating
is a source operation instead:

  git fetch upstream && git merge upstream/<tag>
  # keep the gates in src/fork.rs, then
  cargo build --release

FORK.md records every gate and how to re-enable a removed feature.
"#
        }
        "doctor" => {
            r#"webjitsu doctor / -doc — system diagnostics

USAGE
  webjitsu doctor

13 checks: data directory, Chrome (+version), Xvfb, profile dir, login
persistence, profile hygiene, stale locks, GitHub reachability, binary
integrity, disk space, version-vs-latest. Failures print a fix.
"#
        }
        "rollback" => {
            r#"webjitsu --rollback — REMOVED in this build

USAGE
  webjitsu --rollback             blocked: (exit non-zero)

Restoring a backup would put an older — and possibly unpatched — binary
back in place, so it is refused with the same gate as `update`. To go
back: check out the tag you want in this fork's repository and rebuild.
"#
        }
        "version" => {
            r#"webjitsu -v / --version — version + update status

USAGE
  webjitsu -v

Version + build id, install method (npm / source / binary) and update
state: behind (with the hint), on the latest release, or ahead of it
(dev builds). Never a misleading "up to date" when the check failed.
"#
        }
        _ => return None,
    };
    Some(crate::ui::style_help(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_json_is_complete_and_parses() {
        let raw = help_json(None).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert!(v["instructions"].as_str().unwrap().len() > 200);
        assert_eq!(v["tools"].as_array().unwrap().len(), 5);
        assert!(v["commands"]["act"]["usage"].as_str().is_some());
        assert!(v["commands"]["nav"].is_object());
        assert!(
            v["commands"]["mcp"].is_object(),
            "help --json must cover mcp"
        );
        // FORK (FORK.md): `rb` is deliberately ABSENT from the machine-readable
        // command list — an agent that discovers the surface through
        // `help --json` must not be offered a command this build refuses. The
        // on-demand `help rb` still explains the removal.
        assert!(
            v["commands"]["rb"].is_null(),
            "help --json must not advertise the removed rb command"
        );
        // `help <cmd> --json` nests the command detail under "detail".
        let rb: Value = serde_json::from_str(&help_json(Some("rb")).unwrap()).unwrap();
        assert!(
            rb["detail"]["removed_in_this_build"]
                .as_bool()
                .unwrap_or(false),
            "`help rb` must say the command is removed: {rb}"
        );
        assert!(v["commands"]["doctor"].is_object());
        assert!(v["exit_codes"]["2"].as_str().unwrap().contains("usage"));
        assert!(v["output"]["payloads"]["@file"].as_str().is_some());

        let raw = help_json(Some("act")).unwrap();
        let v: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(v["command"], "act");
        assert_eq!(v["tool_definition"]["name"], "act");
        assert!(help_json(Some("nope")).is_err());
    }

    #[test]
    fn every_command_has_help_text_and_json() {
        for c in [
            "nav", "see", "act", "state", "run", "vision", "daemon", "stop", "help", "rb", "mcp",
            "audit", "update", "doctor", "rollback", "version",
        ] {
            let text = command_help_text(c).unwrap_or_else(|| panic!("{c} human help"));
            assert!(text.contains("USAGE"), "{c} help lacks USAGE");
            let j = command_help_json(c).unwrap_or_else(|| panic!("{c} json help"));
            assert!(j["usage"].as_str().is_some(), "{c} json lacks usage");
        }
        // Aliases resolve to the canonical command (v3.9.10: `help mcp` etc. work).
        assert!(command_help_text("-u").is_some() && command_help_text("-doc").is_some());
        assert!(
            command_help_text("--version").is_some() && command_help_text("--rollback").is_some()
        );
        assert!(command_help_json("-u").is_some() && command_help_json("-v").is_some());
        assert!(command_help_text("bogus").is_none());
        assert!(command_help_json("bogus").is_none());
    }

    #[test]
    fn suggestions_map_typos_and_synonyms() {
        assert_eq!(suggest_command("nva"), Some("nav"));
        assert_eq!(suggest_command("stte"), Some("state"));
        assert_eq!(suggest_command("visio"), Some("vision"));
        // FORK (FORK.md): the rb vocabulary is out of the suggestion space —
        // suggesting `rb use` would send an agent to a command this build
        // refuses, so typos and the words users reach for now suggest nothing.
        assert_eq!(suggest_command("realbrwoser"), None);
        assert_eq!(suggest_command("browser"), None);
        assert_eq!(suggest_command("browsers"), None);
        assert_eq!(suggest_command("completely-unrelated"), None);
    }

    /// The audit total is platform/profile-dependent and printed by the run
    /// itself (audit.rs); a number pinned in help drifted (help said 61
    /// while the suite had grown to 111). No help surface may pin one.
    #[test]
    fn audit_help_does_not_pin_a_check_count() {
        fn pins_a_count(text: &str) -> bool {
            let b = text.as_bytes();
            (0..b.len()).any(|i| {
                b[i].is_ascii_digit() && (i == 0 || !b[i - 1].is_ascii_digit()) && {
                    let end = (i..b.len())
                        .find(|&j| !b[j].is_ascii_digit())
                        .unwrap_or(b.len());
                    let tail = &text[end..];
                    tail.starts_with("-check")
                        || tail.starts_with(" checks")
                        || tail.starts_with(" vectors")
                        || (tail.starts_with('/')
                            && tail[1..].starts_with(|c: char| c.is_ascii_digit()))
                }
            })
        }
        assert!(pins_a_count("a 61-check suite") && pins_a_count("61/61 is the bar"));
        assert!(!pins_a_count("vector suite + boot self-check"));
        let manual = help_text();
        let audit_line = manual
            .lines()
            .find(|l| l.trim_start().starts_with("audit "))
            .expect("manual lists audit");
        for text in [
            audit_line.to_string(),
            command_help_text("audit").unwrap(),
            command_help_json("audit").unwrap().to_string(),
        ] {
            assert!(!pins_a_count(&text), "audit help pins a count: {text}");
        }
    }

    #[test]
    fn help_text_mentions_version_and_contract() {
        let t = help_text();
        assert!(t.contains(env!("CARGO_PKG_VERSION")));
        assert!(t.contains("EXIT CODES"));
        assert!(t.contains("help --json"));
    }
}
