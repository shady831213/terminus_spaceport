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
use crate::devices::{PixelFormat, KeyCode, MAX_ABS_SCALE};

impl PixelFormat {
    fn sdl2format(&self) -> PixelFormatEnum {
        match self {
            PixelFormat::RGB565 => PixelFormatEnum::RGB565,
            PixelFormat::RGB888 => PixelFormatEnum::ARGB8888,
        }
    }
}

impl KeyCode {
    fn from_sdl_scancode(scan_code: &Scancode) -> KeyCode {
        match scan_code {
            Scancode::A => KeyCode::A,
            Scancode::B => KeyCode::B,
            Scancode::C => KeyCode::C,
            Scancode::D => KeyCode::D,
            Scancode::E => KeyCode::E,
            Scancode::F => KeyCode::F,
            Scancode::G => KeyCode::G,
            Scancode::H => KeyCode::H,
            Scancode::I => KeyCode::I,
            Scancode::J => KeyCode::J,
            Scancode::K => KeyCode::K,
            Scancode::L => KeyCode::L,
            Scancode::M => KeyCode::M,
            Scancode::N => KeyCode::N,
            Scancode::O => KeyCode::O,
            Scancode::P => KeyCode::P,
            Scancode::Q => KeyCode::Q,
            Scancode::R => KeyCode::R,
            Scancode::S => KeyCode::S,
            Scancode::T => KeyCode::T,
            Scancode::U => KeyCode::U,
            Scancode::V => KeyCode::V,
            Scancode::W => KeyCode::W,
            Scancode::X => KeyCode::X,
            Scancode::Y => KeyCode::Y,
            Scancode::Z => KeyCode::Z,
            Scancode::Num1 => KeyCode::NUM1,
            Scancode::Num2 => KeyCode::NUM2,
            Scancode::Num3 => KeyCode::NUM3,
            Scancode::Num4 => KeyCode::NUM4,
            Scancode::Num5 => KeyCode::NUM5,
            Scancode::Num6 => KeyCode::NUM6,
            Scancode::Num7 => KeyCode::NUM7,
            Scancode::Num8 => KeyCode::NUM8,
            Scancode::Num9 => KeyCode::NUM9,
            Scancode::Num0 => KeyCode::NUM0,
            Scancode::Return => KeyCode::ENTER,
            Scancode::Escape => KeyCode::ESC,
            Scancode::Backspace => KeyCode::BACKSPACE,
            Scancode::Tab => KeyCode::TAB,
            Scancode::Space => KeyCode::SPACE,
            Scancode::Minus => KeyCode::MINUS,
            Scancode::Equals => KeyCode::EQUAL,
            Scancode::LeftBracket => KeyCode::LEFTBRACE,
            Scancode::RightBracket => KeyCode::RIGHTBRACE,
            Scancode::Backslash => KeyCode::BACKSLASH,
            Scancode::NonUsHash => KeyCode::RESERVED,
            Scancode::Semicolon => KeyCode::SEMICOLON,
            Scancode::Apostrophe => KeyCode::APOSTROPHE,
            Scancode::Grave => KeyCode::GRAVE,
            Scancode::Comma => KeyCode::COMMA,
            Scancode::Period => KeyCode::DOT,
            Scancode::Slash => KeyCode::SLASH,
            Scancode::CapsLock => KeyCode::CAPSLOCK,
            Scancode::F1 => KeyCode::F1,
            Scancode::F2 => KeyCode::F2,
            Scancode::F3 => KeyCode::F3,
            Scancode::F4 => KeyCode::F4,
            Scancode::F5 => KeyCode::F5,
            Scancode::F6 => KeyCode::F6,
            Scancode::F7 => KeyCode::F7,
            Scancode::F8 => KeyCode::F8,
            Scancode::F9 => KeyCode::F9,
            Scancode::F10 => KeyCode::F10,
            Scancode::F11 => KeyCode::F11,
            Scancode::F12 => KeyCode::F12,
            Scancode::PrintScreen => KeyCode::PRINT,
            Scancode::ScrollLock => KeyCode::SCROLLLOCK,
            Scancode::Pause => KeyCode::PAUSE,
            Scancode::Insert => KeyCode::INSERT,
            Scancode::Home => KeyCode::HOME,
            Scancode::PageUp => KeyCode::PAGEUP,
            Scancode::Delete => KeyCode::DELETE,
            Scancode::End => KeyCode::END,
            Scancode::PageDown => KeyCode::PAGEDOWN,
            Scancode::Right => KeyCode::RIGHT,
            Scancode::Left => KeyCode::LEFT,
            Scancode::Down => KeyCode::DOWN,
            Scancode::Up => KeyCode::UP,
            Scancode::NumLockClear => KeyCode::NUMLOCK,
            Scancode::KpDivide => KeyCode::KPSLASH,
            Scancode::KpMultiply => KeyCode::KPASTERISK,
            Scancode::KpMinus => KeyCode::KPMINUS,
            Scancode::KpPlus => KeyCode::KPPLUS,
            Scancode::KpEnter => KeyCode::KPENTER,
            Scancode::Kp1 => KeyCode::KP1,
            Scancode::Kp2 => KeyCode::KP2,
            Scancode::Kp3 => KeyCode::KP3,
            Scancode::Kp4 => KeyCode::KP4,
            Scancode::Kp5 => KeyCode::KP5,
            Scancode::Kp6 => KeyCode::KP6,
            Scancode::Kp7 => KeyCode::KP7,
            Scancode::Kp8 => KeyCode::KP8,
            Scancode::Kp9 => KeyCode::KP9,
            Scancode::Kp0 => KeyCode::KP0,
            Scancode::KpPeriod => KeyCode::KPDOT,
            Scancode::NonUsBackslash => KeyCode::RESERVED,
            Scancode::Application => KeyCode::RESERVED,
            Scancode::Power => KeyCode::POWER,
            Scancode::KpEquals => KeyCode::KPEQUAL,
            Scancode::F13 => KeyCode::F13,
            Scancode::F14 => KeyCode::F14,
            Scancode::F15 => KeyCode::F15,
            Scancode::F16 => KeyCode::F16,
            Scancode::F17 => KeyCode::F17,
            Scancode::F18 => KeyCode::F18,
            Scancode::F19 => KeyCode::F19,
            Scancode::F20 => KeyCode::F20,
            Scancode::F21 => KeyCode::F21,
            Scancode::F22 => KeyCode::F22,
            Scancode::F23 => KeyCode::F23,
            Scancode::F24 => KeyCode::F24,
            Scancode::Execute => KeyCode::RESERVED,
            Scancode::Help => KeyCode::HELP,
            Scancode::Menu => KeyCode::MENU,
            Scancode::Select => KeyCode::RESERVED,
            Scancode::Stop => KeyCode::STOP,
            Scancode::Again => KeyCode::AGAIN,
            Scancode::Undo => KeyCode::UNDO,
            Scancode::Cut => KeyCode::CUT,
            Scancode::Copy => KeyCode::COPY,
            Scancode::Paste => KeyCode::PASTE,
            Scancode::Find => KeyCode::FIND,
            Scancode::Mute => KeyCode::MUTE,
            Scancode::VolumeUp => KeyCode::VOLUMEUP,
            Scancode::VolumeDown => KeyCode::VOLUMEDOWN,
            Scancode::KpComma => KeyCode::KPCOMMA,
            Scancode::KpEqualsAS400 => KeyCode::RESERVED,
            Scancode::International1 => KeyCode::RESERVED,
            Scancode::International2 => KeyCode::RESERVED,
            Scancode::International3 => KeyCode::RESERVED,
            Scancode::International4 => KeyCode::RESERVED,
            Scancode::International5 => KeyCode::RESERVED,
            Scancode::International6 => KeyCode::RESERVED,
            Scancode::International7 => KeyCode::RESERVED,
            Scancode::International8 => KeyCode::RESERVED,
            Scancode::International9 => KeyCode::RESERVED,
            Scancode::Lang1 => KeyCode::RESERVED,
            Scancode::Lang2 => KeyCode::RESERVED,
            Scancode::Lang3 => KeyCode::RESERVED,
            Scancode::Lang4 => KeyCode::RESERVED,
            Scancode::Lang5 => KeyCode::RESERVED,
            Scancode::Lang6 => KeyCode::RESERVED,
            Scancode::Lang7 => KeyCode::RESERVED,
            Scancode::Lang8 => KeyCode::RESERVED,
            Scancode::Lang9 => KeyCode::RESERVED,
            Scancode::AltErase => KeyCode::ALTERASE,
            Scancode::SysReq => KeyCode::SYSRQ,
            Scancode::Cancel => KeyCode::CANCEL,
            Scancode::Clear => KeyCode::RESERVED,
            Scancode::Prior => KeyCode::RESERVED,
            Scancode::Return2 => KeyCode::RESERVED,
            Scancode::Separator => KeyCode::RESERVED,
            Scancode::Out => KeyCode::RESERVED,
            Scancode::Oper => KeyCode::RESERVED,
            Scancode::ClearAgain => KeyCode::RESERVED,
            Scancode::CrSel => KeyCode::RESERVED,
            Scancode::ExSel => KeyCode::RESERVED,
            Scancode::Kp00 => KeyCode::RESERVED,
            Scancode::Kp000 => KeyCode::RESERVED,
            Scancode::ThousandsSeparator => KeyCode::RESERVED,
            Scancode::DecimalSeparator => KeyCode::RESERVED,
            Scancode::CurrencyUnit => KeyCode::RESERVED,
            Scancode::CurrencySubUnit => KeyCode::RESERVED,
            Scancode::KpLeftParen => KeyCode::KPLEFTPAREN,
            Scancode::KpRightParen => KeyCode::KPRIGHTPAREN,
            Scancode::KpLeftBrace => KeyCode::RESERVED,
            Scancode::KpRightBrace => KeyCode::RESERVED,
            Scancode::KpTab => KeyCode::RESERVED,
            Scancode::KpBackspace => KeyCode::RESERVED,
            Scancode::KpA => KeyCode::RESERVED,
            Scancode::KpB => KeyCode::RESERVED,
            Scancode::KpC => KeyCode::RESERVED,
            Scancode::KpD => KeyCode::RESERVED,
            Scancode::KpE => KeyCode::RESERVED,
            Scancode::KpF => KeyCode::RESERVED,
            Scancode::KpXor => KeyCode::RESERVED,
            Scancode::KpPower => KeyCode::RESERVED,
            Scancode::KpPercent => KeyCode::RESERVED,
            Scancode::KpLess => KeyCode::RESERVED,
            Scancode::KpGreater => KeyCode::RESERVED,
            Scancode::KpAmpersand => KeyCode::RESERVED,
            Scancode::KpDblAmpersand => KeyCode::RESERVED,
            Scancode::KpVerticalBar => KeyCode::RESERVED,
            Scancode::KpDblVerticalBar => KeyCode::RESERVED,
            Scancode::KpColon => KeyCode::RESERVED,
            Scancode::KpHash => KeyCode::RESERVED,
            Scancode::KpSpace => KeyCode::RESERVED,
            Scancode::KpAt => KeyCode::RESERVED,
            Scancode::KpExclam => KeyCode::RESERVED,
            Scancode::KpMemStore => KeyCode::RESERVED,
            Scancode::KpMemRecall => KeyCode::RESERVED,
            Scancode::KpMemClear => KeyCode::RESERVED,
            Scancode::KpMemAdd => KeyCode::RESERVED,
            Scancode::KpMemSubtract => KeyCode::RESERVED,
            Scancode::KpMemMultiply => KeyCode::RESERVED,
            Scancode::KpMemDivide => KeyCode::RESERVED,
            Scancode::KpPlusMinus => KeyCode::KPPLUSMINUS,
            Scancode::KpClear => KeyCode::RESERVED,
            Scancode::KpClearEntry => KeyCode::RESERVED,
            Scancode::KpBinary => KeyCode::RESERVED,
            Scancode::KpOctal => KeyCode::RESERVED,
            Scancode::KpDecimal => KeyCode::RESERVED,
            Scancode::KpHexadecimal => KeyCode::RESERVED,
            Scancode::LCtrl => KeyCode::LEFTCTRL,
            Scancode::LShift => KeyCode::LEFTSHIFT,
            Scancode::LAlt => KeyCode::LEFTALT,
            Scancode::LGui => KeyCode::LEFTMETA,
            Scancode::RCtrl => KeyCode::RIGHTCTRL,
            Scancode::RShift => KeyCode::RIGHTSHIFT,
            Scancode::RAlt => KeyCode::RIGHTALT,
            Scancode::RGui => KeyCode::RIGHTMETA,
            Scancode::Mode => KeyCode::RESERVED,
            Scancode::AudioNext => KeyCode::NEXTSONG,
            Scancode::AudioPrev => KeyCode::PREVIOUSSONG,
            Scancode::AudioStop => KeyCode::STOPCD,
            Scancode::AudioPlay => KeyCode::PLAYCD,
            Scancode::AudioMute => KeyCode::MICMUTE,
            Scancode::MediaSelect => KeyCode::MEDIA,
            Scancode::Www => KeyCode::WWW,
            Scancode::Mail => KeyCode::MAIL,
            Scancode::Calculator => KeyCode::CALC,
            Scancode::Computer => KeyCode::COMPUTER,
            Scancode::AcSearch => KeyCode::SEARCH,
            Scancode::AcHome => KeyCode::HOMEPAGE,
            Scancode::AcBack => KeyCode::BACK,
            Scancode::AcForward => KeyCode::FORWARD,
            Scancode::AcStop => KeyCode::RESERVED,
            Scancode::AcRefresh => KeyCode::REFRESH,
            Scancode::AcBookmarks => KeyCode::BOOKMARKS,
            Scancode::BrightnessDown => KeyCode::BRIGHTNESSDOWN,
            Scancode::BrightnessUp => KeyCode::BRIGHTNESSUP,
            Scancode::DisplaySwitch => KeyCode::ROTATEDISPLAY,
            Scancode::KbdIllumToggle => KeyCode::KBDILLUMTOGGLE,
            Scancode::KbdIllumDown => KeyCode::KBDILLUMDOWN,
            Scancode::KbdIllumUp => KeyCode::KBDILLUMUP,
            Scancode::Eject => KeyCode::EJECTCD,
            Scancode::Sleep => KeyCode::SLEEP,
            Scancode::App1 => KeyCode::RESERVED,
            Scancode::App2 => KeyCode::RESERVED,
            Scancode::Num => KeyCode::RESERVED,
        }
    }
}

