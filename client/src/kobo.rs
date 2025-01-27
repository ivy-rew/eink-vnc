use display::device::CURRENT_DEVICE;
use display::framebuffer::{Framebuffer, KoboFramebuffer1, KoboFramebuffer2};

use anyhow::{Context as ResultExt, Error};

const FB_DEVICE: &str = "/dev/fb0";

pub fn new_frame_buffer(rotate: i8) -> Box<dyn Framebuffer>{
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
