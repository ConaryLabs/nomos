#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

pub fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate is under the workspace crates directory")
        .to_owned()
}

pub fn scene_one() -> Vec<u8> {
    fs::read(root().join("fixtures/r2/scenes/scene_one.json")).expect("read scene one")
}

pub fn plan_one() -> Vec<u8> {
    fs::read(root().join("fixtures/r2/plans/scene_one.json")).expect("read plan one")
}

pub fn maximum() -> Vec<u8> {
    fs::read(root().join("fixtures/r2/maximum-observed-scene.json")).expect("read maximum scene")
}

pub fn fresh_dir(label: &str) -> PathBuf {
    let path = root().join("target/r2-tests").join(format!(
        "{label}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    if path.exists() {
        fs::remove_dir_all(&path).expect("remove prior test directory");
    }
    fs::create_dir_all(&path).expect("create test directory");
    path
}

pub fn replace_once(bytes: &[u8], before: &str, after: &str) -> Vec<u8> {
    let text = std::str::from_utf8(bytes).expect("fixture is UTF-8");
    let replaced = text.replacen(before, after, 1);
    assert_ne!(replaced, text, "mutation target is absent: {before}");
    replaced.into_bytes()
}
