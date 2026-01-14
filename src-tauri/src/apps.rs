//! Linux Application Launcher
//!
//! Parses .desktop files from standard XDG locations and provides
//! application search functionality for the Ruty launcher.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Represents a desktop application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    pub id: String,
    pub name: String,
    pub generic_name: Option<String>,
    pub comment: Option<String>,
    pub exec: String,
    pub icon: Option<String>,
    pub categories: Vec<String>,
    pub keywords: Vec<String>,
    pub terminal: bool,
    pub no_display: bool,
    pub desktop_file: PathBuf,
}

impl Application {
    /// Launch the application
    pub fn launch(&self) -> Result<(), String> {
        // Parse the Exec field - remove field codes like %f, %u, etc.
        let exec = self.exec
            .replace("%f", "")
            .replace("%F", "")
            .replace("%u", "")
            .replace("%U", "")
            .replace("%d", "")
            .replace("%D", "")
            .replace("%n", "")
            .replace("%N", "")
            .replace("%i", "")
            .replace("%c", "")
            .replace("%k", "")
            .trim()
            .to_string();

        // Split into command and args
        let parts: Vec<&str> = exec.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty exec command".to_string());
        }

        let cmd = parts[0];
        let args = &parts[1..];

        // Spawn detached process
        Command::new(cmd)
            .args(args)
            .spawn()
            .map_err(|e| format!("Failed to launch {}: {}", self.name, e))?;

        Ok(())
    }
}

/// Application indexer - scans and caches desktop applications
pub struct AppIndexer {
    apps: Vec<Application>,
    name_index: HashMap<String, usize>,
}

impl AppIndexer {
    /// Create a new indexer and scan for applications
    pub fn new() -> Self {
        let mut indexer = Self {
            apps: Vec::new(),
            name_index: HashMap::new(),
        };
        indexer.scan();
        indexer
    }

    /// Get all applications
    pub fn all(&self) -> &[Application] {
        &self.apps
    }

    /// Search applications by query (fuzzy matching)
    pub fn search(&self, query: &str) -> Vec<&Application> {
        if query.is_empty() {
            // Return all visible apps sorted by name
            return self.apps.iter()
                .filter(|app| !app.no_display)
                .take(20)
                .collect();
        }

        let query_lower = query.to_lowercase();
        let mut results: Vec<(&Application, i32)> = self.apps.iter()
            .filter(|app| !app.no_display)
            .filter_map(|app| {
                let score = self.calculate_score(app, &query_lower);
                if score > 0 {
                    Some((app, score))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (highest first)
        results.sort_by(|a, b| b.1.cmp(&a.1));

        results.into_iter()
            .map(|(app, _)| app)
            .take(10)
            .collect()
    }

    /// Calculate match score for an app
    fn calculate_score(&self, app: &Application, query: &str) -> i32 {
        let name_lower = app.name.to_lowercase();
        
        // Exact match = highest score
        if name_lower == query {
            return 1000;
        }
        
        // Starts with = high score
        if name_lower.starts_with(query) {
            return 500 + (100 - name_lower.len() as i32).max(0);
        }
        
        // Contains = medium score
        if name_lower.contains(query) {
            return 200;
        }
        
        // Check generic name
        if let Some(ref generic) = app.generic_name {
            let generic_lower = generic.to_lowercase();
            if generic_lower.contains(query) {
                return 150;
            }
        }
        
        // Check keywords
        for keyword in &app.keywords {
            if keyword.to_lowercase().contains(query) {
                return 100;
            }
        }
        
        // Check categories
        for category in &app.categories {
            if category.to_lowercase().contains(query) {
                return 50;
            }
        }
        
        0
    }

    /// Scan standard XDG locations for .desktop files
    fn scan(&mut self) {
        let locations = self.get_desktop_dirs();
        
        for dir in locations {
            if let Ok(entries) = fs::read_dir(&dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.extension().map(|e| e == "desktop").unwrap_or(false) {
                        if let Some(app) = self.parse_desktop_file(&path) {
                            let idx = self.apps.len();
                            self.name_index.insert(app.name.to_lowercase(), idx);
                            self.apps.push(app);
                        }
                    }
                }
            }
        }

        // Sort by name
        self.apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    }

    /// Get standard XDG desktop file directories
    fn get_desktop_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        // System applications
        dirs.push(PathBuf::from("/usr/share/applications"));
        dirs.push(PathBuf::from("/usr/local/share/applications"));

        // User applications
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(format!("{}/.local/share/applications", home)));
        }

        // XDG_DATA_DIRS
        if let Ok(xdg_dirs) = std::env::var("XDG_DATA_DIRS") {
            for dir in xdg_dirs.split(':') {
                dirs.push(PathBuf::from(format!("{}/applications", dir)));
            }
        }

        // Flatpak
        dirs.push(PathBuf::from("/var/lib/flatpak/exports/share/applications"));
        if let Ok(home) = std::env::var("HOME") {
            dirs.push(PathBuf::from(format!("{}/.local/share/flatpak/exports/share/applications", home)));
        }

        // Snap
        dirs.push(PathBuf::from("/var/lib/snapd/desktop/applications"));

        dirs
    }

    /// Parse a .desktop file
    fn parse_desktop_file(&self, path: &PathBuf) -> Option<Application> {
        let content = fs::read_to_string(path).ok()?;
        
        let mut in_desktop_entry = false;
        let mut fields: HashMap<String, String> = HashMap::new();

        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Check for section headers
            if line.starts_with('[') {
                in_desktop_entry = line == "[Desktop Entry]";
                continue;
            }

            // Only parse [Desktop Entry] section
            if !in_desktop_entry {
                continue;
            }

            // Parse key=value
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim().to_string();
                let value = line[pos + 1..].trim().to_string();
                fields.insert(key, value);
            }
        }

        // Required fields
        let name = fields.get("Name")?.clone();
        let exec = fields.get("Exec")?.clone();
        
        // Check if it's an application (not Link or Directory)
        let entry_type = fields.get("Type").map(|s| s.as_str()).unwrap_or("Application");
        if entry_type != "Application" {
            return None;
        }

        // Parse categories
        let categories: Vec<String> = fields.get("Categories")
            .map(|c| c.split(';').filter(|s| !s.is_empty()).map(String::from).collect())
            .unwrap_or_default();

        // Parse keywords
        let keywords: Vec<String> = fields.get("Keywords")
            .map(|k| k.split(';').filter(|s| !s.is_empty()).map(String::from).collect())
            .unwrap_or_default();

        // Generate ID from filename
        let id = path.file_stem()?.to_string_lossy().to_string();

        Some(Application {
            id,
            name,
            generic_name: fields.get("GenericName").cloned(),
            comment: fields.get("Comment").cloned(),
            exec,
            icon: fields.get("Icon").cloned(),
            categories,
            keywords,
            terminal: fields.get("Terminal").map(|v| v == "true").unwrap_or(false),
            no_display: fields.get("NoDisplay").map(|v| v == "true").unwrap_or(false),
            desktop_file: path.clone(),
        })
    }
}

impl Default for AppIndexer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_indexer_creation() {
        let indexer = AppIndexer::new();
        println!("Found {} applications", indexer.apps.len());
        assert!(indexer.apps.len() > 0, "Should find some applications");
    }

    #[test]
    fn test_search() {
        let indexer = AppIndexer::new();
        let results = indexer.search("fire");
        for app in results {
            println!("Found: {} ({})", app.name, app.exec);
        }
    }
}
