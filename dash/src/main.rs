use slint::ComponentHandle;
use std::error::Error;

slint::include_modules!();

mod android_auto;
mod music;
mod settings;

fn main() -> Result<(), Box<dyn Error>> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Warn)
        .init()
        .ok();
    let ui = AppWindow::new()?;
    let _android_auto = android_auto::AndroidAutoController::new(&ui);
    let _music = music::MusicController::new(&ui);
    let _settings = settings::SettingsController::new(&ui);

    ui.run()?;

    Ok(())
}
