extern crate sdl2;

use self::sdl2::Sdl;
use self::sdl2::render::{WindowCanvas, Texture};
use crate::devices::{FrameBuffer, Display};
use std::cell::RefCell;
use self::sdl2::rect::Rect;
use self::sdl2::event::Event;
use crate::devices::display::{DisplayInput, MOUSE_BTN_LEFT, MOUSE_BTN_RIGHT, MOUSE_BTN_MIDDLE};
use self::sdl2::keyboard::Keycode;
use self::sdl2::mouse::{MouseButton, MouseState, MouseWheelDirection};
pub use self::sdl2::*;

pub trait SDLFrameBuffer: FrameBuffer {
    fn texture(&self) -> Result<&Texture, String>;
}

pub struct SDL<FB: SDLFrameBuffer, I: DisplayInput> {
    context: Sdl,
    canvas: RefCell<WindowCanvas>,
    width: usize,
    height: usize,
    fb: FB,
    key_pressed: RefCell<[bool; 256]>,
    quit: Box<dyn Fn() + 'static>,
    input: I,
}

impl<FB: SDLFrameBuffer, I: DisplayInput> SDL<FB, I> {
    pub fn new<QF: Fn() + 'static>(title: &str, fb: FB, width: usize, height: usize, quit: QF, input: I) -> Result<SDL<FB, I>, String> {
        let context = sdl2::init()?;
        let video_subsystem = context.video()?;
        let window = video_subsystem.window(title, width as u32, height as u32)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;

        let canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
        Ok(SDL {
            context,
            canvas: RefCell::new(canvas),
            width,
            height,
            fb,
            key_pressed: RefCell::new([false; 256]),
            quit: Box::new(quit),
            input,
        })
    }

    fn key_up(&self, code: &Keycode) {
        let idx = *code as i32 as usize;
        let mut key_pressed = self.key_pressed.borrow_mut();
        if (*key_pressed)[idx] {
            (*key_pressed)[idx] = false;
            self.input.send_key_event(false, idx as u32)
        }
    }

    fn key_down(&self, code: &Keycode) {
        let idx = *code as i32 as usize;
        (*self.key_pressed.borrow_mut())[idx] = true;
        self.input.send_key_event(true, idx as u32)
    }

    fn mouse_motion(&self, state: &MouseState, x: i32, y: i32, xrel: i32, yrel: i32) {
        if self.input.mouse_absolute() {
            self.input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), 0, state.to_sdl_state())
        } else {
            self.input.send_mouse_event(xrel, yrel, 0, state.to_sdl_state())
        }
    }

    fn mouse_button_down(&self, x: i32, y: i32, sdl_btn: &MouseButton) {
        let btn = self.mouse_btn(sdl_btn);
        if self.input.mouse_absolute() {
            self.input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), 0, btn)
        } else {
            self.input.send_mouse_event(0, 0, 0, btn)
        }
    }

    fn mouse_button_up(&self, x: i32, y: i32) {
        if self.input.mouse_absolute() {
            self.input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), 0, 0)
        } else {
            self.input.send_mouse_event(0, 0, 0, 0)
        }
    }

    fn mouse_wheel(&self, x: i32, y: i32, dir: &MouseWheelDirection) {
        if self.input.mouse_absolute() {
            self.input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), self.mouse_z(dir), 0)
        } else {
            self.input.send_mouse_event(0, 0, self.mouse_z(dir), 0)
        }
    }

    fn mouse_z(&self, dir: &MouseWheelDirection) -> i32 {
        match dir {
            MouseWheelDirection::Normal => 1,
            MouseWheelDirection::Flipped => -1,
            _ => 0
        }
    }

    fn mouse_btn(&self, sdl_btn: &MouseButton) -> u32 {
        match sdl_btn {
            MouseButton::Left => MOUSE_BTN_LEFT,
            MouseButton::Right => MOUSE_BTN_RIGHT,
            MouseButton::Middle => MOUSE_BTN_MIDDLE,
            _ => 0,
        }
    }


    fn mouse_x_abs(&self, x: i32) -> i32 {
        x * 32768 / self.width as i32
    }

    fn mouse_y_abs(&self, y: i32) -> i32 {
        y * 32768 / self.height as i32
    }


    pub fn refresh(&self) -> Result<(), String> {
        self.fb.refresh(self)?;
        let mut event_pump = self.context.event_pump()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    (*&self.quit)()
                }
                Event::KeyDown { keycode: Some(ref code), .. } => { self.key_down(code) }
                Event::KeyUp { keycode: Some(ref code), .. } => { self.key_up(code) }
                Event::MouseMotion { mousestate: ref state, x,y, xrel, yrel, .. } => { self.mouse_motion(state, x, y, xrel, yrel) }
                Event::MouseButtonDown { x,y, mouse_btn: ref btn, .. } => { self.mouse_button_down(x, y, btn) }
                Event::MouseButtonUp { x,y, .. } => { self.mouse_button_up(x, y) }
                Event::MouseWheel { x,y, direction: ref dir, .. } => { self.mouse_wheel(x, y, dir) }
                _ => {}
            }
        }
        Ok(())
    }
}

impl<FB: SDLFrameBuffer, I: DisplayInput> Display for SDL<FB, I> {
    fn draw(&self, x: i32, y: i32, w: u32, h: u32) -> Result<(), String> {
        let texture = self.fb.texture()?;
        let mut canvas = self.canvas.borrow_mut();
        let rect = Rect::new(x, y, w, h);
        canvas.clear();
        canvas.copy(texture, Some(rect), Some(rect))?;
        canvas.present();
        Ok(())
    }
}