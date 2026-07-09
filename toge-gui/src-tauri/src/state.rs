use crate::keyboard::KeyboardSettingsPayload;
use std::env;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use toge_core::config::Config;

pub struct AppState {
    socket_path: PathBuf,
    config_path: PathBuf,
    query_counter: AtomicU64,
    window_counter: AtomicU64,
    exiting: AtomicBool,
    last_window_action_ms: AtomicU64,
    cached_settings: Mutex<Option<KeyboardSettingsPayload>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            socket_path: crate::ipc_client::socket_path(),
            config_path: default_config_path(),
            query_counter: AtomicU64::new(1),
            window_counter: AtomicU64::new(1),
            exiting: AtomicBool::new(false),
            last_window_action_ms: AtomicU64::new(0),
            cached_settings: Mutex::new(None),
        }
    }

    pub fn socket_path(&self) -> PathBuf {
        self.socket_path.clone()
    }

    pub fn next_query_id(&self) -> u64 {
        self.query_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub fn next_window_id(&self) -> u64 {
        self.window_counter.fetch_add(1, Ordering::SeqCst)
    }

    pub fn mark_exiting(&self) {
        self.exiting.store(true, Ordering::SeqCst);
    }

    pub fn is_exiting(&self) -> bool {
        self.exiting.load(Ordering::SeqCst)
    }

    pub fn should_process_window_action(&self, debounce_ms: u64) -> bool {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let last = self.last_window_action_ms.load(Ordering::SeqCst);
        if now_ms.saturating_sub(last) < debounce_ms {
            return false;
        }
        self.last_window_action_ms.store(now_ms, Ordering::SeqCst);
        true
    }

    pub fn config_path(&self) -> PathBuf {
        self.config_path.clone()
    }

    pub fn load_config(&self) -> Config {
        Config::load(&self.config_path).unwrap_or_else(|_| Config::default_config())
    }

    pub fn save_config(&self, config: &Config) -> Result<(), String> {
        config.save(&self.config_path)
    }

    pub fn set_cached_settings(&self, settings: KeyboardSettingsPayload) {
        if let Ok(mut guard) = self.cached_settings.lock() {
            *guard = Some(settings);
        }
    }

    pub fn get_cached_settings(&self) -> Option<KeyboardSettingsPayload> {
        self.cached_settings
            .lock()
            .ok()
            .and_then(|guard| guard.clone())
    }
}

fn default_config_path() -> PathBuf {
    env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let home = env::var_os("HOME").expect("HOME not set");
            PathBuf::from(home).join(".config")
        })
        .join("toge")
        .join("config.toml")
}
