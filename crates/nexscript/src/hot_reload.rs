#![deny(clippy::all)]

use crate::runtime::ScriptRuntime;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

pub struct FileWatchEvent {
    pub path: String,
}

pub struct HotReloader {
    pub enabled: bool,
    pub watch_paths: Vec<String>,
    pub last_modified: HashMap<String, std::time::SystemTime>,
    pub poll_interval: Duration,
    pub runtime: Option<std::cell::RefCell<ScriptRuntime>>,
    rx: Option<mpsc::Receiver<FileWatchEvent>>,
    tx: Option<mpsc::Sender<FileWatchEvent>>,
}

impl HotReloader {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            enabled: true,
            watch_paths: Vec::new(),
            last_modified: HashMap::new(),
            poll_interval: Duration::from_millis(500),
            runtime: None,
            rx: Some(rx),
            tx: Some(tx),
        }
    }

    pub fn add_watch(&mut self, path: &str) {
        self.watch_paths.push(path.to_string());
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                self.last_modified.insert(path.to_string(), modified);
            }
        }
    }

    pub fn attach_runtime(&mut self, runtime: std::cell::RefCell<ScriptRuntime>) {
        self.runtime = Some(runtime);
    }

    pub fn poll(&mut self) -> Vec<FileWatchEvent> {
        let mut events = Vec::new();
        if !self.enabled { return events; }

        for path in &self.watch_paths {
            if !Path::new(path).exists() { continue; }
            if let Ok(metadata) = std::fs::metadata(path) {
                if let Ok(modified) = metadata.modified() {
                    let last = self.last_modified.get(path).copied();
                    if last.map(|l| modified > l).unwrap_or(true) {
                        self.last_modified.insert(path.clone(), modified);
                        events.push(FileWatchEvent { path: path.clone() });

                        // Auto-reload if runtime is attached
                        if let Some(ref rt) = self.runtime {
                            if let Ok(source) = std::fs::read_to_string(path) {
                                let mut runtime = rt.borrow_mut();
                                match runtime.load_script(&source) {
                                    Ok(()) => log::info!("[HotReload] Reloaded: {}", path),
                                    Err(e) => log::error!("[HotReload] Failed to reload {}: {}", path, e),
                                }
                            }
                        }
                    }
                }
            }
        }
        events
    }

    pub fn enable(&mut self) { self.enabled = true; }
    pub fn disable(&mut self) { self.enabled = false; }

    pub fn is_modified(&self, path: &str) -> bool {
        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Some(&last) = self.last_modified.get(path) {
                    return modified > last;
                }
            }
        }
        false
    }

    pub fn force_reload(&mut self, path: &str) -> Result<(), String> {
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("Cannot read {}: {}", path, e))?;
        if let Some(ref rt) = self.runtime {
            let mut runtime = rt.borrow_mut();
            runtime.load_script(&source)?;
            log::info!("[HotReload] Force reloaded: {}", path);
        }
        Ok(())
    }
}

impl Default for HotReloader {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotreloader_creation() {
        let hr = HotReloader::new();
        assert!(hr.enabled);
        assert_eq!(hr.watch_paths.len(), 0);
    }

    #[test]
    fn test_add_watch() {
        let mut hr = HotReloader::new();
        hr.add_watch("test.nxs");
        assert_eq!(hr.watch_paths.len(), 1);
    }

    #[test]
    fn test_poll_no_changes() {
        let mut hr = HotReloader::new();
        hr.add_watch("nonexistent.nxs");
        let events = hr.poll();
        assert!(events.is_empty());
    }

    #[test]
    fn test_poll_detects_change() {
        let mut hr = HotReloader::new();
        let tmp_path = std::env::temp_dir().join("test_hotreload.nxs");
        let path_str = tmp_path.to_str().unwrap().to_string();

        hr.add_watch(&path_str);

        // Write initial file AFTER watch has been added
        std::fs::write(&tmp_path, b"// test\n").unwrap();

        // First poll — should detect new file
        let events = hr.poll();
        assert_eq!(events.len(), 1, "Should detect new file");

        // Second poll — no change
        let events = hr.poll();
        assert!(events.is_empty(), "Should not detect change");

        std::fs::remove_file(&tmp_path).ok();
    }

    #[test]
    fn test_enable_disable() {
        let mut hr = HotReloader::new();
        hr.disable();
        assert!(!hr.enabled);
        hr.enable();
        assert!(hr.enabled);
    }
}
