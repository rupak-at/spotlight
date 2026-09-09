use crate::{
    config::Settings,
    model::{Entry, Kind},
    Result,
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Default)]
pub struct Scan {
    pub entries: Vec<Entry>,
    pub directories: Vec<PathBuf>,
    pub warnings: Vec<String>,
    pub truncated: bool,
}

pub fn scan(settings: &Settings, cancelled: impl Fn() -> bool) -> Scan {
    let mut scan = Scan::default();
    let mut seen = HashSet::new();
    let mut roots: Vec<_> = settings
        .roots
        .iter()
        .filter_map(|root| match Path::new(root).canonicalize() {
            Ok(path) if path.is_dir() => Some(path),
            _ => {
                scan.warnings.push(format!("Folder is unavailable: {root}"));
                None
            }
        })
        .collect();
    roots.sort();
    roots.dedup();
    let roots: Vec<_> = roots
        .iter()
        .filter(|root| {
            !roots
                .iter()
                .any(|other| other != *root && root.starts_with(other))
        })
        .cloned()
        .collect();
    'roots: for root in roots {
        let walker = WalkDir::new(&root)
            .follow_links(false)
            .max_depth(if settings.max_depth == 0 {
                usize::MAX
            } else {
                settings.max_depth
            })
            .sort_by_file_name()
            .into_iter()
            .filter_entry(|entry| {
                if entry.depth() == 0 {
                    return true;
                }
                let name = entry.file_name().to_string_lossy();
                (settings.include_hidden || !name.starts_with('.'))
                    && !settings.excluded_names.iter().any(|skip| skip == &name)
            });
        for item in walker {
            if cancelled() {
                break 'roots;
            }
            let item = match item {
                Ok(item) => item,
                Err(error) => {
                    if scan.warnings.len() < 10 {
                        scan.warnings.push(error.to_string());
                    }
                    continue;
                }
            };
            if item.file_type().is_symlink()
                || !(item.file_type().is_dir() || item.file_type().is_file())
            {
                continue;
            }
            if !seen.insert(item.path().to_path_buf()) {
                continue;
            }
            if settings.max_entries != 0 && scan.entries.len() >= settings.max_entries {
                scan.truncated = true;
                break 'roots;
            }
            if item.file_type().is_dir() {
                scan.directories.push(item.path().to_path_buf());
                if settings.max_depth != 0 && item.depth() == settings.max_depth {
                    scan.truncated = true;
                }
            }
            let path = item.path().to_string_lossy().into_owned();
            scan.entries.push(Entry {
                id: format!("path:{path}"),
                name: item.file_name().to_string_lossy().into_owned(),
                path,
                kind: if item.file_type().is_dir() {
                    Kind::Folder
                } else {
                    Kind::File
                },
                keywords: String::new(),
            });
        }
    }
    scan
}

pub fn allowed_path(path: &Path, settings: &Settings) -> Result<PathBuf> {
    let real = path
        .canonicalize()
        .map_err(|e| format!("This item is no longer available: {e}"))?;
    if settings
        .roots
        .iter()
        .filter_map(|root| Path::new(root).canonicalize().ok())
        .any(|root| real.starts_with(root))
    {
        Ok(real)
    } else {
        Err("This item is outside your current search folders. Refresh the index.".into())
    }
}
