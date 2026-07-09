pub mod actions;
pub mod commands;
pub mod format;
pub mod global_hotkeys;
pub mod ipc_client;
pub mod keyboard;
pub mod state;
pub mod tray;

use std::io;
use state::AppState;
use tauri::Manager;
use tauri::WindowEvent;

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::new())
        .setup(|app| {
            crate::tray::initialize(&app.handle())
                .map_err(io::Error::other)?;
            crate::global_hotkeys::initialize(&app.handle())
                .map_err(io::Error::other)?;
            Ok(())
        })
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. }) {
                let _ = crate::commands::handle_main_window_close_requested(window, event);
            }

            if let WindowEvent::Focused(focused) = event {
                let label = window.label();
                eprintln!("[focus] {} focused={}", label, focused);
                if !crate::commands::is_main_window_label(label) {
                    return;
                }

                let app = window.app_handle();
                if *focused {
                    let _ = crate::global_hotkeys::unregister_all_shortcuts(app);
                } else {
                    let any_focused = app
                        .webview_windows()
                        .iter()
                        .any(|(l, w)| {
                            crate::commands::is_main_window_label(l)
                                && w.is_focused().unwrap_or(false)
                        });
                    if !any_focused {
                        let state = app.state::<AppState>();
                        let settings = match state.get_cached_settings() {
                            Some(s) => s,
                            None => {
                                let config = state.load_config();
                                let s = crate::keyboard::settings_from_config(&config);
                                state.set_cached_settings(s.clone());
                                s
                            }
                        };
                        let _ = crate::global_hotkeys::register_window_hotkeys(app, &settings);
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::search_query,
            commands::get_status,
            commands::open_path,
            commands::reveal_in_folder,
            commands::copy_to_clipboard,
            commands::trash_path,
            commands::delete_path,
            commands::reindex_index,
            commands::run_watcher_self_test,
            commands::open_debug_window,
            commands::open_options_window,
            commands::close_options_window,
            commands::create_new_main_window,
            commands::show_main_window,
            commands::toggle_main_window,
            commands::get_keyboard_settings,
            commands::save_keyboard_settings,
            commands::restore_default_keyboard_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
