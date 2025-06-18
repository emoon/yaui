use crate::bindings::*;

#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(C)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b, a: 255.0 }
    }
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Allows using hex values to build colors
    /// ```
    /// use clay_layout::color::Color;
    /// assert_eq!(Color::rgb(255.0, 255.0, 255.0), Color::u_rgb(0xFF, 0xFF, 0xFF));
    /// ```
    pub const fn u_rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgb(r as _, g as _, b as _)
    }
    /// Allows using hex values to build colors
    /// ```
    /// use clay_layout::color::Color;
    /// assert_eq!(Color::rgba(255.0, 255.0, 255.0, 255.0), Color::u_rgba(0xFF, 0xFF, 0xFF, 0xFF));
    /// ```
    pub const fn u_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::rgba(r as _, g as _, b as _, a as _)
    }
}

impl From<Clay_Color> for Color {
    fn from(value: Clay_Color) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}
impl From<Color> for Clay_Color {
    fn from(value: Color) -> Self {
        unsafe { core::mem::transmute(value) }
    }
}

impl From<(f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32)) -> Self {
        Self::rgb(value.0, value.1, value.2)
    }
}
impl From<(f32, f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::rgba(value.0, value.1, value.2, value.3)
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Self::u_rgb(value.0, value.1, value.2)
    }
}
impl From<(u8, u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8, u8)) -> Self {
        Self::u_rgba(value.0, value.1, value.2, value.3)
    }
}
