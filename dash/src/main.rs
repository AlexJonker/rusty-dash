use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;

    use slint::ComponentHandle;

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

    let window_weak = ui.as_weak();
    ui.on_quit(move || {
        let window = window_weak.unwrap();
        // Hide the window to terminate the slint run loop
        window.window().hide().unwrap();
    });

    ui.run()?;

    Ok(())
}
