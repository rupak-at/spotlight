use spotlight_core::{
    config::{self, Settings},
    files,
    model::{Entry, Kind},
    repository,
    search::Index,
};
use std::{fs, time::Instant};

fn entry(name: &str, kind: Kind) -> Entry {
    Entry {
        id: name.into(),
        name: name.into(),
        path: format!("/test/{name}"),
        kind,
        keywords: String::new(),
    }
}

#[test]
fn ranks_exact_prefix_substring_and_fuzzy_matches() {
    let index = Index::new(vec![
        entry("My Firefox notes", Kind::File),
        entry("Firefox Developer", Kind::App),
        entry("Firefox", Kind::App),
        entry("file explorer", Kind::App),
    ]);
    let results = index.search("firefox", None, 20).results;
    assert_eq!(
        results.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(),
        ["Firefox", "Firefox Developer", "My Firefox notes"]
    );
    assert_eq!(index.search("ffx", None, 1).results[0].name, "Firefox");
    assert!(index.search("no match at all", None, 20).results.is_empty());
}

#[test]
fn handles_unicode_filters_whitespace_and_bounded_results() {
    let index = Index::new(vec![
        entry("Café", Kind::Folder),
        entry("Café notes", Kind::File),
        entry("calculator", Kind::App),
    ]);
    assert_eq!(
        index.search(" CAFÉ ", Some(Kind::Folder), 30).results[0].name,
        "Café"
    );
    assert_eq!(index.search("", None, 1).results[0].kind, Kind::App);
    assert_eq!(index.search("", None, 0).results.len(), 1);
    assert!(index.search("café", Some(Kind::App), 20).results.is_empty());
}

#[test]
fn traversal_excludes_hidden_dependencies_and_symlinks_and_deduplicates_roots() {
    let dir = tempfile::tempdir().unwrap();
    for sub in ["docs", "node_modules", ".private"] {
        fs::create_dir(dir.path().join(sub)).unwrap();
    }
    for file in [
        "docs/readme.md",
        "node_modules/skip.js",
        ".private/secret",
        "hello.txt",
    ] {
        fs::write(dir.path().join(file), "fixture").unwrap();
    }
    std::os::unix::fs::symlink(dir.path(), dir.path().join("loop")).unwrap();
    let settings = Settings {
        roots: vec![
            dir.path().display().to_string(),
            dir.path().join("docs").display().to_string(),
        ],
        ..Settings::default()
    };
    let scan = files::scan(&settings, || false);
    assert_eq!(
        scan.entries
            .iter()
            .filter(|e| e.name == "readme.md")
            .count(),
        1
    );
    assert!(!scan
        .entries
        .iter()
        .any(|e| ["skip.js", "secret", "loop"].contains(&e.name.as_str())));
    assert!(scan.warnings.is_empty());
    assert!(!scan.truncated);
    let limited = files::scan(
        &Settings {
            max_entries: 2,
            ..settings
        },
        || false,
    );
    assert_eq!(limited.entries.len(), 2);
    assert!(limited.truncated);
}

#[test]
fn launch_scope_rejects_symlink_escape_and_removed_roots() {
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let path = root.path().join("note.txt");
    fs::write(&path, "ok").unwrap();
    let settings = Settings {
        roots: vec![root.path().display().to_string()],
        ..Settings::default()
    };
    assert!(files::allowed_path(&path, &settings).is_ok());
    fs::remove_file(&path).unwrap();
    fs::write(outside.path().join("secret"), "secret").unwrap();
    std::os::unix::fs::symlink(outside.path().join("secret"), &path).unwrap();
    assert!(files::allowed_path(&path, &settings).is_err());
}

#[test]
fn settings_roundtrip_and_validation() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("settings.json");
    let settings = Settings {
        roots: vec![dir.path().display().to_string()],
        ..Settings::default()
    };
    config::save(&path, &settings).unwrap();
    assert_eq!(config::load(&path).unwrap(), settings);
    assert!(Settings {
        accent: "url(evil)".into(),
        ..settings.clone()
    }
    .validate()
    .is_err());
    assert!(Settings {
        roots: vec!["/".into()],
        ..settings.clone()
    }
    .validate()
    .is_err());
    assert!(Settings {
        max_depth: 99,
        ..settings
    }
    .validate()
    .is_err());
    fs::write(path.clone(), "{broken").unwrap();
    assert!(config::load(&path).is_err());
}

#[test]
fn unavailable_saved_root_is_preserved_and_reported() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("removable-drive");
    fs::create_dir(&root).unwrap();
    let path = dir.path().join("settings.json");
    let settings = Settings {
        roots: vec![root.display().to_string()],
        ..Settings::default()
    };
    config::save(&path, &settings).unwrap();
    fs::remove_dir(root).unwrap();
    let loaded = config::load(&path).unwrap();
    assert_eq!(loaded.roots, settings.roots);
    assert!(!files::scan(&loaded, || false).warnings.is_empty());
    assert!(config::save(&path, &loaded).is_err());
}

#[test]
fn sqlite_snapshot_replacement_invalidates_old_settings_and_rolls_back_failures() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("index.sqlite3");
    repository::replace(&db, "roots-a", &[entry("a", Kind::App)]).unwrap();
    assert_eq!(repository::load(&db, "roots-a").unwrap().len(), 1);
    assert!(repository::load(&db, "roots-b").unwrap().is_empty());
    let duplicate = vec![entry("b", Kind::File), entry("b", Kind::File)];
    assert!(repository::replace(&db, "roots-b", &duplicate).is_err());
    assert_eq!(repository::load(&db, "roots-a").unwrap()[0].name, "a");
    repository::replace(&db, "roots-b", &[entry("c", Kind::Folder)]).unwrap();
    assert_eq!(repository::load(&db, "roots-b").unwrap()[0].name, "c");
    assert!(repository::load(&db, "roots-a").unwrap().is_empty());
}

#[test]
fn search_fifty_thousand_entries() {
    let entries = (0..50_000)
        .map(|i| entry(&format!("document-{i:05}.md"), Kind::File))
        .collect();
    let index = Index::new(entries);
    let start = Instant::now();
    let response = index.search("document-49999", None, 30);
    assert_eq!(response.results[0].name, "document-49999.md");
    assert!(response.results.len() <= 30);
    eprintln!("50,000 entries, query wall time: {:?}", start.elapsed());
}
