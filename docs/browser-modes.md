# Browser modes

[WebJitsu README](../README.md) · [Configuration](configuration.md)

WebJitsu drives one browser: a Chromium-family instance it owns, on a dedicated
profile. Upstream's second mode — driving *your* browser — is removed in this
fork; see [Your browser](#your-browser-removed-in-this-fork) and
[FORK.md](../FORK.md).

## Agent browser

The browser is owned by WebJitsu and uses a dedicated profile. Browser state can
persist across orderly shutdowns. To isolate a task from your ordinary WebJitsu
data, set a separate `BLADE_HOME` before launching the CLI or MCP server.
`BLADE_FRESH=1` requests an ephemeral profile. Incognito compatibility is a
separate mode with different storage behavior.
[Storage options](configuration.md#data-and-profiles).

Recognized Chromium-family installs include Chromium, Chrome (Stable/Beta/Canary),
Ungoogled Chromium, Brave, Helium, Arc, Edge, Vivaldi, Opera, Opera GX, and
Thorium — including PATH installs and Linux Flatpak exports. For a portable
binary or AppImage, launch it once to create a profile, then point WebJitsu at
the binary. Recognition and runtime verification differ by engine; see the
[support matrix](../SUPPORT-MATRIX.md).

Owned launches mute audio by default so media players never produce unexpected
sound; opt in with `webjitsu audio on` (or `BLADE_AUDIO=1`).

## Your browser: removed in this fork

Upstream's `bladebro rb` lane could enumerate your installed browsers and their
profiles, import a copy of one (cookies, logins, storage), launch your live
profile, or attach to a browser you had already started — and it refreshed that
copy whenever your real browser wrote newer state.

That means the agent acts in every session you are logged into, steered partly
by the pages it reads, on your display, with your identity. Here that lane is
deleted:

```sh
webjitsu rb status          # → reports the lane disabled, exit 0
webjitsu rb use             # → blocked: … (exit non-zero)
webjitsu rb on              # → blocked: …
```

Nothing else changes shape: the same five tools, the same refs and verdicts,
the same CLI and MCP interface — on the agent browser only. `webjitsu state
save|load` still persists sessions, and those are the agent browser's own
sessions, which is all an automation task needs.

Why the fork exists: [FORK.md](../FORK.md#why-the-lane-is-gone). What survives
upstream of that decision (including `--host`/`--port`, which can still attach
to a browser you armed with a debug endpoint):
[FORK.md#known-gaps](../FORK.md#known-gaps--what-this-fork-does-not-protect-you-from).

To pause an agent mid-task on this fork, stop the MCP client or run
`webjitsu stop` — the daemon closes the browser it owns.
