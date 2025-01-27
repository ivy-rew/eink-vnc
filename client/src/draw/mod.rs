#![allow(unused)]

pub mod kobo;
pub mod pixmap;
pub mod util;

use display::device::CURRENT_DEVICE;
use display::framebuffer::{Framebuffer, Pixmap, UpdateMode};
use display::geom::Rectangle;
use display::rect;
use std::time::Instant;
use vnc::Rect;

const MAX_DIRTY_REFRESHES: usize = 500;
pub struct Draw {
    pub dirty_rects: Vec<Rectangle>,
    pub dirty_rects_since_refresh: Vec<Rectangle>,
    pub has_drawn_once: bool,
    pub dirty_update_count: usize,
    pub time_at_last_draw: Instant,
}

impl Draw {
    pub fn new() -> Draw {
        return Draw {
            dirty_rects: Vec::<Rectangle>::new(),
            dirty_rects_since_refresh: Vec::<Rectangle>::new(),
            has_drawn_once: false,
            dirty_update_count: 0,
            time_at_last_draw: Instant::now(),
        };
    }
}

pub fn update(fb: &mut Box<dyn Framebuffer>, fb_rect: Rectangle, draw: &mut Draw) {
    draw.dirty_rects.clear();
    draw.dirty_rects_since_refresh.clear();
    #[cfg(feature = "eink_device")]
    {
        if !draw.has_drawn_once || draw.dirty_update_count > MAX_DIRTY_REFRESHES {
            fb.update(&fb_rect, UpdateMode::Full).ok();
            draw.dirty_update_count = 0;
            draw.has_drawn_once = true;
        } else {
            fb.update(&fb_rect, UpdateMode::Partial).ok();
        }
    }
}

pub fn refresh(fb: &mut Box<dyn Framebuffer>, draw: &mut Draw) {
    for dr in &draw.dirty_rects_since_refresh {
        #[cfg(feature = "eink_device")]
        {
            fb.update(&dr, UpdateMode::Full).ok();
        }
    }
    draw.dirty_update_count = 0;
    draw.dirty_rects_since_refresh.clear();
}

pub fn draw_end(fb: &mut Box<dyn Framebuffer>, draw: &mut Draw) {
    if !draw.has_drawn_once {
        draw.has_drawn_once = draw.dirty_rects.len() > 0;
    }

    draw.dirty_update_count += 1;

    if draw.dirty_update_count > MAX_DIRTY_REFRESHES {
        info!("Full refresh!");
        refresh(fb, draw);
    } else {
        for dr in &draw.dirty_rects {
            debug!("Updating dirty rect {:?}", dr);

            #[cfg(feature = "eink_device")]
            {
                if dr.height() < 100 && dr.width() < 100 {
                    debug!("Fast mono update!");
                    fb.update(&dr, UpdateMode::FastMono).ok();
                } else {
                    fb.update(&dr, UpdateMode::Partial).ok();
                }
            }

            push_to_dirty_rect_list(&mut draw.dirty_rects_since_refresh, *dr);
        }

        draw.time_at_last_draw = Instant::now();
    }

    draw.dirty_rects.clear();
}

pub fn push_to_dirty_rect_list(list: &mut Vec<Rectangle>, rect: Rectangle) {
    for dr in list.iter_mut() {
        if dr.contains(&rect) {
            return;
        }
        if rect.contains(&dr) {
            *dr = rect;
            return;
        }
        if rect.extends(&dr) {
            dr.absorb(&rect);
            return;
        }
    }

    list.push(rect);
}
