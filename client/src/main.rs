#![allow(unused)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate flate2;

mod device;
mod framebuffer;
#[macro_use]
mod geom;
mod color;
mod input;
mod settings;
mod touch;
mod config;
mod auth;
mod processing;

use crate::device::CURRENT_DEVICE;
use vnc::Client;

use einkvnc::framebuffer::{Framebuffer, KoboFramebuffer1, KoboFramebuffer2};
use config::Connection;
use clap::ArgMatches;
use anyhow::{Context as ResultExt, Error};

const FB_DEVICE: &str = "/dev/fb0";


fn main() -> Result<(), Error> {
    env_logger::init();
    let args: ArgMatches = einkvnc::config::Config::arguments();
    let config = einkvnc::config::Config::cli(&args);

    let mut vnc = einkvnc::connect(config.connection);
    let mut fb: Box<dyn Framebuffer> = kobo_frame_buffer(config.rotate);
    
    return einkvnc::run(&mut vnc, &mut fb, &config);
}

pub fn kobo_frame_buffer(rotate: i8) -> Box<dyn Framebuffer>{
    let mut fb: Box<dyn Framebuffer> = if CURRENT_DEVICE.mark() != 8 {
        Box::new(
            KoboFramebuffer1::new(FB_DEVICE)
                .context("can't create framebuffer")
                .unwrap(),
        )
    } else {
        Box::new(
            KoboFramebuffer2::new(FB_DEVICE)
                .context("can't create framebuffer")
                .unwrap(),
        )
    };

    #[cfg(feature = "eink_device")]
    {
        fb.set_rotation(rotate).ok();
    }
    fb
}


