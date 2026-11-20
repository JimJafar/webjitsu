'use strict';

const { accessSync, chmodSync, constants, statSync } = require('fs');
const { homedir } = require('os');
const { dirname, isAbsolute, join } = require('path');

const PLATFORMS = {
  'linux-x64': 'webjitsu-linux-x64',
  'linux-arm64': 'webjitsu-linux-arm64',
  'darwin-x64': 'webjitsu-darwin-x64',
  'darwin-arm64': 'webjitsu-darwin-arm64',
  'win32-x64': 'webjitsu-windows-x64',
};

const EXE = process.platform === 'win32' ? 'webjitsu.exe' : 'webjitsu';

function isExecutable(path) {
  try {
    if (!statSync(path).isFile()) return false;
    accessSync(path, process.platform === 'win32' ? constants.F_OK : constants.X_OK);
    return true;
  } catch {
    return false;
  }
}

// FORK (FORK.md): upstream resolved the binary ONLY through the optional
// platform packages, which is right for an npm-installed package and wrong for
// a source-only fork that pi loads in place (`pi install ./npm/webjitsu` does
// not install dependencies). Both this launcher and pi-extension.ts therefore
// search the SAME order, so the CLI launcher, the DSH plugin and pi can never
// disagree about which build is running:
//
//   WEBJITSU_BIN → a build in this source tree → ~/.local/bin/webjitsu →
//   this platform's optional package
//
// Never through a shell, and never a desktop app's PATH.
function search() {
  const fromEnv = (process.env.WEBJITSU_BIN || '').trim();
  if (fromEnv) {
    // An explicit override wins or fails: falling through to another build
    // would run a binary the operator did not choose.
    if (!isAbsolute(fromEnv) || fromEnv.includes('\0')) {
      throw new Error('webjitsu: WEBJITSU_BIN must be an absolute executable path');
    }
    return fromEnv;
  }

  for (const profile of ['release', 'gate']) {
    const candidate = join(__dirname, '..', '..', 'target', profile, EXE);
    if (isExecutable(candidate)) return candidate;
  }

  const installed = join(homedir(), '.local', 'bin', EXE);
  if (isExecutable(installed)) return installed;

  const pkg = PLATFORMS[`${process.platform}-${process.arch}`];
  if (pkg) {
    try {
      const candidate = join(dirname(require.resolve(`${pkg}/package.json`)), EXE);
      if (isExecutable(candidate)) return candidate;
    } catch {
      /* platform package not installed — fall through to the error below */
    }
  }

  throw new Error(
    `webjitsu: no binary found for ${process.platform}-${process.arch}. Build it ` +
      `(cargo build --release), run scripts/fork-install.sh, or set WEBJITSU_BIN to the ` +
      "binary's absolute path. See FORK.md.",
  );
}

/**
 * Resolve the executable to spawn. `override` is an absolute path from
 * configuration (the DSH plugin's `binaryPath`); when absent, `search()` decides.
 */
function resolveBinary(override) {
  if (
    override !== undefined &&
    (typeof override !== 'string' || !isAbsolute(override) || override.includes('\0'))
  ) {
    throw new Error('webjitsu: binaryPath must be an absolute executable path');
  }
  const path = override || search();

  try {
    if (!statSync(path).isFile()) throw new Error('not a file');
    const mode = process.platform === 'win32' ? constants.F_OK : constants.X_OK;
    try {
      accessSync(path, mode);
    } catch (accessErr) {
      // Some tarball extractors / WSL setups strip the executable bit from
      // optional-dependency binaries. Attempt a safe in-place 0755 repair on
      // Unix before failing.
      if (process.platform !== 'win32' && accessErr && accessErr.code === 'EACCES') {
        try {
          chmodSync(path, 0o755);
          accessSync(path, mode);
        } catch {
          throw accessErr;
        }
      } else {
        throw accessErr;
      }
    }
  } catch (error) {
    throw new Error(`webjitsu: cannot execute ${path}: ${error.message}. See FORK.md to build it.`);
  }
  return path;
}

module.exports = { resolveBinary };
