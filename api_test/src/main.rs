#![allow(dead_code)]

use crate::ui::FontStyle;
use minifb::{Key, Window, WindowOptions};
mod daw_ui;
mod font;
mod internal_error;
mod render_api;
mod tiny_skia_renderer;
mod ui;

use ui::Ui;
use crate::daw_ui::{daw_ui, DawState};

// Re-export for use in other modules
pub use ui::{rgb, rgba};

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let ui = Ui::new();

    let font = ui
        .load_font("data/Source_Sans_3/static/SourceSans3-Regular.ttf")
        .unwrap();

    ui.register_font(font, FontStyle::Default);
    ui.set_font(font);

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(60);
    let mut daw_state = DawState::default(); // In real app, this would be persistent

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        ui.begin((WIDTH, HEIGHT));
        
        daw_ui(&mut daw_state, &ui, WIDTH as f32, HEIGHT as f32);

        ui.end(&mut buffer);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
