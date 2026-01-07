//! File system watcher for live compilation
//!
//! This module provides file watching capabilities for automatic
//! recompilation when source files change.

use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    receiver: Receiver<Result<Event, notify::Error>>,
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Result<Self, notify::Error> {
        let (tx, rx) = channel();

        let watcher = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default().with_poll_interval(Duration::from_millis(500)),
        )?;

        Ok(Self {
            watcher,
            receiver: rx,
        })
    }

    /// Start watching a path
    pub fn watch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self.watcher.watch(path, RecursiveMode::Recursive)
    }

    /// Stop watching a path
    pub fn unwatch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self.watcher.unwatch(path)
    }

    /// Get the next file change event (blocking)
    pub fn next_event(&self) -> Option<Event> {
        match self.receiver.recv() {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    }

    /// Try to get the next file change event (non-blocking)
    pub fn try_next_event(&self) -> Option<Event> {
        match self.receiver.try_recv() {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create file watcher")
    }
}
