#![allow(unused)]

#[macro_use]
extern crate log;

mod localbuffer;

use std::time::Duration;
use anyhow::{Context as ResultExt, Error};
use einkvnc::config::Config;
use localbuffer::FBCanvas;
use einkvnc::framebuffer::Framebuffer;
use einkvnc::device::CURRENT_DEVICE;

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

