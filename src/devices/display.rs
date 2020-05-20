use std::cell::RefMut;

pub enum PixelFormat {
    RGB565,
    RGB888,
}

impl PixelFormat {
    pub fn size(&self) -> u32 {
        match self {
            PixelFormat::RGB565 => 2,
            PixelFormat::RGB888 => 4,
        }
    }
}

pub trait FrameBuffer {
    fn refresh<DRAW: Fn(i32, i32, u32, u32) -> Result<(), String>>(&self, d: DRAW) -> Result<(), String>;
    fn data(&self) -> RefMut<'_, Vec<u8>>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn stride(&self) -> u32;
    fn pixel_format(&self) -> &PixelFormat;
}

pub const MOUSE_BTN_LEFT: u32 = 0x1;
pub const MOUSE_BTN_RIGHT: u32 = 0x2;
pub const MOUSE_BTN_MIDDLE: u32 = 0x3;


pub trait KeyBoard {
    fn send_key_event(&self, key_down: bool, val: u32);
}

pub trait Mouse {
    fn send_mouse_event(&self, x: i32, y: i32, z: i32, buttons: u32);
    fn mouse_absolute(&self) -> bool;
}