use std::cell::UnsafeCell;
use background_worker::WorkSystem;
use crate::font::{FontHandle, TextGenerator};
use crate::internal_error::{InternalResult};
use std::collections::HashMap;
use clay_layout::{
    color::Color as ClayColor,
    Clay,
    ClayLayoutScope,
    Clay_Dimensions,
    math::Dimensions,
    text::TextConfig,
    Clay_StringSlice, Clay_TextElementConfig,
    Declaration,
    id::Id,
};
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

struct State<'a> {
    bg_worker: WorkSystem,
    layout: Clay,
    text_generator: TextGenerator,
    font_styles: HashMap<FontStyle, FontHandle>,
    active_font: FontHandle,
    layout_scope: Option<UiLayoutScope<'a>>,
    font_size: u32,
    window_size: (usize, usize),
}

impl<'a> State<'a> {
    #[inline(always)]
    fn layout(&mut self) -> &mut UiLayoutScope<'a> {
        unsafe { self.layout_scope.as_mut().unwrap_unchecked() }
    }
}

macro_rules! get_state_mut {
    ($self:expr) => {
        unsafe { &mut *$self.state.get() }
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
        };

        let data = Box::new(Ui {
            state: UnsafeCell::new(state),
        });

        // This is a hack. To be fixed later
        unsafe {
            let raw_ptr = Box::into_raw(data);
            Clay::set_measure_text_function_unsafe(
                Self::measure_text_trampoline,
                raw_ptr as _,
            );
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

    pub fn render(&self) {
        // Placeholder for rendering logic
    }

    pub fn label(&self, text: &str, col: ClayColor) {
        let state = get_state_mut!(self);
        let font_id = state.active_font;
        let font_size = state.font_size;

        let _ = state.text_generator.queue_generate_text(
            text,
            font_size,
            font_id,
            &state.bg_worker,
        );

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


    pub fn begin(&self, window_size: (usize, usize)) {
        let state = get_state_mut!(self);
        state.window_size = window_size;
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

    pub fn end(&self, output: &mut [u32]) {
        let state = get_state_mut!(self);
        let text_generator = &state.text_generator;
        let mut pixmap = Pixmap::new(state.window_size.0 as u32, state.window_size.1 as u32).unwrap();

        let scope = unsafe { state.layout_scope.as_mut().unwrap_unchecked() };

        crate::tiny_skia_renderer::clay_tiny_skia_render(&mut pixmap, scope.end(), text_generator);

        for (index, p) in pixmap.data().chunks_exact(4).enumerate() {
            output[index] = u32::from_le_bytes([p[0], p[1], p[2], p[3]]);
        }


        /*
        for command in scope.end() {
            match command.config {
                RenderCommandConfig::Text(text) => {
                    let text_data = text.text;
                    let font_size = text.font_size as u32;
                    let font_id = text.font_id as FontHandle;

                    if let Some(size) = state.text_generator.get_text(text_data, font_size, font_id) {
                        // Render the text here, e.g., draw it to a texture or directly to the output
                        // This is a placeholder for actual rendering logic
                        println!("Rendering text: {} with size: {:?}", text_data, size);
                    }
                }
                _ => {
                    // Handle other render commands (e.g., images, rectangles, etc.)
                }
            }
        }
         */
    }
}