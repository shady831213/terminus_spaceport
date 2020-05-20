use std::cell::RefMut;

// pub trait Display {
//     fn draw(&self, data:&mut [u8], fb_width:u32, fb_height:u32, fb_stride:u32, x: i32, y: i32, w: u32, h: u32) -> Result<(), String>;
// }

pub trait FrameBuffer {
    fn refresh<DRAW: FnMut(i32, i32, u32, u32)->Result<(), String>>(&self, d: DRAW) -> Result<(), String>;
    fn data(&self)->RefMut<'_, Vec<u8>>;
    fn width(&self)->u32;
    fn height(&self)->u32;
    fn stride(&self)->u32;

}

pub const MOUSE_BTN_LEFT:u32 = 0x1;
pub const MOUSE_BTN_RIGHT:u32 = 0x2;
pub const MOUSE_BTN_MIDDLE:u32 = 0x3;


pub trait KeyBoard {
    fn send_key_event(&self, key_down: bool, val: u32);
}

pub trait Mouse{
    fn send_mouse_event(&self, x: i32, y: i32, z: i32, buttons: u32);
    fn mouse_absolute(&self)->bool;
}