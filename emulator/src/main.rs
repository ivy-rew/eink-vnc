#![allow(unused)]

#[macro_use]
extern crate log;

mod localbuffer;

use std::mem;
use std::thread;
use std::fs::File;
use std::sync::mpsc;
use std::collections::VecDeque;
use std::path::Path;
use std::time::Duration;
use anyhow::{Context as ResultExt, Error};
use einkvnc::config::Config;
use localbuffer::FBCanvas;
use chrono::Local;
use sdl2::event::Event as SdlEvent;
use sdl2::keyboard::{Scancode, Keycode, Mod};
use sdl2::mouse::MouseState;
use einkvnc::framebuffer::{Framebuffer, UpdateMode};
use einkvnc::input::{DeviceEvent, FingerStatus, ButtonCode, ButtonStatus};
use einkvnc::geom::{Rectangle, Axis};
//use plato_core::gesture::{GestureEvent, gesture_events};
use einkvnc::device::CURRENT_DEVICE;
// use plato_core::font::Fonts;
// use plato_core::pt;
// use plato_core::png;
//use clap::ArgMatches;

pub const APP_NAME: &str = "EinkVNC Emulator";

const CLOCK_REFRESH_INTERVAL: Duration = Duration::from_secs(60);

fn main() -> Result<(), Error> {
    env_logger::init();

    let config = local_config();
    let mut vnc = einkvnc::connect(config.connection);

    let (width, height) = CURRENT_DEVICE.dims;
    let mut vnc_fb: Box<dyn Framebuffer> = localbuffer::new(APP_NAME, width, height);
    println!("{} is running on a Kobo {}.", APP_NAME, CURRENT_DEVICE.model);

    ctrlc::set_handler(move || {
        println!("received Ctrl+C!");
        std::process::exit(1);
    }).expect("Error setting Ctrl-C handler");

    einkvnc::run(&mut vnc, &mut vnc_fb, &config)
}

fn local_config() -> Config<'static> {
    Config{
        connection: einkvnc::config::Connection { host: "localhost", port: 5901, username: None, password: Some("123456"), exclusive: false },
        processing: einkvnc::processing::PostProcConfig { contrast_exp: 1.1, contrast_gray_point: 224.0, white_cutoff: 225 },
        rotate: 1,
        view_only: true,
        touch_input: "/dev/oblivion".to_string(),
    }
}

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

