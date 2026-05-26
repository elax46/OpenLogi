//! Opt-in update check (P2.8).
//!
//! Off by default — controlled by [`AppSettings::check_for_updates`]. When
//! enabled, performs **exactly one** `HEAD` request per launch against
//! GitHub's "latest release" redirect for the OpenLogi repo and logs the
//! result. No automatic download, no polling.
//!
//! Why HEAD: GitHub's `/repos/{owner}/{repo}/releases/latest` endpoint
//! 302-redirects to the latest tag URL. A HEAD request returns the
//! redirect target without downloading the release notes payload, and
//! `ureq` exposes the resolved final URL via `ResponseExt::get_uri`.

use std::time::Duration;

use openlogi_core::config::AppSettings;
use tracing::{debug, info, warn};
use ureq::ResponseExt as _;

const RELEASES_LATEST_URL: &str = "https://github.com/AprilNEA/OpenLogi/releases/latest";

/// Run a one-shot update check on a dedicated OS thread if enabled. No-op
/// when `settings.check_for_updates` is false (the default).
///
/// Called from `main.rs` after the inventory + config load and before the
/// GUI runtime starts. Decoupled from GPUI so the HTTP roundtrip doesn't
/// block window draw.
pub fn maybe_check(settings: &AppSettings) {
    if !settings.check_for_updates {
        debug!("update check disabled — skipping HEAD");
        return;
    }
    let current = env!("CARGO_PKG_VERSION").to_string();
    let spawn = std::thread::Builder::new()
        .name("openlogi-updater".into())
        .spawn(move || run_check(&current));
    if let Err(e) = spawn {
        warn!(error = %e, "could not spawn updater thread");
    }
}

fn run_check(current_version: &str) {
    match fetch_latest_tag() {
        Ok(latest) => {
            if newer_than(&latest, current_version) {
                info!(
                    current = current_version,
                    latest = %latest,
                    "new OpenLogi release available — visit {RELEASES_LATEST_URL}"
                );
            } else {
                debug!(
                    current = current_version,
                    latest = %latest,
                    "OpenLogi is up to date"
                );
            }
        }
        Err(e) => debug!(error = %e, "update check failed (network / parse)"),
    }
}

fn fetch_latest_tag() -> Result<String, String> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .build();
    let agent: ureq::Agent = config.into();
    let response = agent
        .head(RELEASES_LATEST_URL)
        .call()
        .map_err(|e| format!("{e}"))?;
    let final_uri = response.get_uri().to_string();
    let tag = final_uri
        .rsplit("/tag/")
        .next()
        .ok_or_else(|| format!("unexpected redirect target: {final_uri}"))?;
    Ok(tag.trim_start_matches('v').to_string())
}

/// Crude semver-ish comparison: split on `.`, compare numerically. Falls
/// back to lexical compare on parse failure. Sufficient for tag strings
/// shaped like `0.0.2`; not a full semver impl (pre-release, build
/// metadata, etc. are out of scope).
fn newer_than(latest: &str, current: &str) -> bool {
    let parse = |s: &str| -> Vec<u32> { s.split('.').filter_map(|p| p.parse().ok()).collect() };
    let l = parse(latest);
    let c = parse(current);
    if l.is_empty() || c.is_empty() {
        return latest > current;
    }
    l > c
}

#[cfg(test)]
mod tests {
    use super::newer_than;

    #[test]
    fn newer_than_compares_numerically() {
        assert!(newer_than("0.0.2", "0.0.1"));
        assert!(newer_than("0.1.0", "0.0.99"));
        assert!(!newer_than("0.0.1", "0.0.2"));
        assert!(!newer_than("0.0.1", "0.0.1"));
    }

    #[test]
    fn newer_than_handles_unparseable() {
        // The function falls back to lex compare only when *both* sides
        // produce empty parts lists. A leading "v" produces [0] (via the
        // "0.0.0-rc"-style suffix parse), so we test the pure-junk case.
        assert!(newer_than("abc", "abb"));
    }
}
