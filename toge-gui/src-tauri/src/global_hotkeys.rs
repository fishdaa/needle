use crate::commands;
use crate::keyboard::KeyboardSettingsPayload;
use crate::state::AppState;
use std::str::FromStr;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

pub fn initialize(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    let config = state.load_config();
    let settings = crate::keyboard::settings_from_config(&config);
    state.set_cached_settings(settings.clone());
    register_window_hotkeys(app, &settings)
}

pub fn unregister_all_shortcuts(app: &AppHandle) -> Result<(), String> {
    eprintln!("[hotkeys] unregister_all");
    app.global_shortcut()
        .unregister_all()
        .map_err(|e| e.to_string())
}

pub fn register_window_hotkeys(
    app: &AppHandle,
    settings: &KeyboardSettingsPayload,
) -> Result<(), String> {
    eprintln!(
        "[hotkeys] register new={} show={} toggle={}",
        settings.new_window_hotkey,
        settings.show_window_hotkey,
        settings.toggle_window_hotkey
    );
    let manager = app.global_shortcut();
    manager.unregister_all().map_err(|e| e.to_string())?;

    for (action, accelerator) in [
        (WindowHotkeyAction::NewWindow, settings.new_window_hotkey.as_str()),
        (WindowHotkeyAction::ShowWindow, settings.show_window_hotkey.as_str()),
        (WindowHotkeyAction::ToggleWindow, settings.toggle_window_hotkey.as_str()),
    ] {
        if accelerator.is_empty() {
            continue;
        }

        let shortcut = Shortcut::from_str(accelerator).map_err(|e| e.to_string())?;
        manager
            .on_shortcut(shortcut, move |app, _shortcut, event| {
                handle_shortcut_event(app, action, event);
            })
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn handle_shortcut_event(app: &AppHandle, action: WindowHotkeyAction, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed {
        return;
    }

    eprintln!("[hotkeys] global event {:?}", action);
    let state = app.state::<AppState>();
    let _ = match action {
        WindowHotkeyAction::NewWindow => commands::create_new_main_window_internal(app, &state),
        WindowHotkeyAction::ShowWindow => commands::show_main_window_internal(app, &state),
        WindowHotkeyAction::ToggleWindow => commands::toggle_main_window_internal(app, &state),
    };
}

#[derive(Clone, Copy, Debug)]
enum WindowHotkeyAction {
    NewWindow,
    ShowWindow,
    ToggleWindow,
}
