pub trait Display {
    fn draw(&self, x: i32, y: i32, w: u32, h: u32) -> Result<(), String>;
}

pub trait FrameBuffer {
    fn refresh<D: Display>(&self, d: &D) -> Result<(), String>;
}

pub const MOUSE_BTN_LEFT:u32 = 0x1;
pub const MOUSE_BTN_RIGHT:u32 = 0x2;
pub const MOUSE_BTN_MIDDLE:u32 = 0x3;


pub trait DisplayInput {
    fn send_key_event(&self, key_down: bool, val: u32);
    fn send_mouse_event(&self, x: i32, y: i32, z: i32, buttons: u32);
    fn mouse_absolute(&self)->bool;
}