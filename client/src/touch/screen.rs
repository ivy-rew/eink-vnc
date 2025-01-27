use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use std::thread;
use vnc::Client;
use crate::MOUSE_UNKNOWN;
use crate::full_rect;

use crate::{Touch, TouchEventListener};

pub fn record_screen(touch_input: String) -> Receiver<Touch> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let screen = TouchEventListener::open_input(touch_input).unwrap();
        loop {
            match screen.next_touch(None) {
                Some(touch) => {
                    debug!("touched on screen {:?}", touch.position);
                    tx.send(touch).unwrap();
                },
                None => {}
            };
        }
    });
    return rx;
}

pub fn touch_vnc(mut vnc: &mut Client, touch: Touch, last_button: u8) {
    let send_button: u8 = if touch.distance.is_some() && touch.distance.unwrap().is_positive() {
        MOUSE_UNKNOWN // not-touching; keep mouse up (pre-serving any passed last_button state)
    } else {
        last_button
    };
    vnc.send_pointer_event(send_button,
        touch.position.x.try_into().unwrap(),
        touch.position.y.try_into().unwrap()
    ).unwrap();
    if touch.stylus_back.is_some() && touch.stylus_back.unwrap().eq(&1) {
        info!("full update due to stylus back-button-touch");
        vnc.request_update(full_rect(vnc.size()), false).unwrap();
    }
}
