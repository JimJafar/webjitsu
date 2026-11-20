//! Fork contract: the real-browser lane and self-update are removed here.
//!
//! These are the guarantees this fork exists to provide, so they are pinned at
//! the process boundary (a subprocess with an isolated `BLADE_HOME`) and at the
//! library boundary — the places a future `git merge upstream` would notice if
//! an upstream change quietly reopened a gate. See `FORK.md`.
//!
//! None of these paths may launch Chrome: `CHROME_PATH` points at a missing
//! binary so a regression fails fast instead of starting a browser.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::Value;

fn home(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("bladebro-fork-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn command(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bladebro"));
    cmd.env("BLADE_HOME", home)
        // Upstream's env override for the real lane: it must lose here, so it is
        // set for every case rather than removed.
        .env("BLADE_LANE", "real")
        .env("BLADE_NO_WARMING", "1")
        .env("BLADE_IDLE_TIMEOUT", "5")
        .env("BLADE_PLAIN", "1")
        .env("CHROME_PATH", home.join("no-chrome-here"))
        .stdin(Stdio::null());
    cmd
}

fn bladebro(home: &Path, args: &[&str]) -> Output {
    command(home).args(args).output().unwrap()
}

fn both(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn json(out: &Output) -> Value {
    serde_json::from_str(&String::from_utf8_lossy(&out.stdout))
        .unwrap_or_else(|e| panic!("not JSON: {e}\n{}", both(out)))
}

/// Every way upstream offered to put the agent in a real browser, in a real
/// profile, or onto a running browser — and the self-update paths that would
/// have replaced this binary with upstream's.
#[test]
fn real_lane_and_self_update_are_refused() {
    let h = home("refused");
    for args in [
        vec!["rb", "on"],
        vec!["rb", "true"],
        vec!["rb", "enable"],
        vec!["rb", "use"],
        vec!["rb", "use", "chrome"],
        vec!["rb", "use", "--binary", "/usr/bin/chromium"],
        vec!["rb", "mode", "clone"],
        vec!["rb", "mode", "profile"],
        vec!["rb", "mode", "attach"],
        vec!["rb", "profile", "Default"],
        vec!["rb", "refresh"],
        vec!["rb", "forget"],
        vec!["rb", "pause"],
        vec!["rb", "resume"],
        vec!["realbrowser", "use"],
        vec!["update"],
        vec!["update", "--check"],
        vec!["-u"],
        vec!["--rollback"],
        vec!["rollback"],
    ] {
        let out = bladebro(&h, &args);
        let text = both(&out);
        assert!(
            !out.status.success(),
            "`bladebro {}` must fail in this build:\n{text}",
            args.join(" ")
        );
        assert!(
            text.contains("blocked:"),
            "`bladebro {}` must refuse with 'blocked:' (the verdict agents branch on):\n{text}",
            args.join(" ")
        );
        assert!(
            text.contains("FORK.md"),
            "`bladebro {}` must point at the fork's own record of why:\n{text}",
            args.join(" ")
        );
    }
}

/// `rb status` is a probe (the daemon surface and external tooling call it) and
/// `rb off` belongs to operators' setup scripts, so both stay successful — and
/// both must report the truth: there is no real-browser lane here.
#[test]
fn status_and_off_report_the_disabled_lane() {
    for sub in ["status", "off", "disable"] {
        let h = home(&format!("status-{sub}"));
        std::fs::write(
            h.join("realbrowser.json"),
            br#"{"enabled":true,"mode":"clone"}"#,
        )
        .unwrap();
        for extra in [&[][..], &["--json"][..]] {
            let args: Vec<&str> = ["rb", sub]
                .into_iter()
                .chain(extra.iter().copied())
                .collect();
            let out = bladebro(&h, &args);
            let text = both(&out);
            assert!(out.status.success(), "`bladebro rb {sub}`: {text}");
            if extra.is_empty() {
                assert!(
                    text.contains("removed in this build"),
                    "`bladebro rb {sub}` must say the lane is gone:\n{text}"
                );
            } else {
                let v = json(&out);
                assert_eq!(v["ok"], true, "`bladebro rb {sub} --json`: {v}");
                assert_eq!(v["enabled"], false, "the lane must report itself off: {v}");
                assert_eq!(
                    v["disabled_in_this_build"], true,
                    "a probe must be able to tell this build apart: {v}"
                );
            }
        }
    }
}

/// The realistic accident this fork prevents: a `realbrowser.json` left behind
/// by an upstream install (or hand-edited) that enables the real lane, plus an
/// explicit `BLADE_LANE=real` in the environment. Neither may be honoured.
#[test]
fn a_stale_enabled_config_cannot_open_the_real_lane() {
    let h = home("stale-config");
    std::fs::write(h.join("realbrowser.json"), br#"{"enabled":true}"#).unwrap();
    let out = bladebro(&h, &["rb", "status", "--json"]);
    let v = json(&out);
    assert_eq!(
        v["enabled"], false,
        "BLADE_LANE=real + enabled:true must still resolve to the agent lane: {v}"
    );

    // The library boundary is what every in-process reader consults: the lane
    // decision and the daemon's drift check. Asserted on an explicit path so the
    // test never reads the operator's own data dir.
    let cfg_path = h.join("realbrowser.json");
    assert!(
        !bladebro::realbrowser::config_from(&cfg_path).enabled,
        "the loader must not hand back an enabled real lane"
    );
    for env in [Some("real"), Some("agent"), None] {
        assert!(
            !bladebro::realbrowser::lane_from(env, true),
            "lane_from({env:?}, enabled=true) must be false in this build"
        );
    }
}

/// Copying a profile is the step that hands the agent every cookie and login in
/// it. It is refused at the function itself, independently of the lane gate, so
/// a refactor cannot reintroduce the read while the CLI still says "blocked".
#[test]
fn importing_a_real_profile_refuses_at_the_function() {
    let src = home("fake-profile-root");
    std::fs::create_dir_all(src.join("Default")).unwrap();
    std::fs::write(src.join("Local State"), b"{}").unwrap();
    std::fs::write(src.join("Default/Cookies"), b"not a real cookie DB").unwrap();

    let Err(failure) = bladebro::realbrowser::import_template("chrome", &src) else {
        panic!("importing a user profile must never succeed in this build");
    };
    let err = failure.to_string();
    assert!(err.contains("blocked:"), "unexpected refusal: {err}");
    assert!(err.contains("FORK.md"), "the refusal must say why: {err}");

    // A refusal must not have half-done anything: the source profile is exactly
    // as it was, and no template directory was created for this browser id.
    assert!(
        src.join("Default/Cookies").is_file(),
        "source must be untouched"
    );
    assert!(
        !bladebro::realbrowser::has_template("chrome"),
        "a refused import must leave no template behind"
    );
}

/// The machine-readable surface an agent actually reads: `rb` must not be
/// offered as a command, and the update paths must not look usable.
#[test]
fn help_does_not_advertise_removed_surfaces() {
    let h = home("help");
    let out = bladebro(&h, &["help", "--json"]);
    let v = json(&out);
    assert!(
        v["commands"]["rb"].is_null(),
        "help --json must not offer rb. commands: {}",
        v["commands"]
    );
    assert!(
        v["commands"].get("realbrowser").is_none(),
        "rb's long name must be absent too: {}",
        v["commands"]
    );
    for c in ["nav", "see", "act", "state", "run"] {
        assert!(v["commands"][c].is_object(), "{c} must still be documented");
    }

    for (cmd, needle) in [
        ("rb", "REMOVED"),
        ("update", "REMOVED"),
        ("rollback", "REMOVED"),
    ] {
        let out = bladebro(&h, &["help", cmd]);
        let text = both(&out);
        assert!(
            text.contains(needle),
            "`help {cmd}` must state the removal ({needle}):\n{text}"
        );
    }
}

/// `-v` must identify the build and must not contact the release registry: an
/// "update available" nag here is an invitation to install an unpatched binary.
#[test]
fn version_identifies_the_fork_and_stays_offline() {
    let h = home("version");
    let out = bladebro(&h, &["-v"]);
    let text = both(&out);
    assert!(
        text.contains("no-rb"),
        "`-v` must name the fork build:\n{text}"
    );
    assert!(
        !text.to_lowercase().contains("update available"),
        "`-v` must not nag about an update it must not install:\n{text}"
    );
}
