extern crate sdl2;

use self::sdl2::EventPump;
use std::cell::RefCell;
use self::sdl2::rect::Rect;
use self::sdl2::event::Event;
use crate::devices::display::{FrameBuffer, KeyBoard, Mouse, MOUSE_BTN_LEFT, MOUSE_BTN_RIGHT, MOUSE_BTN_MIDDLE};
use self::sdl2::keyboard::Scancode;
use self::sdl2::mouse::{MouseButton, MouseState, MouseWheelDirection, Cursor};
use self::sdl2::surface::Surface;
use self::sdl2::pixels::{PixelFormatEnum, Color};
use self::sdl2::video::{Window, DisplayMode};
use crate::devices::PixelFormat;

impl PixelFormat {
    fn sdl2format(&self) -> PixelFormatEnum {
        match self {
            PixelFormat::RGB565 => PixelFormatEnum::RGB565,
            PixelFormat::RGB888 => PixelFormatEnum::ARGB8888,
        }
    }
}

pub struct SDL {
    event_pump: RefCell<EventPump>,
    window: Window,
    width: usize,
    height: usize,
    key_pressed: RefCell<[bool; 256]>,
    quit: Box<dyn Fn() + 'static>,
}

impl SDL {
    pub fn new<QF: Fn() + 'static>(title: &str, width: usize, height: usize, format: PixelFormat, quit: QF) -> Result<SDL, String> {
        let context = sdl2::init()?;
        let video_subsystem = context.video()?;
        let mut window = video_subsystem.window(title, width as u32, height as u32)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        window.set_display_mode(DisplayMode::new(format.sdl2format(), width as i32, height as i32, 60))?;
        let event_pump = context.event_pump()?;
        let mut screen = window.surface(&event_pump)?;
        screen.fill_rect(Rect::new(0, 0, width as u32, height as u32), Color::BLACK)?;
        screen.update_window()?;
        let cursor_data = vec![0; 1];
        let cursor = Cursor::new(&cursor_data, &cursor_data, 8, 1, 0, 0)?;
        cursor.set();
        Ok(SDL {
            event_pump: RefCell::new(event_pump),
            window,
            width,
            height,
            key_pressed: RefCell::new([false; 256]),
            quit: Box::new(quit),
        })
    }

    fn key_up<I: KeyBoard>(&self, input: &I, code: &Scancode) {
        let idx = self.get_key(code);
        let mut key_pressed = self.key_pressed.borrow_mut();
        if key_pressed[idx] {
            key_pressed[idx] = false;
            input.send_key_event(false, idx as u16)
        }
    }

    fn key_down<I: KeyBoard>(&self, input: &I, code: &Scancode) {
        let idx = self.get_key(code);
        let mut key_pressed = self.key_pressed.borrow_mut();
        key_pressed[idx] = true;
        input.send_key_event(true, idx as u16)
    }

    fn get_key(&self, code: &Scancode) -> usize {
        let scan_node = *code as i32;
        if scan_node < 9 {
            0
        } else if scan_node < 255 + 8 {
            (scan_node - 8) as usize
        } else {
            0
        }
    }

    fn mouse_motion<I: Mouse>(&self, input: &I, state: &MouseState, x: i32, y: i32, xrel: i32, yrel: i32) {
        if input.mouse_absolute() {
            input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), 0, state.to_sdl_state())
        } else {
            input.send_mouse_event(xrel, yrel, 0, state.to_sdl_state())
        }
    }

    fn mouse_button_down<I: Mouse>(&self, input: &I, x: i32, y: i32, sdl_btn: &MouseButton) {
        let btn = self.mouse_btn(sdl_btn);
        if input.mouse_absolute() {
            input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), 0, btn)
        } else {
            input.send_mouse_event(0, 0, 0, btn)
        }
    }

    fn mouse_button_up<I: Mouse>(&self, input: &I, x: i32, y: i32) {
        if input.mouse_absolute() {
            input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), 0, 0)
        } else {
            input.send_mouse_event(0, 0, 0, 0)
        }
    }

    fn mouse_wheel<I: Mouse>(&self, input: &I, x: i32, y: i32, dir: &MouseWheelDirection) {
        if input.mouse_absolute() {
            input.send_mouse_event(self.mouse_x_abs(x), self.mouse_y_abs(y), self.mouse_z(dir), 0)
        } else {
            input.send_mouse_event(0, 0, self.mouse_z(dir), 0)
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


    pub fn refresh<FB: FrameBuffer, K: KeyBoard, M: Mouse>(&self, fb: &FB, k: &K, m: &M) -> Result<(), String> {
        let mut event_pump = self.event_pump.borrow_mut();
        let mut data = fb.data();
        let surface = Surface::from_data(&mut data, fb.width(), fb.height(), fb.stride(), fb.pixel_format().sdl2format())?;
        let screen = self.window.surface(&event_pump)?;
        fb.refresh(|x, y, w, h| {
            let rect = Rect::new(x, y, w, h);
            let mut s = self.window.surface(&event_pump)?;
            unsafe { surface.lower_blit(rect, &mut s, rect) }?;
            Ok(())
        })?;
        screen.update_window()?;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    (*&self.quit)()
                }
                Event::KeyDown { scancode: Some(ref code), .. } => { self.key_down(k, code) }
                Event::KeyUp { scancode: Some(ref code), .. } => { self.key_up(k, code) }
                Event::MouseMotion { mousestate: ref state, x, y, xrel, yrel, .. } => { self.mouse_motion(m, state, x, y, xrel, yrel) }
                Event::MouseButtonDown { x, y, mouse_btn: ref btn, .. } => { self.mouse_button_down(m, x, y, btn) }
                Event::MouseButtonUp { x, y, .. } => { self.mouse_button_up(m, x, y) }
                Event::MouseWheel { x, y, direction: ref dir, .. } => { self.mouse_wheel(m, x, y, dir) }
                _ => {}
            }
        }
        Ok(())
    }
}
