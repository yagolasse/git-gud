use std::fs;
use std::path::{Path, PathBuf};

pub struct AppPrefs {
    pub dark_mode: bool,
    pub last_repo: Option<PathBuf>,
}

impl AppPrefs {
    pub fn load_default() -> Self {
        let path = Self::default_path();
        Self::load_from_file(&path).unwrap_or_default()
    }

    pub fn save_default(&self) -> std::io::Result<()> {
        let path = Self::default_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        self.save_to_file(&path)
    }

    fn default_path() -> PathBuf {
        let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("git-gud");
        path.push("prefs.txt");
        path
    }

    fn load_from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        let mut prefs = Self::default();
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("dark_mode=") {
                prefs.dark_mode = val == "true";
            } else if let Some(val) = line.strip_prefix("last_repo=") {
                if !val.is_empty() {
                    prefs.last_repo = Some(PathBuf::from(val));
                }
            }
        }
        Ok(prefs)
    }

    fn save_to_file(&self, path: &Path) -> std::io::Result<()> {
        let mut lines = vec![format!("dark_mode={}", self.dark_mode)];
        if let Some(repo) = &self.last_repo {
            lines.push(format!("last_repo={}", repo.display()));
        } else {
            lines.push("last_repo=".to_string());
        }
        fs::write(path, lines.join("\n"))
    }
}

impl Default for AppPrefs {
    fn default() -> Self {
        Self { dark_mode: false, last_repo: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_round_trip_dark_mode() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("prefs.txt");
        let prefs = AppPrefs { dark_mode: true, last_repo: None };
        prefs.save_to_file(&path).unwrap();
        let loaded = AppPrefs::load_from_file(&path).unwrap();
        assert!(loaded.dark_mode);
        assert!(loaded.last_repo.is_none());
    }

    #[test]
    fn test_round_trip_last_repo() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("prefs.txt");
        let repo_path = PathBuf::from("/some/repo/path");
        let prefs = AppPrefs { dark_mode: false, last_repo: Some(repo_path.clone()) };
        prefs.save_to_file(&path).unwrap();
        let loaded = AppPrefs::load_from_file(&path).unwrap();
        assert!(!loaded.dark_mode);
        assert_eq!(loaded.last_repo, Some(repo_path));
    }

    #[test]
    fn test_load_nonexistent_falls_back_to_default() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.txt");
        let prefs = AppPrefs::load_from_file(&path).unwrap_or_default();
        assert!(!prefs.dark_mode);
        assert!(prefs.last_repo.is_none());
    }

    #[test]
    fn test_round_trip_dark_mode_false() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("prefs.txt");
        let prefs = AppPrefs { dark_mode: false, last_repo: None };
        prefs.save_to_file(&path).unwrap();
        let loaded = AppPrefs::load_from_file(&path).unwrap();
        assert!(!loaded.dark_mode);
    }
}
