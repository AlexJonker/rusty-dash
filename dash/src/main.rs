// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = PlayerWindow::new()?;

    let ui_handle = ui.as_weak();
    ui.on_play_pause(move || {
        let ui = ui_handle.unwrap();
        ui.set_is_playing(!ui.get_is_playing());
    });

    let ui_handle = ui.as_weak();
    ui.on_toggle_loop(move || {
        let ui = ui_handle.unwrap();
        ui.set_is_loop_enabled(!ui.get_is_loop_enabled());
    });

    let ui_handle = ui.as_weak();
    ui.on_toggle_shuffle(move || {
        let ui = ui_handle.unwrap();
        ui.set_is_shuffle_enabled(!ui.get_is_shuffle_enabled());
    });

    ui.run()?;

    Ok(())
}
