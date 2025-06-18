use crate::font::FontHandle;
use clay_layout::math::{BoundingBox, Dimensions};
use clay_layout::render_commands::{Custom, RenderCommand, RenderCommandConfig};
use clay_layout::text::TextConfig;
use clay_layout::{ClayLayoutScope, Color as ClayColor};
use tiny_skia::*;
use crate::font::TextGenerator;

pub fn clay_to_tiny_skia_color(color: ClayColor) -> Color {
    Color::from_rgba8(
        (color.r).round() as u8,
        (color.g).round() as u8,
        (color.b).round() as u8,
        (color.a).round() as u8,
    )
}

fn clay_to_tiny_skia_rect(rect: BoundingBox) -> Rect {
    Rect::from_xywh(rect.x, rect.y, rect.width, rect.height)
        .expect("Invalid rectangle dimensions")
}

/// Represents a pre-rendered text glyph as a pixmap
pub type TextBuffer = Pixmap;

// Example usage of the Clay tiny-skia renderer with clipping
//
// let mut pixmap = Pixmap::new(800, 600).unwrap();
// let text_pixmaps = vec![/* your pre-rendered text pixmaps */];
//
// clay_tiny_skia_render(
//     &mut pixmap,
//     render_commands,
//     |_command, _custom, _pixmap| {
//         // Handle custom elements
//     },
//     &text_pixmaps,
// );
//
// // Convert to minifb buffer
// let buffer = pixmap_to_minifb_buffer(&pixmap);

/// Create a pixmap from A8 alpha data
pub fn pixmap_from_a8_data(width: u32, height: u32, alpha_data: &[u8]) -> Option<Pixmap> {
    let mut pixmap = Pixmap::new(width, height)?;

    for (i, &alpha) in alpha_data.iter().enumerate() {
        // Create white pixels with varying alpha
        let color = PremultipliedColorU8::from_rgba(255, 255, 255, alpha).unwrap();
        pixmap.pixels_mut()[i] = color;
    }

    Some(pixmap)
}

/// Draw a text pixmap onto the target pixmap with color modulation
fn draw_text_pixmap(
    target: &mut Pixmap,
    text_pixmap: &Pixmap,
    x: i32,
    y: i32,
    color: Color,
) {
    // Create a paint for color modulation if needed
    let mut paint = PixmapPaint::default();
    paint.opacity = color.alpha();
    paint.blend_mode = BlendMode::SourceOver;

    // If the text pixmap is in alpha-only format, you might want to
    // create a colored version first, or use it as a mask

    target.draw_pixmap(
        x,
        y,
        text_pixmap.as_ref(),
        &paint,
        Transform::identity(),
        None, // No additional clipping
    );
}

/*
/// Alternative: Create a colored text pixmap from an alpha mask
fn create_colored_text_pixmap(
    alpha_mask: &Pixmap,
    color: Color,
) -> Option<Pixmap> {
    let mut colored_pixmap = Pixmap::new(alpha_mask.width(), alpha_mask.height())?;

    // Fill with the desired color, using the alpha mask
    let mut paint = Paint::default();
    paint.set_color(color);

    // Method 1: Fill the entire pixmap with color
    colored_pixmap.fill(color);

    // Method 2: Use the alpha mask to modulate the color
    // This requires pixel-by-pixel operation or using the alpha mask as a pattern
    for (i, &alpha_pixel) in alpha_mask.pixels().iter().enumerate() {
        let alpha = alpha_pixel.alpha();
        if alpha > 0 {
            let final_color = Color::from_rgba8(
                color.red(),
                color.green(),
                color.blue(),
                (alpha as f32 * color.alpha() * 255.0) as u8,
            );
            colored_pixmap.pixels_mut()[i] = final_color.to_color_u8();
        }
    }

    Some(colored_pixmap)
}
 */

/// Create a path for rounded rectangle
fn create_rounded_rect_path(rect: Rect, corner_radii: &[f32; 4]) -> Option<Path> {
    let mut pb = PathBuilder::new();

    let [tl, tr, bl, br] = *corner_radii;
    let x = rect.x();
    let y = rect.y();
    let w = rect.width();
    let h = rect.height();

    // Start from top-left corner (after radius)
    pb.move_to(x + tl, y);

    // Top edge
    pb.line_to(x + w - tr, y);

    // Top-right corner
    if tr > 0.0 {
        pb.quad_to(x + w, y, x + w, y + tr);
    }

    // Right edge
    pb.line_to(x + w, y + h - br);

    // Bottom-right corner
    if br > 0.0 {
        pb.quad_to(x + w, y + h, x + w - br, y + h);
    }

    // Bottom edge
    pb.line_to(x + bl, y + h);

    // Bottom-left corner
    if bl > 0.0 {
        pb.quad_to(x, y + h, x, y + h - bl);
    }

    // Left edge
    pb.line_to(x, y + tl);

    // Top-left corner
    if tl > 0.0 {
        pb.quad_to(x, y, x + tl, y);
    }

    pb.close();
    pb.finish()
}

