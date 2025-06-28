use crate::font::{FontHandle, TextGenerator};
use crate::internal_error::InternalResult;
use background_worker::WorkSystem;
use clay_layout::layout::{Alignment, LayoutAlignmentX, LayoutAlignmentY};
use clay_layout::{
    Clay, Clay_Dimensions, Clay_StringSlice, Clay_TextElementConfig, ClayLayoutScope, Declaration,
    color::Color as ClayColor, fixed, grow, id::Id, layout::LayoutDirection, math::Dimensions,
    text::TextConfig,
};
use glam::Vec4;
use std::cell::UnsafeCell;
use std::collections::HashMap;
use tiny_skia::Pixmap;

// TODO: We likely need something better than this
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum FontStyle {
    Default,
    Bold,
    Thin,
    Light,
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct ImageInfo {
    pixmap: Pixmap,
}

type UiDeclaration<'a> = Declaration<'a, ImageInfo, ()>;
type UiLayoutScope<'a> = ClayLayoutScope<'a, 'a, ImageInfo, ()>;
#[derive(Debug, Default)]
#[allow(dead_code)]
pub struct ItemState {
    pub aabb: Vec4,
    pub was_hovered: bool,
    pub was_clicked: bool,
    pub active: f32,
    pub frame: u64,
}

struct State<'a> {
    bg_worker: WorkSystem,
    layout: Clay,
    text_generator: TextGenerator,
    font_styles: HashMap<FontStyle, FontHandle>,
    item_states: HashMap<u32, ItemState>, // TODO: Arena hashmap
    active_font: FontHandle,
    layout_scope: Option<UiLayoutScope<'a>>,
    font_size: u32,
    window_size: (usize, usize),
    current_frame: u64,
    delta_time: f32,
    focus_id: Option<Id>,
}

impl<'a> State<'a> {
    #[inline(always)]
    pub fn layout(&mut self) -> &mut UiLayoutScope<'a> {
        unsafe { self.layout_scope.as_mut().unwrap_unchecked() }
    }
}

macro_rules! get_state_mut {
    ($self:expr) => {
        unsafe { &mut *$self.state.get() }
    };
}

macro_rules! get_layout_mut {
    ($self:expr) => {
        unsafe { $self.layout_scope.as_mut().unwrap_unchecked() }
    };
}

pub struct Ui<'a> {
    state: UnsafeCell<State<'a>>,
}

impl<'a> Ui<'a> {
    pub fn new() -> Box<Self> {
        let bg_worker = WorkSystem::new(2);

        let state = State {
            text_generator: TextGenerator::new(&bg_worker),
            layout: Clay::new(Dimensions::new(320.0, 256.0)),
            layout_scope: None,
            bg_worker,
            font_styles: HashMap::with_capacity(8),
            active_font: 0,
            font_size: 32,
            window_size: (320, 256),
            item_states: HashMap::with_capacity(64),
            current_frame: 0,
            delta_time: 0.0,
            focus_id: None,
        };

        let data = Box::new(Ui {
            state: UnsafeCell::new(state),
        });

        // This is a hack. To be fixed later
        unsafe {
            let raw_ptr = Box::into_raw(data);
            Clay::set_measure_text_function_unsafe(Self::measure_text_trampoline, raw_ptr as _);
            Box::from_raw(raw_ptr)
        }
    }

    unsafe extern "C" fn measure_text_trampoline(
        text_slice: Clay_StringSlice,
        config: *mut Clay_TextElementConfig,
        user_data: *mut core::ffi::c_void,
    ) -> Clay_Dimensions {
        unsafe {
            let text = core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                text_slice.chars as *const u8,
                text_slice.length as _,
            ));

            let text_config = TextConfig::from(*config);
            let ui = &*(user_data as *const Ui);

