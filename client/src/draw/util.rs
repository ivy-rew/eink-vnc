use crate::draw::pixmap::ReadonlyPixmap;
use display::device::CURRENT_DEVICE;
use display::geom::Rectangle;
use display::rect;
use vnc::Rect;

use super::kobo::MapDelta;

pub fn to_map<'a>(vnc_rect: &'a Rect, pixels: &'a Vec<u8>) -> ReadonlyPixmap<'a> {
    let w = vnc_rect.width as u32;
    let h = vnc_rect.height as u32;
    
    let colors = if CURRENT_DEVICE.color_samples() == 1 { 1 } else { 4 };
    let pixmap = ReadonlyPixmap {
        width: w as u32,
        height: h as u32,
        data: pixels,
        samples: colors,
    };
    return pixmap;
}

pub fn to_delta_rect(vnc_rect: &Rect) -> Rectangle {
    let w = vnc_rect.width as i32;
    let h = vnc_rect.height as i32;
    let l = vnc_rect.left as i32;
    let t = vnc_rect.top as i32;
    rect![l, t, l + w, t + h]
}

pub fn to_delta_map(dst: &Rect) -> MapDelta{
    MapDelta { 
        left: dst.left as u32, 
        top: dst.top as u32 
    }
}
