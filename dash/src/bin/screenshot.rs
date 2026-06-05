use slint::platform::software_renderer::{MinimalSoftwareWindow, RepaintBufferType};
use slint::platform::{Platform, WindowAdapter};
use slint::{PlatformError, Rgb8Pixel};
use std::rc::Rc;

slint::include_modules!();

const WIDTH: u32 = 800;
const HEIGHT: u32 = 480;

struct MyPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for MyPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }
}

fn main() {
    let window = MinimalSoftwareWindow::new(RepaintBufferType::NewBuffer);
    slint::platform::set_platform(Box::new(MyPlatform {
        window: window.clone(),
    }))
    .expect("platform already set");

    let ui = AppWindow::new().unwrap();
    ui.set_app_version(env!("CARGO_PKG_VERSION").into());

    ui.set_is_shuffle_enabled(true);

    ui.set_artist_name("Linkin Park".into());
    ui.set_album_name("Hybrid Theory".into());
    ui.set_track_title("Papercut".into());

    ui.set_next_track_title("In the End".into());
    ui.set_next_track_artist("Linkin Park".into());

    ui.set_total_time("3:05".into());
    ui.set_current_time("0:56".into());
    ui.set_progress(0.3);

    window.set_size(slint::PhysicalSize::new(WIDTH, HEIGHT));

    let pages = ["music", "library", "android-auto", "settings"];
    let mut buffer = vec![Rgb8Pixel { r: 0, g: 0, b: 0 }; (WIDTH * HEIGHT) as usize];

    let mut screenshot = |page: usize, name: &str, mode: &str| {
        ui.set_current_page(page as i32);
        window.window().request_redraw();

        buffer.fill(Rgb8Pixel { r: 0, g: 0, b: 0 });
        window.draw_if_needed(|renderer| {
            renderer.render(&mut buffer, WIDTH as usize);
        });

        let pixels: Vec<u8> = buffer.iter().flat_map(|p| [p.r, p.g, p.b]).collect();
        let filepath = format!("../screenshots/{name}_{mode}.png");

        image::save_buffer(&filepath, &pixels, WIDTH, HEIGHT, image::ColorType::Rgb8).unwrap();
        println!("Saved {filepath}");
    };

    for (page, name) in pages.iter().enumerate() {
        ui.set_dark_mode(true);
        screenshot(page, name, "dark");
    }

    for (page, name) in pages.iter().enumerate() {
        ui.set_dark_mode(false);
        screenshot(page, name, "light");
    }

}
