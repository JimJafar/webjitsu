//! Fork-local safety gates.
//!
//! Everything this build removes hangs off the constants here, so
//! `grep -rn "fork::" src` lists the whole behavioural diff in one pass, and
//! `git merge upstream` never has to guess what the fork meant. See `FORK.md`
//! for why each gate exists and what re-enabling it costs.
//!
//! Each guard is written as `if crate::fork::<CONST> { return … }` rather
//! than an unconditional `return`, so the upstream code below stays
//! reachable to the compiler (no `unreachable_code`/`dead_code` warnings
//! under the project's `clippy -D warnings` gate) and a one-word flip
//! restores the feature.

use crate::error::BladeError;

/// The real-browser lane is removed in this build: `bladebro rb` refuses,
/// the lane can never be selected (config or `BLADE_LANE=real`), and copying
/// a user profile cannot happen.
pub const REAL_BROWSER_DISABLED: bool = true;

/// Self-update and rollback are removed in this build. Without this gate
/// `bladebro update` would download upstream's unpatched binary and swap it
/// in, silently restoring the real-browser lane this fork exists to remove.
pub const SELF_UPDATE_DISABLED: bool = true;

/// Suffix identifying this build, so `bladebro -v` and any bug report show a
/// patched binary rather than an upstream one.
pub const FORK_TAG: &str = "no-rb";

/// The refusal a gated surface returns. `blocked:` is the word agents already
/// branch on for a refused action, and naming the fork stops the reader
/// hunting for a config key that does not exist.
pub fn blocked(what: &str) -> BladeError {
    BladeError::Other(format!(
        "blocked: {what} is removed in this bladebro build (fork `{FORK_TAG}`, see FORK.md)."
    ))
}

/// True when this process must not contact the release registry.
///
/// Upstream tags describe upstream's binary, which this build deliberately
/// diverges from, so a version check would only nag about an update that must
/// never be installed. Honours upstream's `BLADE_NO_UPDATE_CHECK=1` too, so
/// the existing "skipped" reporting paths stay honest.
pub fn update_check_disabled() -> bool {
    SELF_UPDATE_DISABLED
        || std::env::var("BLADE_NO_UPDATE_CHECK")
            .map(|v| v == "1")
            .unwrap_or(false)
}
