use display::device::CURRENT_DEVICE;
use display::framebuffer::{Framebuffer, KoboFramebuffer1, KoboFramebuffer2, Pixmap};

use crate::draw::pixmap::ReadonlyPixmap;
use vnc::Rect;

use anyhow::Context;

const FB_DEVICE: &str = "/dev/fb0";

pub fn new_frame_buffer(rotate: i8) -> Box<dyn Framebuffer> {
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

pub fn set_pixel_map_ro(fb: &mut Box<dyn Framebuffer>, delta: &MapDelta, pixmap: &ReadonlyPixmap) {
    #[cfg(feature = "eink_device")]
    {
        for y in 0..pixmap.height {
            for x in 0..pixmap.width {
                let px = x + delta.left;
                let py = y + delta.top;
                let color = pixmap.get_pixel(x, y);
                fb.set_pixel(px, py, color);
            }
        }
    }
}

pub fn set_pixel_map(fb: &mut Box<dyn Framebuffer>, delta: &MapDelta, pixmap: &Pixmap) {
    for y in 0..pixmap.height {
        for x in 0..pixmap.width {
            let color = pixmap.get_pixel(x, y);
            fb.set_pixel(delta.left + x, delta.top + y, color);
        }
    }
}

pub struct MapDelta {
    pub left: u32,
    pub top: u32,
}
