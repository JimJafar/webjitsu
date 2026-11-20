/**
 * WebJitsu pi extension — the WebJitsu MCP server, registered with pi.
 *
 * This is the whole extension. It resolves the WebJitsu binary and hands it to
 * pi as a stdio MCP server; pi's own MCP client does everything else — spawn,
 * handshake, tool-call proxying, shutdown. The five tools arrive with the
 * binary's full definitions (descriptions, schemas and the server's
 * `initialize` instructions), so no meaning is copied here and there is nothing
 * that can drift when the Rust side changes.
 *
 * `exposure: "direct"` declares the tools to the model like any other pi tool,
 * and the session waits for this server before the first prompt. The timeout
 * covers the driver's long operations (run/batch/collect), not pi's 60s default.
 *
 * FORK DIFFERENCE (FORK.md): upstream resolved the binary ONLY through the
 * optional platform packages (`webjitsu-linux-x64`, …), which is right for an
 * npm-installed package and wrong here. This fork is source-only and installs
 * with `pi install ./npm/webjitsu`, and pi loads a local path in place without
 * installing its dependencies — so the platform packages are never present and
 * the binary has to be found by convention. `resolveBinary` therefore searches,
 * in order: `WEBJITSU_BIN`, a build in the source tree this package lives in,
 * `~/.local/bin/webjitsu` (what `scripts/fork-install.sh` writes), and finally
 * the upstream platform-package route.
 */
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { accessSync, constants, existsSync, statSync } from "node:fs";
import { createRequire } from "node:module";
import { homedir } from "node:os";
import { dirname, isAbsolute, join } from "node:path";

/** npm platform packages → the binary each one carries. */
const PLATFORM_MAP: Record<string, string> = {
  "linux-x64": "webjitsu-linux-x64",
  "linux-arm64": "webjitsu-linux-arm64",
  "darwin-x64": "webjitsu-darwin-x64",
  "darwin-arm64": "webjitsu-darwin-arm64",
  "win32-x64": "webjitsu-windows-x64",
};

const EXE = process.platform === "win32" ? "webjitsu.exe" : "webjitsu";

/** Resolve a module loader that works under both jiti CJS and native Node/Bun ESM. */
function getRequire(): NodeRequire {
  if (typeof require === "function") return require;
  return createRequire(import.meta.url);
}

/** This file's directory, under both jiti CJS (`__dirname`) and native ESM. */
function packageDir(): string | null {
  if (typeof __dirname === "string" && __dirname) return __dirname;
  try {
    return dirname(new URL(import.meta.url).pathname);
  } catch {
    return null;
  }
}

/** True when `path` is a regular file this process can execute. */
function executable(path: string): boolean {
  try {
    if (!existsSync(path) || !statSync(path).isFile()) return false;
    accessSync(path, process.platform === "win32" ? constants.F_OK : constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

/** The binary carried by this platform's optional dependency (upstream's route). */
function fromPlatformPackage(): string | null {
  const pkgName = PLATFORM_MAP[`${process.platform}-${process.arch}`];
  if (!pkgName) return null;
  try {
    const req = getRequire();
    const pkgJsonPath = req.resolve(`${pkgName}/package.json`);
    const pkgJson = req(pkgJsonPath);
    const binPath = join(dirname(pkgJsonPath), pkgJson.main || EXE);
    return executable(binPath) ? binPath : null;
  } catch {
    return null;
  }
}

/** Absolute path of the WebJitsu executable, or null when none is installed. */
function resolveBinary(): string | null {
  // 1. An explicit override always wins: it is how an operator points the
  //    extension at a specific build (a worktree, a debug build, another user).
  const fromEnv = process.env.WEBJITSU_BIN;
  if (fromEnv && fromEnv.trim()) {
    const path = fromEnv.trim();
    if (isAbsolute(path) && executable(path)) return path;
    return null; // A wrong override must not silently fall through to another build.
  }

  // 2. A build sitting in the source tree this package was installed from
  //    (`pi install ./npm/webjitsu` loads it in place).
  const dir = packageDir();
  if (dir) {
    for (const profile of ["release", "gate"]) {
      const candidate = join(dir, "..", "..", "target", profile, EXE);
      if (executable(candidate)) return candidate;
    }
  }

  // 3. The documented install location for this fork.
  const installed = join(homedir(), ".local", "bin", EXE);
  if (executable(installed)) return installed;

  // 4. Upstream's route: the platform package, when one was installed.
  return fromPlatformPackage();
}

export default function webjitsuExtension(pi: ExtensionAPI) {
  const key = `${process.platform}-${process.arch}`;
  const binary = resolveBinary();

  if (!binary || typeof pi.registerMcpServer !== "function") {
    const why = !binary
      ? `no WebJitsu binary found for ${key}. Build it (cargo build --release), or run ` +
        `scripts/fork-install.sh, or set WEBJITSU_BIN to the binary's absolute path.` +
        (process.env.WEBJITSU_BIN ? ` WEBJITSU_BIN=${process.env.WEBJITSU_BIN} is not an executable.` : "")
      : "this pi build has no MCP registration API — update pi";
    pi.on("session_start", (_event, ctx) => ctx.ui.notify(`WebJitsu: ${why}`, "error"));
    return;
  }

  pi.registerMcpServer("webjitsu", {
    description: "Stealthy agentic browser driver — act, see, state, run, vision.",
    command: binary,
    args: ["mcp"],
    exposure: "direct",
    timeout: 180,
  });
}
