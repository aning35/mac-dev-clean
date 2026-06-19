use std::path::{Path, PathBuf};

/// Checks if a path is strictly forbidden to be deleted.
/// This prevents catastrophic errors if a scanner goes wild.
pub fn is_forbidden(path: &Path) -> bool {
    let forbidden_paths = vec![
        "/",
        "/System",
        "/Library",
        "/Applications",
        "/usr",
        "/bin",
        "/sbin",
        "/etc",
        "/var",
        "/private",
    ];

    let path_str = path.to_string_lossy();
    
    // Explicit forbidden paths
    for fb in forbidden_paths {
        if path_str == fb {
            return true;
        }
    }

    // Home directory and sensitive subdirectories
    if let Some(home_dir) = dirs::home_dir() {
        if path == home_dir {
            return true;
        }

        let sensitive_home_dirs = vec![
            home_dir.join("Documents"),
            home_dir.join("Desktop"),
            home_dir.join("Library/Keychains"),
            home_dir.join(".ssh"),
            home_dir.join(".gnupg"),
            home_dir.join(".kube"),
            home_dir.join(".aws"),
            home_dir.join(".azure"),
            home_dir.join(".config/gcloud"),
        ];

        for sensitive in sensitive_home_dirs {
            if path == sensitive {
                return true;
            }
        }
    }

    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if name == ".git" || name == ".env" || name == ".env.local" || name == "terraform.tfstate" {
            return true;
        }

        let lower_name = name.to_lowercase();
        if lower_name.ends_with(".pem")
            || lower_name.ends_with(".key")
            || lower_name.ends_with(".p12")
            || lower_name.ends_with(".mobileprovision")
            || lower_name.ends_with(".sqlite")
            || lower_name.ends_with(".db")
            || lower_name.ends_with(".wal")
            || lower_name.ends_with(".shm") {
            return true;
        }
    }

    false
}

/// Expand tilde (~) in paths to the actual home directory
pub fn expand_tilde(path: &Path) -> PathBuf {
    if !path.starts_with("~") {
        return path.to_path_buf();
    }

    let mut out_path = PathBuf::new();
    if let Some(home_dir) = dirs::home_dir() {
        out_path.push(home_dir);
    }
    
    // Strip the "~/" or "~"
    let stripped = path.strip_prefix("~").unwrap_or(path);
    out_path.push(stripped);
    
    out_path
}