            ui.measure_text(text, &text_config).into()
        }
    }

    #[inline(always)]
    fn state(&'a self) -> &'a mut State<'a> {
        unsafe { &mut *self.state.get() }
    }

    // Internal helper for the area! macro
    #[doc(hidden)]
    pub fn __internal_with_layout<F>(&self, declaration: &Declaration<'a, ImageInfo, ()>, f: F)
    where
        F: FnOnce(&Ui),
    {
        let state = unsafe { &mut *self.state.get() };
        let clay_scope = state.layout();

        clay_scope.with(declaration, |_clay| {
            f(self);
        });
    }

    pub fn load_font(&self, path: &str) -> InternalResult<FontHandle> {
        let state = get_state_mut!(self);
        state.text_generator.load_font(path, &state.bg_worker)
    }

    pub fn register_font(&self, font_id: FontHandle, style: FontStyle) {
        let state = get_state_mut!(self);
        state.font_styles.insert(style, font_id);
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn set_font(&self, font_handle: FontHandle) {
        let state = get_state_mut!(self);
        state.active_font = font_handle;
    }

    #[allow(dead_code)]
    pub fn set_font_style(&'a self, style: FontStyle) {
        let state = self.state();
        if let Some(font_handle) = state.font_styles.get(&style) {
            state.active_font = *font_handle;
        } else {
            // Handle the case where the font style is not registered
            eprintln!("Font style {:?} not registered", style);
        }
    }

    pub fn text_size(&'a self, text: &str, font_size: u32) -> Dimensions {
        let state = self.state();
        let size = state
            .text_generator
            .measure_text_size(text, state.active_font, font_size as _)
            .unwrap();

        Dimensions::new(size.0 as _, size.1 as _)
    }

    fn measure_text(&'a self, text: &str, config: &TextConfig) -> Dimensions {
        self.text_size(text, config.font_size as u32)
    }

    pub fn label(&self, text: &str, col: ClayColor) {
        let state = get_state_mut!(self);
        let font_id = state.active_font;
        let font_size = state.font_size;

        let _ =
            state
                .text_generator
                .queue_generate_text(text, font_size, font_id, &state.bg_worker);

        self.with_layout(
            &Declaration::new()
                .id(self.id(text))
                .layout()
                .width(grow!())
                .height(fixed!(80.0))
                .child_alignment(Alignment::new(
                    LayoutAlignmentX::Center,
                    LayoutAlignmentY::Center,
                ))
                .child_gap(40)
                .direction(LayoutDirection::LeftToRight)
                .end(),
            |_ui| {
                let scope = state.layout();

                scope.text(
                    text,
                    TextConfig::new()
                        .font_id(font_id as u16)
                        .font_size(font_size as _)
                        .wrap_mode(clay_layout::text::TextElementConfigWrapMode::None)
                        .color(col)
                        .end(),
                );
            },
        );
    }

    pub fn with_layout<F: FnOnce(&Ui)>(&self, declaration: &Declaration<'a, ImageInfo, ()>, f: F) {
        let state = get_state_mut!(self);
        let scope = state.layout();

        scope.with(declaration, |_clay| {
            f(self);
        });
    }

    #[inline]
    pub fn id(&self, name: &str) -> Id {
        let state = get_state_mut!(self);
        let scope = state.layout();
        scope.id(name)
    }

    pub fn begin(&self, delta_time: f32, window_size: (usize, usize)) {
        let state = get_state_mut!(self);
        state.window_size = window_size;
        state.delta_time = delta_time;
        state
            .layout
            .set_layout_dimensions(Dimensions::new(window_size.0 as f32, window_size.1 as f32));

        state.layout_scope = Some(state.layout.begin::<ImageInfo, ()>());

        self.update();
    }

    fn update(&self) {
        let state = get_state_mut!(self);
        state.text_generator.update();
    }

    pub fn set_focus_id(&self, id: Id) {
        let state = unsafe { &mut *self.state.get() };
        state.focus_id = Some(id);
    }

    pub fn end(&self, output: &mut [u32]) {
        let state = get_state_mut!(self);
        let text_generator = &state.text_generator;
        let mut pixmap =
            Pixmap::new(state.window_size.0 as u32, state.window_size.1 as u32).unwrap();

        let scope = get_layout_mut!(state);

        // TODO: Fix me
        let render_items: Vec<_> = scope.end().collect();

        let anim_rate = 1.0 - 2f32.powf(-8.0 * state.delta_time);

        let focus_id = if let Some(id) = state.focus_id {
            id.id
        } else {
            scope.id("").id
        };

        for command in &render_items {
            let bb = command.bounding_box;

            let item = state.item_states.entry(command.id).or_insert(ItemState {
                ..Default::default()
            });

            let is_active = if command.id == focus_id.id { 1.0 } else { 0.0 };

            item.active += anim_rate * (is_active - item.active);
            item.aabb = Vec4::new(bb.x, bb.y, bb.x + bb.width, bb.y + bb.height);
            item.frame = state.current_frame;
        }

        crate::tiny_skia_renderer::clay_tiny_skia_render(
            &mut pixmap,
            &render_items,
            text_generator,
        );

        for (index, p) in pixmap.data().chunks_exact(4).enumerate() {
            // Convert RGBA to ARGB: tiny-skia uses RGBA, minifb expects ARGB
            output[index] = ((p[3] as u32) << 24) | // Alpha
                           ((p[0] as u32) << 16) | // Red  
                           ((p[1] as u32) << 8)  | // Green
                           (p[2] as u32); // Blue
        }

        // remove all items that doesn't match the current frame
        state
            .item_states
            .retain(|_, item| item.frame == state.current_frame);

        state.current_frame += 1;
    }
}

/// Creates an RGB color with values from 0-255
///
/// # Examples
/// ```rust
/// use crate::rgb;
///
/// let red = rgb(255, 0, 0);
/// let green = rgb(0, 255, 0);
/// let blue = rgb(0, 0, 255);
/// let gray = rgb(128, 128, 128);
/// ```
#[inline]
pub fn rgb(r: u8, g: u8, b: u8) -> ClayColor {
    ClayColor::rgb(r as f32, g as f32, b as f32)
}

/// Creates an RGBA color with values from 0-255 for RGBA
///
/// # Examples
/// ```rust
/// use crate::rgba;
///
/// let semi_red = rgba(255, 0, 0, 128);
/// let transparent_black = rgba(0, 0, 0, 0.0);
/// let opaque_white = rgba(255, 255, 255, 255);
/// ```
#[inline]
pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> ClayColor {
    ClayColor::rgba(r as f32, g as f32, b as f32, a as f32)
}

/// The `area!` macro provides a clean, intuitive way to create UI layouts without exposing
/// the underlying Clay implementation. It abstracts the complexity of Clay's declaration
/// system and provides a more user-friendly API.
///
/// # Syntax
/// ```rust
/// area!(ui, {
///     id: "my_element",
///     layout: {
///         width: fixed!(100.0),
///         height: grow!(),
///         direction: LayoutDirection::LeftToRight,
///         padding: Padding::all(10.0),
///         child_gap: 5,
///         child_alignment: Alignment::new(LayoutAlignmentX::Center, LayoutAlignmentY::Center),
///     },
///     background_color: rgb(50, 50, 50),
///     corner_radius: {
///         all: 5.0,
///     },
///     border: {
///         width: 2,
///         color: rgb(100, 100, 100),
///     },
/// }, |ui| {
///     // Child elements here
/// });
/// ```
#[macro_export]
macro_rules! area {
    ($ui:expr, {
        $(id: $id:expr,)?
        $(layout: {
            $(width: $width:expr,)?
            $(height: $height:expr,)?
            $(padding: $padding:expr,)?
            $(direction: $direction:expr,)?
            $(child_gap: $gap:expr,)?
            $(child_alignment: $align:expr,)?
        },)?
        $(corner_radius: {
            $(all: $radius:expr,)?
            $(top_left: $tl:expr,)?
            $(top_right: $tr:expr,)?
            $(bottom_left: $bl:expr,)?
            $(bottom_right: $br:expr,)?
        },)?
        $(background_color: $bg:expr,)?
        $(border: {
            $(width: $border_width:expr,)?
            $(left: $border_left:expr,)?
            $(right: $border_right:expr,)?
            $(top: $border_top:expr,)?
            $(bottom: $border_bottom:expr,)?
            $(between_children: $border_between:expr,)?
            $(color: $border_color:expr,)?
        },)?
        $(floating: {
            $(offset: $float_offset:expr,)?
            $(dimensions: $float_dimensions:expr,)?
            $(z_index: $float_z:expr,)?
            $(parent_id: $float_parent:expr,)?
            $(attach_points: ($float_element:expr, $float_parent_point:expr),)?
            $(attach_to: $float_attach:expr,)?
            $(pointer_capture_mode: $float_capture:expr,)?
        },)?
        $(aspect_ratio: $aspect:expr,)?
        $(clip: ($clip_h:expr, $clip_v:expr, $clip_offset:expr),)?
    }, $body:expr) => {
        {
            use clay_layout::Declaration;
            let mut decl = Declaration::new();

            // Set ID if provided (automatically convert string to ID)
            $(decl.id($ui.id($id));)?

            // Configure layout if provided
            $(
                {
                    let mut layout = decl.layout();
                    $(layout.width($width);)?
                    $(layout.height($height);)?
                    $(layout.padding($padding);)?
                    $(layout.direction($direction);)?
                    $(layout.child_gap($gap);)?
                    $(layout.child_alignment($align);)?
                    layout.end();
                }
            )?

            // Configure corner radius if provided
            $(
                {
                    let mut corner = decl.corner_radius();
                    $(corner.all($radius);)?
                    $(corner.top_left($tl);)?
                    $(corner.top_right($tr);)?
                    $(corner.bottom_left($bl);)?
                    $(corner.bottom_right($br);)?
                    corner.end();
                }
            )?

            // Set background color if provided
            $(decl.background_color($bg);)?

            // Configure border if provided
            $(
                {
                    let mut border = decl.border();
                    $(border.all_directions($border_width);)?
                    $(border.left($border_left);)?
                    $(border.right($border_right);)?
                    $(border.top($border_top);)?
                    $(border.bottom($border_bottom);)?
                    $(border.between_children($border_between);)?
                    $(border.color($border_color);)?
                    border.end();
                }
            )?

            // Configure floating if provided
            $(
                {
                    let mut floating = decl.floating();
                    $(floating.offset($float_offset);)?
                    $(floating.dimensions($float_dimensions);)?
                    $(floating.z_index($float_z);)?
                    $(floating.parent_id($float_parent);)?
                    $(floating.attach_points($float_element, $float_parent_point);)?
                    $(floating.attach_to($float_attach);)?
                    $(floating.pointer_capture_mode($float_capture);)?
                    floating.end();
                }
            )?

            // Set aspect ratio if provided
            $(decl.aspect_ratio($aspect);)?

            // Set clip if provided
            $(decl.clip($clip_h, $clip_v, $clip_offset);)?

            // Execute the with call using the internal helper
            $ui.with_layout(&decl, $body)
        }
    };
}
