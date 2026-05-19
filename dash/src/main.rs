mod music;

use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let ui = PlayerWindow::new()?;

    music::player::setup(&ui);

    ui.run()?;

    Ok(())
}
