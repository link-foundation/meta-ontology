//! Keep the release lockfile in the same commit as the manifest version bump.

use std::path::Path;
use std::process::Command;

pub fn refresh_and_stage(root_manifest: &Path) -> Result<(), String> {
    let root = root_manifest.parent().unwrap_or_else(|| Path::new("."));
    for (program, args) in [
        (
            "cargo",
            vec!["update", "--workspace", "--manifest-path", "Cargo.toml"],
        ),
        ("git", vec!["add", "--", "Cargo.lock"]),
    ] {
        let output = Command::new(program)
            .args(args)
            .current_dir(root)
            .output()
            .map_err(|error| format!("Failed to execute {program}: {error}"))?;
        if !output.status.success() {
            return Err(format!(
                "{program} failed while preparing the release lockfile: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    Ok(())
}
