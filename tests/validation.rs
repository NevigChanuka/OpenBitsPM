use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use openbits_validate::load_catalog;

const SEED: &str = include_str!("../manifests/v/VideoLAN/VLC.yaml");
const SEED_PATH: &str = "v/VideoLAN/VLC.yaml";
static NEXT_TREE: AtomicUsize = AtomicUsize::new(0);

struct Tree(PathBuf);

impl Tree {
    fn new() -> Self {
        loop {
            let id = NEXT_TREE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("openbits-validation-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("creating test directory: {error}"),
            }
        }
    }

    fn write(&self, path: &str, yaml: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, yaml).unwrap();
    }

    fn rejects(&self, message: &str) {
        let error = format!("{:#}", load_catalog(&self.0).unwrap_err());
        assert!(error.contains(message), "expected {message:?} in {error:?}");
    }
}

impl Drop for Tree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn seed_catalog_is_publishable_and_sorted() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../manifests");
    let catalog = load_catalog(&root).unwrap();
    assert!(!catalog.verified.is_empty());
    assert!(
        catalog
            .verified
            .windows(2)
            .all(|pair| { pair[0]["id"].as_str().unwrap() < pair[1]["id"].as_str().unwrap() })
    );
}

#[test]
fn review_record_is_excluded() {
    let tree = Tree::new();
    let review = SEED
        .replace(
            "status: verified",
            "status: requires-verification\nreview_reason: Confirm upstream release",
        )
        .lines()
        .filter(|line| {
            !["version:", "verification:", "  release:", "  license:"]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .collect::<Vec<_>>()
        .join("\n");
    tree.write(SEED_PATH, &review);
    let catalog = load_catalog(&tree.0).unwrap();
    assert!(catalog.verified.is_empty());
    assert_eq!(catalog.requires_verification, ["VideoLAN.VLC"]);
}

#[test]
fn malformed_yaml_fixtures_fail() {
    let tree = Tree::new();
    for fixture in [
        include_str!("fixtures/invalid/syntax.yaml"),
        include_str!("fixtures/invalid/duplicate-key.yaml"),
        include_str!("fixtures/invalid/multiple-documents.yaml"),
    ] {
        tree.write(SEED_PATH, fixture);
        tree.rejects("invalid YAML");
    }
}

#[test]
fn rejects_invalid_schema_values_and_urls() {
    let tree = Tree::new();
    let version = SEED
        .lines()
        .find(|line| line.starts_with("version:"))
        .unwrap();
    for (from, to, message) in [
        ("channel: stable", "channel: beta", "/channel"),
        (version, "version: \"3.0.24-rc1\"", "/version"),
        ("windows:", "android:", "/platforms"),
        ("architecture: x64", "architecture: mips", "/architecture"),
        ("provider: winget", "provider: flatpak", "/provider"),
        ("provider: winget", "provider: custom", "/provider"),
        ("package_id: VideoLAN.VLC", "package_id: ''", "/package_id"),
        (
            "license: GPL-2.0-or-later",
            "license: MadeUp",
            "invalid SPDX",
        ),
        (
            "license: GPL-2.0-or-later",
            "license: MIT OR SSPL-1.0",
            "OSI-approved",
        ),
        (
            "homepage: https://www.videolan.org/",
            "homepage: https://example.net@www.videolan.org/",
            "homepage",
        ),
    ] {
        assert!(SEED.contains(from), "test seed has no {from}");
        tree.write(SEED_PATH, &SEED.replace(from, to));
        tree.rejects(message);
    }
}

#[test]
fn direct_installers_require_a_sha256() {
    let tree = Tree::new();
    let direct = SEED
        .replace("provider: winget", "provider: github-releases")
        .replace("package_id: VideoLAN.VLC", "package_id: fictional/app");
    tree.write(SEED_PATH, &direct);
    tree.rejects("installer");
    let direct = format!(
        "{direct}\n      installer:\n        url: https://example.org/fixture.exe\n        sha256: \"invalid\"\n"
    );
    tree.write(SEED_PATH, &direct);
    tree.rejects("/sha256");
    tree.write(
        SEED_PATH,
        &direct.replace("\"invalid\"", &format!("\"{}\"", "ab".repeat(32))),
    );
    assert_eq!(load_catalog(&tree.0).unwrap().verified.len(), 1);
}

#[test]
fn rejects_wrong_path_and_version_directory() {
    let wrong = Tree::new();
    wrong.write("v/VideoLAN/Other.yaml", SEED);
    wrong.rejects("expected path");
    let history = Tree::new();
    history.write("v/VideoLAN/3.0.24/VLC.yaml", SEED);
    history.rejects("unexpected manifest path");
}

#[test]
fn rejects_duplicate_repositories_and_provider_ids() {
    let tree = Tree::new();
    tree.write(SEED_PATH, SEED);
    let duplicate = SEED.replace("\nid: VideoLAN.VLC", "\nid: Example.App");
    tree.write("e/Example/App.yaml", &duplicate);
    tree.rejects("share provider package winget videolan.vlc");
    let duplicate = duplicate.replace("package_id: VideoLAN.VLC", "package_id: Example.App");
    tree.write("e/Example/App.yaml", &duplicate);
    tree.rejects("share repository");
}

#[cfg(not(windows))]
#[test]
fn rejects_package_ids_that_differ_only_in_case() {
    let tree = Tree::new();
    tree.write(SEED_PATH, SEED);
    let duplicate = SEED.replace("\nid: VideoLAN.VLC", "\nid: videolan.vlc");
    tree.write("v/videolan/vlc.yaml", &duplicate);
    tree.rejects("share package ID videolan.vlc");
}

#[test]
fn cli_exits_nonzero_on_invalid_manifest() {
    let tree = Tree::new();
    tree.write(SEED_PATH, "schema_version: 1");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_openbits-validate"))
        .arg(&tree.0)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("schema error"));
}