pub struct SDL {
    event_pump: RefCell<EventPump>,
    window: Window,
    width: u32,
    height: u32,
    key_pressed: RefCell<[bool; 256]>,
    quit: Box<dyn Fn() + 'static>,
}

impl SDL {
    pub fn new<QF: Fn() + 'static>(title: &str, width: u32, height: u32, format: PixelFormat, quit: QF) -> Result<SDL, String> {
        let context = sdl2::init()?;
        let video_subsystem = context.video()?;
        let mut window = video_subsystem.window(title, width, height)
            .position_centered()
            .build()
            .map_err(|e| e.to_string())?;
        window.set_display_mode(DisplayMode::new(format.sdl2format(), width as i32, height as i32, 60))?;
        let event_pump = context.event_pump()?;
        let mut screen = window.surface(&event_pump)?;
        screen.fill_rect(Rect::new(0, 0, width, height), Color::BLACK)?;
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
        if key_pressed[idx as usize] {
            key_pressed[idx as usize] = false;
            input.send_key_event(false, idx)
        }
    }

    fn key_down<I: KeyBoard>(&self, input: &I, code: &Scancode) {
        let idx = self.get_key(code);
        let mut key_pressed = self.key_pressed.borrow_mut();
        key_pressed[idx as usize] = true;
        input.send_key_event(true, idx)
    }

    fn get_key(&self, code: &Scancode) -> u16 {
        KeyCode::from_sdl_scancode(code) as u16
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
        x * MAX_ABS_SCALE / self.width as i32
    }

    fn mouse_y_abs(&self, y: i32) -> i32 {
        y * MAX_ABS_SCALE / self.height as i32
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
