use anyhow::Result;
use notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher as NotifyWatcher};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::draw_runtime::DrawRuntime;

pub struct Watcher {
    dirty_flag: Arc<Mutex<bool>>,
    watcher: ReadDirectoryChangesWatcher,
    currently_watching: Vec<PathBuf>,
}

impl Watcher {
    pub fn new() -> Result<Self> {
        let dirty_flag = Arc::new(Mutex::new(false));
        let thread_dirty_flag = dirty_flag.clone();
        let watcher = notify::recommended_watcher(
            move |res: Result<notify::Event, notify::Error>| match res {
                Ok(_) => {
                    *thread_dirty_flag.lock().unwrap() = true;
                }
                Err(err) => eprintln!("failed to watch files: {err}"),
            },
        )?;

        Ok(Watcher {
            dirty_flag,
            watcher: watcher,
            currently_watching: vec![],
        })
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty_flag.lock().unwrap().clone()
    }

    pub fn stop_watching(&mut self) -> Result<()> {
        let currently_watching = std::mem::take(&mut self.currently_watching);
        for path in currently_watching {
            self.watcher.unwatch(path.as_path())?
        }
        Ok(())
    }

    pub fn start_watching(&mut self, runtime: &mut DrawRuntime) -> Result<()> {
        self.stop_watching()?;
        *self.dirty_flag.lock().unwrap() = false;
        let loaded_modules = runtime.get_loaded_modules()?;
        for module_path in loaded_modules.iter() {
            self.watcher
                .watch(module_path, RecursiveMode::NonRecursive)?;
        }
        self.currently_watching = loaded_modules
            .into_iter()
            .map(|it| it.to_path_buf())
            .collect();
        Ok(())
    }
}
