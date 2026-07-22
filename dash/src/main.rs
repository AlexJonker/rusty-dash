use slint::ComponentHandle;
use std::error::Error;
use std::sync::Arc;

use playback_rs::Player;

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
    let player = Arc::new(Player::new(None).unwrap());
    player.set_playing(false);
    let _android_auto = android_auto::auto::AndroidAutoController::new(&ui);
    let _music = music::MusicController::new(&ui, player.clone());
    music::MusicController::play_music(&ui, player);
    let _settings = settings::SettingsController::new(&ui);

    ui.run()?;

    Ok(())
}
