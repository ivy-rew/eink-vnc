#![allow(unused)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate flate2;

mod config;
mod processing;

use display::device::CURRENT_DEVICE;
use display::framebuffer::{Framebuffer, KoboFramebuffer1, KoboFramebuffer2};
use config::Connection;
use clap::ArgMatches;
use anyhow::{Context as ResultExt, Error};

const FB_DEVICE: &str = "/dev/fb0";


fn main() -> Result<(), Error> {
    env_logger::init();
    let args: ArgMatches = einkvnc::config::Config::arguments();
    let config = einkvnc::config::Config::cli(&args);

    let mut vnc = einkvnc::vnc::connect(config.connection);
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

    #[cfg(feature = "eink_device")]
    debug!(
        "running on device model=\"{}\" /dpi={} /dims={}x{}", 
        CURRENT_DEVICE.model,
        CURRENT_DEVICE.dpi,
        CURRENT_DEVICE.dims.0,
        CURRENT_DEVICE.dims.1
    );

    fb
}


