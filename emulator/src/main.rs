#![allow(unused)]

#[macro_use]
extern crate log;

use std::mem;
use std::thread;
use std::fs::File;
use std::sync::mpsc;
use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;
use anyhow::{Context as ResultExt, Error};
use einkvnc::config::Config;
// use plato_core::anyhow::{Error, Context as ResultExt};
use chrono::Local;
use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::{Scancode, Keycode, Mod};
use sdl2::render::{WindowCanvas, BlendMode};
use sdl2::pixels::{Color as SdlColor, PixelFormatEnum};
use sdl2::mouse::MouseState;
use sdl2::rect::Point as SdlPoint;
use sdl2::rect::Rect as SdlRect;
use einkvnc::framebuffer::{Framebuffer, UpdateMode};
use einkvnc::input::{DeviceEvent, FingerStatus, ButtonCode, ButtonStatus};
// use plato_core::document::sys_info_as_html;
// use plato_core::view::{View, Event, ViewId, EntryId, AppCmd, EntryKind};
// use plato_core::view::{process_render_queue, wait_for_all, handle_event, RenderQueue, RenderData};
// use plato_core::view::home::Home;
// use plato_core::view::reader::Reader;
// use plato_core::view::notification::Notification;
// use plato_core::view::dialog::Dialog;
// use plato_core::view::frontlight::FrontlightWindow;
// use plato_core::view::menu::{Menu, MenuKind};
// use plato_core::view::intermission::Intermission;
// use plato_core::view::dictionary::Dictionary;
// use plato_core::view::calculator::Calculator;
// use plato_core::view::sketch::Sketch;
// use plato_core::view::touch_events::TouchEvents;
// use plato_core::view::rotation_values::RotationValues;
// use plato_core::view::common::{locate, locate_by_id, transfer_notifications, overlapping_rectangle};
// use plato_core::view::common::{toggle_input_history_menu, toggle_keyboard_layout_menu};
// use plato_core::helpers::{load_toml, save_toml};
// use einkvnc::settings::{Settings, SETTINGS_PATH, IntermKind};
use einkvnc::geom::{Rectangle, Axis};
//use plato_core::gesture::{GestureEvent, gesture_events};
use einkvnc::device::CURRENT_DEVICE;
// use plato_core::battery::{Battery, FakeBattery};
// use plato_core::frontlight::{Frontlight, LightLevels};
// use plato_core::lightsensor::LightSensor;
// use plato_core::library::Library;
// use plato_core::font::Fonts;
// use plato_core::context::Context;
// use plato_core::pt;
// use plato_core::png;
//use clap::ArgMatches;

pub const APP_NAME: &str = "Plato";
const DEFAULT_ROTATION: i8 = 1;

const CLOCK_REFRESH_INTERVAL: Duration = Duration::from_secs(60);

// pub fn build_context(fb: Box<dyn Framebuffer>) -> Result<Context, Error> {
//     let settings = load_toml::<Settings, _>(SETTINGS_PATH)?;
//     let library_settings = &settings.libraries[settings.selected_library];
//     let library = Library::new(&library_settings.path, library_settings.mode)?;

//     let battery = Box::new(FakeBattery::new()) as Box<dyn Battery>;
//     let frontlight = Box::new(LightLevels::default()) as Box<dyn Frontlight>;
//     let lightsensor = Box::new(0u16) as Box<dyn LightSensor>;
//     let fonts = Fonts::load()?;

//     Ok(Context::new(fb, None, library, settings,
//                     fonts, battery, frontlight, lightsensor))
// }

#[inline]
fn seconds(timestamp: u32) -> f64 {
    timestamp as f64 / 1000.0
}

#[inline]
pub fn device_event(event: SdlEvent) -> Option<DeviceEvent> {
    match event {
        // SdlEvent::MouseButtonDown { timestamp, x, y, .. } =>
        //     Some(DeviceEvent::Finger { id: 0,
        //                                status: FingerStatus::Down,
        //                                position: pt!(x, y),
        //                                time: seconds(timestamp) }),
        // SdlEvent::MouseButtonUp { timestamp, x, y, .. } =>
        //     Some(DeviceEvent::Finger { id: 0,
        //                                status: FingerStatus::Up,
        //                                position: pt!(x, y),
        //                                time: seconds(timestamp) }),
        // SdlEvent::MouseMotion { timestamp, x, y, .. } =>
        //     Some(DeviceEvent::Finger { id: 0,
        //                                status: FingerStatus::Motion,
        //                                position: pt!(x, y),
        //                                time: seconds(timestamp) }),
        _ => None,
    }
}

fn code_from_key(key: Scancode) -> Option<ButtonCode> {
    match key {
        Scancode::B => Some(ButtonCode::Backward),
        Scancode::F => Some(ButtonCode::Light),
        Scancode::H => Some(ButtonCode::Home),
        Scancode::E => Some(ButtonCode::Erase),
        Scancode::G => Some(ButtonCode::Highlight),
        _ => None,
    }
}

struct FBCanvas(WindowCanvas);

