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

    pub fn update(&mut self, fb: &mut Box<dyn Framebuffer>, fb_rect: Rectangle) {
        self.dirty_rects.clear();
        self.dirty_rects_since_refresh.clear();
        #[cfg(feature = "eink_device")]
        {
            if !self.has_drawn_once || self.dirty_update_count > MAX_DIRTY_REFRESHES {
                fb.update(&fb_rect, UpdateMode::Full).ok();
                self.dirty_update_count = 0;
                self.has_drawn_once = true;
            } else {
                fb.update(&fb_rect, UpdateMode::Partial).ok();
            }
        }
    }

    pub fn refresh(&mut self, fb: &mut Box<dyn Framebuffer>) {
        for dr in &self.dirty_rects_since_refresh {
            #[cfg(feature = "eink_device")]
            {
                fb.update(&dr, UpdateMode::Full).ok();
            }
        }
        self.dirty_update_count = 0;
        self.dirty_rects_since_refresh.clear();
    }

    pub fn draw_end(&mut self, fb: &mut Box<dyn Framebuffer>) {
        if !self.has_drawn_once {
            self.has_drawn_once = self.dirty_rects.len() > 0;
        }
    
        self.dirty_update_count += 1;
    
        if self.dirty_update_count > MAX_DIRTY_REFRESHES {
            info!("Full refresh!");
            self.refresh(fb);
        } else {
            for dr in &self.dirty_rects {
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
    
                push_to_dirty_rect_list(&mut self.dirty_rects_since_refresh, *dr);
            }
    
            self.time_at_last_draw = Instant::now();
        }
        self.dirty_rects.clear();
    }

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