/// This is a port of Clay's raylib renderer using tiny-skia as the drawing API.
pub fn clay_tiny_skia_render<'a, ImageData: 'a, CustomElementData: 'a>(
    pixmap: &mut Pixmap,
    render_commands: impl Iterator<Item = RenderCommand<'a, ImageData, CustomElementData>>,
    text_generator: &TextGenerator,
    /*
    mut render_custom_element: impl FnMut(
        &RenderCommand<'a, ImageData, CustomElementData>,
        &Custom<'a, CustomElementData>,
        &mut Pixmap,
    ),
     */
) {
    // Save/restore stack for clipping
    let mut clip_stack: Vec<Option<Mask>> = Vec::new();

    for command in render_commands {
        match command.config {
            RenderCommandConfig::Text(text) => {
                let text_data = text.text;
                let font_size = text.font_size as u32;
                let font_id = text.font_id as FontHandle;
                
                if let Some(data) = text_generator.get_text(text_data, font_size, font_id) {
                    // Option 1: Direct draw if text_pixmap is already colored
                    let mut paint = PixmapPaint::default();
                    paint.blend_mode = BlendMode::SourceOver;

                    pixmap.draw_pixmap(
                        command.bounding_box.x as i32,
                        command.bounding_box.y as i32,
                        data.data.as_ref(),
                        &paint,
                        Transform::identity(),
                        None,
                    );
                }
                
                /*
                if let Some(text_pixmap) = text_pixmaps.get(text.font_id as usize) {
                    let color = clay_to_tiny_skia_color(text.color);

                    // Option 1: Direct draw if text_pixmap is already colored
                    let mut paint = PixmapPaint::default();
                    paint.opacity = color.alpha();
                    paint.blend_mode = BlendMode::SourceOver;

                    let current_clip = clip_stack.last().and_then(|c| c.as_ref());

                    pixmap.draw_pixmap(
                        command.bounding_box.x as i32,
                        command.bounding_box.y as i32,
                        text_pixmap.as_ref(),
                        &paint,
                        Transform::identity(),
                        current_clip,
                    );

                    // Option 2: If text_pixmap is alpha-only, create colored version first
                    // if let Some(colored_text) = create_colored_text_pixmap(text_pixmap, color) {
                    //     pixmap.draw_pixmap(
                    //         command.bounding_box.x as i32,
                    //         command.bounding_box.y as i32,
                    //         colored_text.as_ref(),
                    //         &PixmapPaint::default(),
                    //         Transform::identity(),
                    //         current_clip,
                    //     );
                    // }
                }

                 */
            }
            RenderCommandConfig::Image(image) => {
                /*
                // image.data should be a Pixmap containing the image data
                let image_pixmap = &image.data;

                let mut paint = PixmapPaint::default();
                paint.opacity = 1.0;
                paint.blend_mode = BlendMode::SourceOver;

                let current_clip = clip_stack.last().and_then(|c| c.as_ref());

                // For scaling/fitting, you might need to create a scaled version first
                // or use Transform to scale the image to fit the bounding box
                let scale_x = command.bounding_box.width / image_pixmap.width() as f32;
                let scale_y = command.bounding_box.height / image_pixmap.height() as f32;
                let transform = Transform::from_scale(scale_x, scale_y)
                    .post_translate(command.bounding_box.x, command.bounding_box.y);

                pixmap.draw_pixmap(
                    0, 0, // Using transform for positioning instead
                    image_pixmap.as_ref(),
                    &paint,
                    transform,
                    current_clip,
                );

                 */
            }
            RenderCommandConfig::ScissorStart() => {
                /*
                // Create a clip mask for the bounding box
                let clip_rect = clay_to_tiny_skia_rect(command.bounding_box);
                let mut new_clip_mask = Mask::new();

                // Create a path for the clipping rectangle
                if let Some(clip_path) = PathBuilder::from_rect(clip_rect) {
                    new_clip_mask.set_path(
                        pixmap.width(),
                        pixmap.height(),
                        &clip_path,
                        FillRule::Winding,
                        false, // anti-alias
                    );
                    clip_stack.push(Some(new_clip_mask));
                } else {
                    clip_stack.push(None);
                }
                 */
            }
            RenderCommandConfig::ScissorEnd() => {
                //clip_stack.pop();
            }
            RenderCommandConfig::Rectangle(rect) => {
                let mut paint = Paint::default();
                paint.set_color(clay_to_tiny_skia_color(rect.color));
                paint.anti_alias = true;

                let bounds = clay_to_tiny_skia_rect(command.bounding_box);
                let current_clip = None;//clip_stack.last().and_then(|c| c.as_ref());

                if rect.corner_radii.top_left > 0.0
                    || rect.corner_radii.top_right > 0.0
                    || rect.corner_radii.bottom_left > 0.0
                    || rect.corner_radii.bottom_right > 0.0
                {
                    let corner_radii = [
                        rect.corner_radii.top_left,
                        rect.corner_radii.top_right,
                        rect.corner_radii.bottom_left,
                        rect.corner_radii.bottom_right,
                    ];

                    if let Some(path) = create_rounded_rect_path(bounds, &corner_radii) {
                        pixmap.fill_path(
                            &path,
                            &paint,
                            FillRule::Winding,
                            Transform::identity(),
                            current_clip,
                        );
                    }
                } else {
                    pixmap.fill_rect(
                        bounds,
                        &paint,
                        Transform::identity(),
                        current_clip,
                    );
                }
            }
            RenderCommandConfig::Border(border) => {
                let mut paint = Paint::default();
                paint.set_color(clay_to_tiny_skia_color(border.color));
                paint.anti_alias = true;

                let bb = &command.bounding_box;
                let current_clip = clip_stack.last().and_then(|c| c.as_ref());

                // Draw each border side using fill rectangles

                // Left border
                if border.width.left > 0 {
                    let rect = Rect::from_xywh(
                        bb.x,
                        bb.y + border.corner_radii.top_left,
                        border.width.left as f32,
                        bb.height - border.corner_radii.top_left - border.corner_radii.bottom_left,
                    );
                    if let Some(rect) = rect {
                        pixmap.fill_rect(rect, &paint, Transform::identity(), current_clip);
                    }
                }

                // Right border
                if border.width.right > 0 {
                    let rect = Rect::from_xywh(
                        bb.x + bb.width - border.width.right as f32,
                        bb.y + border.corner_radii.top_right,
                        border.width.right as f32,
                        bb.height - border.corner_radii.top_right - border.corner_radii.bottom_right,
                    );
                    if let Some(rect) = rect {
                        pixmap.fill_rect(rect, &paint, Transform::identity(), current_clip);
                    }
                }

                // Top border
                if border.width.top > 0 {
                    let rect = Rect::from_xywh(
                        bb.x + border.corner_radii.top_left,
                        bb.y,
                        bb.width - border.corner_radii.top_left - border.corner_radii.top_right,
                        border.width.top as f32,
                    );
                    if let Some(rect) = rect {
                        pixmap.fill_rect(rect, &paint, Transform::identity(), current_clip);
                    }
                }

                // Bottom border
                if border.width.bottom > 0 {
                    let rect = Rect::from_xywh(
                        bb.x + border.corner_radii.bottom_left,
                        bb.y + bb.height - border.width.bottom as f32,
                        bb.width - border.corner_radii.bottom_left - border.corner_radii.bottom_right,
                        border.width.bottom as f32,
                    );
                    if let Some(rect) = rect {
                        pixmap.fill_rect(rect, &paint, Transform::identity(), current_clip);
                    }
                }

                // For corners with radii, we need to draw arcs using paths
                // tiny-skia doesn't have direct arc drawing, so we approximate with curves

                // Helper to create an arc path (approximate with quadratic curves)
                let create_arc_path = |center_x: f32, center_y: f32, radius: f32, start_angle: f32, end_angle: f32| -> Option<Path> {
                    let mut pb = PathBuilder::new();

                    // Simple approximation - for better arcs, use multiple cubic curves
                    let start_x = center_x + radius * start_angle.to_radians().cos();
                    let start_y = center_y + radius * start_angle.to_radians().sin();
                    let end_x = center_x + radius * end_angle.to_radians().cos();
                    let end_y = center_y + radius * end_angle.to_radians().sin();

                    pb.move_to(start_x, start_y);
                    pb.line_to(end_x, end_y);

                    pb.finish()
                };

                // Draw corner arcs if needed
                if border.corner_radii.top_left > 0.0 {
                    let center_x = bb.x + border.corner_radii.top_left;
                    let center_y = bb.y + border.corner_radii.top_left;
                    if let Some(path) = create_arc_path(center_x, center_y, border.corner_radii.top_left, 180.0, 270.0) {
                        let stroke_paint = paint;
                        pixmap.stroke_path(&path, &stroke_paint, &Stroke::default(), Transform::identity(), current_clip);
                    }
                }
                // ... similar for other corners
            }
            RenderCommandConfig::Custom(ref custom) => {
                //render_custom_element(&command, custom, pixmap);
            }
            RenderCommandConfig::None() => {}
        }
    }
}

pub type TinySkiaClayScope<'clay, 'render, CustomElements> =
ClayLayoutScope<'clay, 'render, Pixmap, CustomElements>; // Using Pixmap for text/image data

// Helper function to get dimensions from your text pixmap
pub fn get_text_pixmap_dimensions(pixmap: &Pixmap) -> Dimensions {
    (pixmap.width() as f32, pixmap.height() as f32).into()
}

// Helper function to create a measure text function
// You'll need to implement this based on how you generate your text pixmaps
pub fn create_measure_text_function(
    text_pixmaps: &'static [Pixmap],
) -> impl Fn(&str, &TextConfig) -> Dimensions + 'static {
    |_text, text_config| {
        // Return dimensions based on your text pixmap for the given font
        if let Some(pixmap) = text_pixmaps.get(text_config.font_id as usize) {
            get_text_pixmap_dimensions(pixmap)
        } else {
            (0.0, 0.0).into()
        }
    }
}
