use std::fs;
use std::path::Path;
use std::process::Command;

#[path = "../../../scripts/release-lockfile.rs"]
mod release_helper;

fn run(root: &Path, program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{program} {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn release_commits_updated_lockfile_for_each_supported_layout() {
    for layout in [".", "rust", "workspace"] {
        let repo = tempfile::tempdir().unwrap();
        let root = repo.path();
        run(root, "git", &["init", "-q"]);
        run(root, "git", &["config", "user.name", "Release test"]);
        run(
            root,
            "git",
            &["config", "user.email", "release@example.com"],
        );
        let cargo_root = root.join(layout);
        fs::create_dir_all(&cargo_root).unwrap();
        let package_root = if layout == "workspace" {
            fs::write(
                cargo_root.join("Cargo.toml"),
                "[workspace]\nmembers = [\"member\"]\nresolver = \"2\"\n",
            )
            .unwrap();
            cargo_root.join("member")
        } else {
            cargo_root.clone()
        };
        fs::create_dir_all(package_root.join("src")).unwrap();
        fs::write(package_root.join("src/lib.rs"), "").unwrap();
        let manifest =
            "[package]\nname = \"release-fixture\"\nversion = \"0.9.0\"\nedition = \"2021\"\n";
        fs::write(package_root.join("Cargo.toml"), manifest).unwrap();
        run(&cargo_root, "cargo", &["generate-lockfile", "--offline"]);
        run(root, "git", &["add", "."]);
        run(root, "git", &["commit", "-qm", "Initial version"]);
        fs::write(
            package_root.join("Cargo.toml"),
            manifest.replace("0.9.0", "0.10.0"),
        )
        .unwrap();

        // This is the production release helper: a version bump must refresh
        // AND stage the workspace lockfile before the release commit is made.
        release_helper::refresh_and_stage(&cargo_root.join("Cargo.toml")).unwrap();
        let lock_path = format!("{layout}/Cargo.lock");
        run(root, "git", &["diff", "--exit-code", "--", &lock_path]);
        let staged = run(root, "git", &["diff", "--cached"]);
        assert!(staged.contains("+version = \"0.10.0\""));
        run(root, "git", &["add", "."]);
        run(root, "git", &["commit", "-qm", "Release version"]);
        run(
            &cargo_root,
            "cargo",
            &["metadata", "--offline", "--locked", "--format-version", "1"],
        );
        assert_eq!(run(root, "git", &["status", "--porcelain"]), "");
    }
}

#[test]
fn invalid_manifest_stops_release_preparation() {
    let repo = tempfile::tempdir().unwrap();
    fs::write(repo.path().join("Cargo.toml"), "invalid manifest").unwrap();
    let error = release_helper::refresh_and_stage(&repo.path().join("Cargo.toml")).unwrap_err();
    assert!(error.contains("cargo failed"));
}
