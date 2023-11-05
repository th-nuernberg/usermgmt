use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver},
};

use hotwatch::{notify::event::CreateKind, Event, EventKind, Hotwatch};

use super::*;
#[derive(Debug)]
pub struct DebugSettingWatcher {
    reader: IoResourceManager<Settings>,
    on_file_change: Receiver<()>,
    start_task_if_ready: bool,
    _watcher: Hotwatch,
}

impl DebugSettingWatcher {
    pub fn tick(&mut self) -> Option<Settings> {
        if self.on_file_change.try_iter().last().is_some() {
            self.start_task_if_ready = true;
        }

        if self.start_task_if_ready && !self.reader.is_loading() {
            let _ = self
                .reader
                .spawn_task(load_settings, String::from("Reading setting file"));
            self.start_task_if_ready = false;
        }
        self.reader.query_task_and_take()
    }
}

impl Default for DebugSettingWatcher {
    fn default() -> Self {
        let mut hotwatcher = Hotwatch::new().expect("Failed to start hotwatcher");
        let (tx, rx) = channel::<()>();
        let settings_path = path_to_settings();
        let assets_path = path_to_assets_folder();
        info!("Developmet: watching file: {:?}", &settings_path);
        hotwatcher
            .watch(&assets_path, {
                let path = settings_path.clone();
                move |event: Event| {
                    if let EventKind::Modify(_) | EventKind::Create(CreateKind::File) = event.kind {
                        if event.paths.contains(&path) {
                            let _ = tx.send(());
                        }
                        debug!("File change for settings.toml detected.");
                    }
                }
            })
            .unwrap_or_else(|_| panic!("Failed to watch file at {:?}", &settings_path));
        Self {
            on_file_change: rx,
            reader: Default::default(),
            start_task_if_ready: false,
            _watcher: hotwatcher,
        }
    }
}
fn path_to_settings() -> PathBuf {
    use crate::constants::SETTINGS_FILE_NAME;
    path_to_assets_folder().join(SETTINGS_FILE_NAME)
}
fn path_to_assets_folder() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets")
}
fn load_settings() -> AppResult<Settings> {
    let path_to_settings = path_to_settings();
    let content = std::fs::read_to_string(path_to_settings)?;
    let parsed = toml::from_str(&content)?;
    Ok(parsed)
}