impl Framebuffer for FBCanvas {
    fn set_pixel(&mut self, x: u32, y: u32, color: u8) {
        self.0.set_draw_color(SdlColor::RGB(color, color, color));
        self.0.draw_point(SdlPoint::new(x as i32, y as i32)).unwrap();
    }

    fn set_blended_pixel(&mut self, x: u32, y: u32, color: u8, alpha: f32) {
        debug!("set blended pixel {}/{}", x, y);
        self.0.set_draw_color(SdlColor::RGBA(color, color, color, (alpha * 255.0) as u8));
        self.0.draw_point(SdlPoint::new(x as i32, y as i32)).unwrap();
    }

    fn invert_region(&mut self, rect: &Rectangle) {
        let width = rect.width();
        let s_rect = Some(SdlRect::new(rect.min.x, rect.min.y,
                                       width, rect.height()));
        if let Ok(data) = self.0.read_pixels(s_rect, PixelFormatEnum::RGB24) {
            for y in rect.min.y..rect.max.y {
                let v = (y - rect.min.y) as u32;
                for x in rect.min.x..rect.max.x {
                    let u = (x - rect.min.x) as u32;
                    let addr = 3 * (v * width + u);
                    let color = 255 - data[addr as usize];
                    self.set_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    fn shift_region(&mut self, rect: &Rectangle, drift: u8) {
        let width = rect.width();
        let s_rect = Some(SdlRect::new(rect.min.x, rect.min.y,
                                       width, rect.height()));
        if let Ok(data) = self.0.read_pixels(s_rect, PixelFormatEnum::RGB24) {
            for y in rect.min.y..rect.max.y {
                let v = (y - rect.min.y) as u32;
                for x in rect.min.x..rect.max.x {
                    let u = (x - rect.min.x) as u32;
                    let addr = 3 * (v * width + u);
                    let color = data[addr as usize].saturating_sub(drift);
                    self.set_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    fn update(&mut self, _rect: &Rectangle, _mode: UpdateMode) -> Result<u32, Error> {
        debug!("update {:}", _rect);
        self.0.present();
        Ok(Local::now().timestamp_subsec_millis())
    }

    fn wait(&self, _tok: u32) -> Result<i32, Error> {
        Ok(1)
    }

    fn save(&self, path: &str) -> Result<(), Error> {
        let (width, height) = self.dims();
        //let file = File::create(path).with_context(|| format!("can't create output file {}", path))?;
        // let mut encoder = png::Encoder::new(file, width, height);
        // encoder.set_depth(png::BitDepth::Eight);
        // encoder.set_color(png::ColorType::Rgb);
        // let mut writer = encoder.write_header().with_context(|| format!("can't write PNG header for {}", path))?;
        // let data = self.0.read_pixels(self.0.viewport(), PixelFormatEnum::RGB24).unwrap_or_default();
        // writer.write_image_data(&data).with_context(|| format!("can't write PNG data to {}", path))?;
        Ok(())
    }

    fn rotation(&self) -> i8 {
        DEFAULT_ROTATION
    }

    fn set_rotation(&mut self, n: i8) -> Result<(u32, u32), Error> {
        let (mut width, mut height) = self.dims();
        if (width < height && n % 2 == 0) || (width > height && n % 2 == 1) {
            mem::swap(&mut width, &mut height);
        }
        self.0.window_mut().set_size(width, height).ok();
        Ok((width, height))
    }

    fn get_pixel(&self, x: u32, y: u32) -> u8 {
        debug!("virtualfb: get pixel {}/{}", x, y);
        1
    }

    fn set_monochrome(&mut self, _enable: bool) {
        debug!("set mono {}", _enable)
    }

    fn set_dithered(&mut self, _enable: bool) {
        debug!("set dither {}", _enable)
    }

    fn set_inverted(&mut self, _enable: bool) {
        debug!("set invert {}", _enable)
    }

    fn monochrome(&self) -> bool {
        false
    }

    fn dithered(&self) -> bool {
        false
    }

    fn inverted(&self) -> bool {
        false
    }

    fn width(&self) -> u32 {
        self.0.window().size().0
    }

    fn height(&self) -> u32 {
        self.0.window().size().1
    }
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let config = Config{
        connection: einkvnc::config::Connection { host: "localhost", port: 5901, username: None, password: Some("123456"), exclusive: false },
        processing: einkvnc::processing::PostProcConfig { contrast_exp: 1.1, contrast_gray_point: 224.0, white_cutoff: 225 },
        rotate: 1,
        view_only: true,
        touch_input: "/dev/oblivion".to_string(),
    };
    let mut vnc = einkvnc::connect(config.connection);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let (width, height) = CURRENT_DEVICE.dims;
    let window = video_subsystem
                 .window("EinkVNC Emulator", width, height)
                 .position_centered()
                 .build()
                 .unwrap();

    let mut fb: sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().software().build().unwrap();
    fb.set_blend_mode(BlendMode::Blend);

    println!("{} is running on a Kobo {}.", APP_NAME,
                                            CURRENT_DEVICE.model);

    let mut vnc_fb: Box<dyn Framebuffer> = Box::new(FBCanvas(fb));
    einkvnc::run(&mut vnc, &mut vnc_fb, &config)
}
