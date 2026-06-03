use slint::ComponentHandle;

use crate::AppWindow;

pub struct MusicController;

impl MusicController {
    pub fn new(ui: &AppWindow) -> Self {
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

        let ui_handle = ui.as_weak();
        ui.on_next_track(move || {
            let _ui = ui_handle.unwrap();
        });

        let ui_handle = ui.as_weak();
        ui.on_previous_track(move || {
            let _ui = ui_handle.unwrap();
        });

        let ui_handle = ui.as_weak();
        ui.on_set_progress(move |progress| {
            let ui = ui_handle.unwrap();
            let clamped = progress.clamp(0.0, 1.0);
            ui.set_progress(clamped);
            println!("Set progress to {}", clamped);
        });

        Self
    }
}
