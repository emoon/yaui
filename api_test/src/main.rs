#![allow(dead_code)]

use crate::ui::FontStyle;
use minifb::{Key, Window, WindowOptions};
use clay_layout::{Declaration, fixed};
mod ui;
mod font;
mod render_api;
mod internal_error;
mod tiny_skia_renderer;

use ui::Ui;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let ui = Ui::new();

    let font = ui.load_font("data/Source_Sans_3/static/SourceSans3-Regular.ttf").unwrap();
    
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

    while window.is_open() && !window.is_key_down(Key::Escape) {
        for i in buffer.iter_mut() {
            *i = 0; // write something more funny here!
        }

        ui.begin((WIDTH, HEIGHT));
        
        ui.with_layout(&Declaration::new()
                .id(ui.id("red_rectangle"))
                .layout()
                    .width(fixed!(250.))
                    .height(fixed!(250.))
                .end()
                    .corner_radius()
                    .all(20.)
                .end()
                    .background_color((0xFF, 0x00, 0x00).into()), |_|
            {
                ui.label("Red Rectangle", (0xFF, 0xFF, 0xFF).into());
            }
        );

        ui.end(&mut buffer);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
