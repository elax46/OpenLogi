//! `openlogi assets sync` — pull every device's bundle-required files
//! into the openlogi-gui crate so `cargo bundle` can pick them up.
//!
//! Fetches `index.json`, writes per-device files (skipping cache hits via
//! sha256 compare), and prunes depots that no longer appear in the
//! registry. Default `--out` matches the cargo-bundle resources path so
//! the workflow is `openlogi assets sync && cargo bundle --release`.

use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context as _, Result};
use clap::Args;
use openlogi_assets::http;

/// Default origin. Overridable via `--base` / `OPENLOGI_ASSETS`.
const DEFAULT_BASE: &str = "https://assets.openlogi.org";

/// Files the GUI's `AssetCache` opens at runtime. Everything else in
/// each depot stays remote until a feature needs it.
const FETCH_FILES: &[&str] = &["front_core.png", "core_metadata.json"];

const INDEX_NAME: &str = "index.json";

#[derive(Debug, Args)]
pub struct SyncArgs {
    /// Origin of the asset host.
    #[arg(long, default_value = DEFAULT_BASE, env = "OPENLOGI_ASSETS")]
    base: String,
    /// Destination directory. Default matches the cargo-bundle
    /// resources path declared in openlogi-gui/Cargo.toml.
    #[arg(long, default_value = "crates/openlogi-gui/assets")]
    out: PathBuf,
}

pub fn run(args: SyncArgs) -> Result<()> {
    let SyncArgs { base, out } = args;
    let base = base.trim_end_matches('/').to_string();
    fs::create_dir_all(&out).with_context(|| format!("create {}", out.display()))?;

    let (raw, index) = http::fetch_index_raw(&base)?;
    let index_path = out.join(INDEX_NAME);
    fs::write(&index_path, &raw)
        .with_context(|| format!("write {}", index_path.display()))?;
    println!("index.json: {} devices", index.devices.len());

    // Prune orphans so the bundle stays in sync with the registry.
    let expected: HashSet<&str> = index.devices.keys().map(String::as_str).collect();
    for entry in fs::read_dir(&out)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !expected.contains(name_str.as_ref()) {
            println!("  pruning {name_str}");
            fs::remove_dir_all(entry.path())?;
        }
    }

    let mut fetched = 0_u32;
    let mut cache_hits = 0_u32;
    let mut depots: Vec<&String> = index.devices.keys().collect();
    depots.sort();
    for depot in depots {
        let entry = &index.devices[depot];
        let dir = out.join(depot);
        fs::create_dir_all(&dir).with_context(|| format!("create {}", dir.display()))?;
        for name in FETCH_FILES {
            let Some(file_entry) = entry.files.iter().find(|f| f.name == *name) else {
                eprintln!("  WARN {depot}: registry missing {name}");
                continue;
            };
            let dst = dir.join(name);
            if http::cached_matches(&dst, &file_entry.sha256) {
                cache_hits += 1;
                continue;
            }
            let bytes = http::fetch_file(&base, &entry.asset_path, name)?;
            fs::write(&dst, &bytes)
                .with_context(|| format!("write {}", dst.display()))?;
            fetched += 1;
            println!("  {depot}/{name} ({} B)", file_entry.bytes);
        }
    }

    let bundle_bytes: u64 = index
        .devices
        .values()
        .flat_map(|d| d.files.iter())
        .filter(|f| FETCH_FILES.contains(&f.name.as_str()))
        .map(|f| f.bytes)
        .sum();
    #[allow(
        clippy::cast_precision_loss,
        reason = "bundle sizes are well under 2^53 bytes; f64 precision is fine for a display string"
    )]
    let mb = bundle_bytes as f64 / 1024.0 / 1024.0;
    println!(
        "done: {fetched} fetched, {cache_hits} cache-hit, {mb:.1} MB total under {}",
        out.display()
    );
    Ok(())
}
