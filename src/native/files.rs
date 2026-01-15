//! File Search Module
//!
//! Provides fast file searching using fd (or find as fallback).
//! Searches common user directories and returns results with paths.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;

/// File search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileResult {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub extension: Option<String>,
}

/// File searcher - uses fd for fast searching
pub struct FileSearcher {
    /// Use fd if available, otherwise fall back to find
    use_fd: bool,
}

impl FileSearcher {
    pub fn new() -> Self {
        // Check if fd is available
        let use_fd = Command::new("fd")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        
        Self { use_fd }
    }

    /// Search for files matching query
    pub fn search(&self, query: &str, max_results: usize, folders_only: bool) -> Vec<FileResult> {
        if query.is_empty() {
            return Vec::new();
        }

        if self.use_fd {
            self.search_fd(query, max_results, folders_only)
        } else {
            self.search_find(query, max_results, folders_only)
        }
    }

    /// Search using fd (fast, respects .gitignore)
    fn search_fd(&self, query: &str, max_results: usize, folders_only: bool) -> Vec<FileResult> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        
        // Search in common directories
        let search_dirs = vec![
            format!("{}", home),
            format!("{}/Documents", home),
            format!("{}/Downloads", home),
            format!("{}/Desktop", home),
            format!("{}/Projects", home),
        ];

        let mut results = Vec::new();
        
        let mut fd_args = vec![
            "--hidden".to_string(),
            "--no-ignore".to_string(),
            "--max-depth".to_string(), "4".to_string(),
            "--max-results".to_string(), max_results.to_string(),
            "-i".to_string(),
        ];
        
        if folders_only {
            fd_args.push("--type".to_string());
            fd_args.push("d".to_string());
        } else {
            fd_args.push("--type".to_string());
            fd_args.push("f".to_string());
            fd_args.push("--type".to_string());
            fd_args.push("d".to_string());
        }
        
        fd_args.push(query.to_string());

        for dir in search_dirs {
            if !std::path::Path::new(&dir).exists() {
                continue;
            }

            let output = Command::new("fd")
                .args(&fd_args)
                .current_dir(&dir)
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines().take(max_results - results.len()) {
                        let path = if line.starts_with('/') {
                            PathBuf::from(line)
                        } else {
                            PathBuf::from(&dir).join(line)
                        };
                        
                        if let Some(result) = self.path_to_result(&path) {
                            results.push(result);
                        }
                    }
                }
            }

            if results.len() >= max_results {
                break;
            }
        }

        results.truncate(max_results);
        results
    }

    /// Search using find (fallback, slower)
    fn search_find(&self, query: &str, max_results: usize, folders_only: bool) -> Vec<FileResult> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
        
        let mut find_args = vec![
            home.as_str(),
            "-maxdepth", "4",
        ];
        
        if folders_only {
            find_args.extend_from_slice(&["-type", "d"]);
        }
        
        let query_pattern = format!("*{}*", query);
        find_args.extend_from_slice(&["-iname", &query_pattern, "-print"]);

        let output = Command::new("find")
            .args(&find_args)
            .output();

        let mut results = Vec::new();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines().take(max_results) {
                    let path = PathBuf::from(line);
                    if let Some(result) = self.path_to_result(&path) {
                        results.push(result);
                    }
                }
            }
        }

        results
    }

    /// Convert path to FileResult
    fn path_to_result(&self, path: &PathBuf) -> Option<FileResult> {
        let name = path.file_name()?.to_string_lossy().to_string();
        let is_dir = path.is_dir();
        let extension = if is_dir {
            None
        } else {
            path.extension().map(|e| e.to_string_lossy().to_string())
        };

        Some(FileResult {
            name,
            path: path.to_string_lossy().to_string(),
            is_dir,
            extension,
        })
    }

    /// Open file with default application
    pub fn open(&self, path: &str) -> Result<(), String> {
        // Use xdg-open on Linux
        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open {}: {}", path, e))?;
        Ok(())
    }

    /// Open file's containing folder
    pub fn reveal(&self, path: &str) -> Result<(), String> {
        let path = PathBuf::from(path);
        let folder = path.parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());
        
        Command::new("xdg-open")
            .arg(&folder)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
        Ok(())
    }
}

impl Default for FileSearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_search() {
        let searcher = FileSearcher::new();
        println!("Using fd: {}", searcher.use_fd);
        
        let results = searcher.search("rust", 10);
        for r in &results {
            println!("{}: {}", if r.is_dir { "DIR" } else { "FILE" }, r.path);
        }
    }
}
