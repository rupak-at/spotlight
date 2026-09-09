use crate::Result;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub roots: Vec<String>,
    pub excluded_names: Vec<String>,
    pub include_hidden: bool,
    pub max_entries: usize,
    pub max_depth: usize,
    pub result_limit: usize,
    pub shortcut: String,
    pub theme: String,
    pub accent: String,
    pub compact: bool,
    pub background_opacity: u8,
}

impl Default for Settings {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let roots = ["Desktop", "Documents", "Downloads", "Pictures", "projects"]
            .iter()
            .map(|name| home.join(name))
            .filter(|p| p.is_dir())
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        Self {
            roots,
            excluded_names: [
                "node_modules",
                "target",
                "dist",
                "build",
                ".git",
                ".cache",
                ".venv",
                "venv",
                "__pycache__",
            ]
            .map(String::from)
            .to_vec(),
            include_hidden: false,
            max_entries: 50_000,
            max_depth: 12,
            result_limit: 30,
            shortcut: "Super+Space".into(),
            theme: "dark".into(),
            accent: "#c1c5cf".into(),
            compact: true,
            background_opacity: 94,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<()> {
        self.validate_inner(true)
    }

    fn validate_inner(&self, require_available_roots: bool) -> Result<()> {
        if !(80..=100).contains(&self.background_opacity) {
            return Err("Background opacity must be between 80 and 100 percent.".into());
        }
        if !(1..=100).contains(&self.result_limit) {
            return Err("Result limit must be between 1 and 100.".into());
        }
        if self.max_entries != 0 && !(100..=200_000).contains(&self.max_entries) {
            return Err("Index limit must be 0 (unlimited) or between 100 and 200,000.".into());
        }
        if self.max_depth > 32 {
            return Err("Search depth must be 0 (unlimited) or between 1 and 32.".into());
        }
        if !["dark", "light", "system"].contains(&self.theme.as_str()) {
            return Err("Unknown theme.".into());
        }
        if self.accent.len() != 7
            || !self.accent.starts_with('#')
            || !self.accent[1..].chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err("Accent must be a six-digit hex color.".into());
        }
        if self.shortcut.trim().is_empty() || self.shortcut.len() > 100 {
            return Err("Provide a valid keyboard shortcut.".into());
        }
        if self.roots.len() > 32 || self.excluded_names.len() > 128 {
            return Err("Too many search roots or exclusions.".into());
        }
        for root in &self.roots {
            let path = Path::new(root);
            if !path.is_absolute() || (require_available_roots && !path.is_dir()) {
                return Err(format!(
                    "Search folder must be an existing absolute directory: {root}"
                ));
            }
            if path == Path::new("/")
                || path.canonicalize().is_ok_and(|real| real == Path::new("/"))
            {
                return Err("Choose specific folders instead of the entire filesystem.".into());
            }
        }
        for name in &self.excluded_names {
            if name.is_empty() || name.contains('/') || name.len() > 255 {
                return Err("Exclusions must be folder or file names, without slashes.".into());
            }
        }
        Ok(())
    }

    pub fn index_key(&self) -> String {
        serde_json::json!([
            1,
            self.roots,
            self.excluded_names,
            self.include_hidden,
            self.max_entries,
            self.max_depth
        ])
        .to_string()
    }
}

pub fn load(path: &Path) -> Result<Settings> {
    match fs::read(path) {
        Ok(bytes) => {
            let settings: Settings =
                serde_json::from_slice(&bytes).map_err(|e| format!("Invalid settings: {e}"))?;
            // An unplugged drive must not replace the user's chosen roots with defaults.
            // The indexer reports unavailable saved folders; newly saved roots must exist.
            settings.validate_inner(false)?;
            Ok(settings)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Settings::default()),
        Err(e) => Err(format!("Cannot read settings: {e}")),
    }
}

pub fn save(path: &Path, settings: &Settings) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    settings.validate()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let bytes = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .mode(0o600)
        .open(&tmp)
        .map_err(|e| e.to_string())?;
    file.write_all(&bytes)
        .and_then(|_| file.sync_all())
        .map_err(|e| e.to_string())?;
    fs::rename(tmp, path).map_err(|e| e.to_string())
}
