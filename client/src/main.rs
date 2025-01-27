#[macro_use]
extern crate log;
extern crate byteorder;
extern crate flate2;

mod config;
mod processing;
mod kobo;
mod vnc;

use display::framebuffer::Framebuffer;
use clap::ArgMatches;
use anyhow::Error;

fn main() -> Result<(), Error> {
    env_logger::init();
    let args: ArgMatches = einkvnc::config::Config::arguments();
    let config = einkvnc::config::Config::cli(&args);

    let mut vnc = einkvnc::vnc::connect(config.connection);
    let mut fb: Box<dyn Framebuffer> = kobo::new_frame_buffer(config.rotate);
    
    return einkvnc::run(&mut vnc, &mut fb, &config);
}
