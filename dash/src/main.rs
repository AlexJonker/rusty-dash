use std::error::Error;
use system_shutdown::shutdown;

slint::include_modules!();

mod android_auto;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Info)
        .init()
        .ok();
    let ui = AppWindow::new()?;
    let _android_auto: android_auto::AndroidAutoHandle = android_auto::AndroidAutoHandle::start(&ui);

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

    ui.on_shutdown(move || match shutdown() {
        Ok(_) => println!("Shutting down, bye!"),
        Err(error) => eprintln!("Failed to shut down: {}", error),
    });

    ui.run()?;

    Ok(())
}
