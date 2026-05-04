use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub struct PivotMonitor;

impl PivotMonitor {
    pub async fn watch_history(path: String, tx: Sender<String>) {
        let watch_path = PathBuf::from(&path);

        if !watch_path.exists() {
            eprintln!("[Truth-Ctx] Watch directory not found: {}. Attempting to create.", path);
            if let Err(e) = std::fs::create_dir_all(&watch_path) {
                eprintln!("[Truth-Ctx] Cannot create watch directory: {}. Sentinel aborting.", e);
                return;
            }
        }

        let (sync_tx, sync_rx) = mpsc::channel::<notify::Result<Event>>();

        let mut watcher = match RecommendedWatcher::new(sync_tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[Truth-Ctx] Failed to create filesystem watcher: {}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&watch_path, RecursiveMode::Recursive) {
            eprintln!("[Truth-Ctx] Failed to watch '{}': {}", path, e);
            return;
        }

        println!("[Truth-Ctx] Sentinel watching: {}", path);

        // The notify receiver loop is blocking. Run it on a dedicated thread so we
        // don't starve the Tokio thread pool.
        let result = tokio::task::spawn_blocking(move || {
            let _watcher = watcher; // keep alive until this closure exits
            for res in sync_rx {
                match res {
                    Ok(event) if event.kind.is_modify() => {
                        for p in event.paths {
                            // to_string_lossy avoids panicking on non-UTF-8 Windows paths
                            let path_str = p.to_string_lossy().into_owned();
                            if tx.blocking_send(path_str).is_err() {
                                return; // main loop dropped receiver — shut down cleanly
                            }
                        }
                    }
                    Ok(_) => {} // ignore create/delete/access events
                    Err(e) => {
                        // File-lock errors (e.g. ERROR_LOCK_VIOLATION on Windows) are
                        // transient — log them and keep watching.
                        eprintln!("[Truth-Ctx] Watch event error (non-fatal): {}", e);
                    }
                }
            }
        })
        .await;

        if let Err(e) = result {
            eprintln!("[Truth-Ctx] Sentinel task terminated unexpectedly: {}", e);
        }
    }
}
