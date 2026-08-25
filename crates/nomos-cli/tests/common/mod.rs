use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The kernel fixture plus every world discovered in the executable corpus.
pub fn worlds() -> Vec<(String, String)> {
    let root = repo_root();
    let mut worlds = vec![(
        "fixtures/gaol.nomos".to_owned(),
        fs::read_to_string(root.join("fixtures/gaol.nomos"))
            .expect("the kernel fixture is readable"),
    )];
    let areas = root.join("experiments/executable-gaol/areas");
    let mut paths = fs::read_dir(&areas)
        .expect("the executable corpus is readable")
        .filter_map(|entry| {
            let path = entry.expect("an area-directory entry is readable").path();
            path.is_dir().then(|| path.join("world.nomos"))
        })
        .filter(|path| path.is_file())
        .collect::<Vec<_>>();
    paths.sort();
    worlds.extend(paths.into_iter().map(|path| {
        let relative = path
            .strip_prefix(&root)
            .expect("the corpus world is inside the repository")
            .to_str()
            .expect("the corpus path is UTF-8")
            .to_owned();
        let source = fs::read_to_string(path).expect("the corpus world is readable");
        (relative, source)
    }));
    worlds
}
