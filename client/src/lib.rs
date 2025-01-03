#![allow(unused)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate flate2;

pub mod device;
pub mod framebuffer;
#[macro_use]
pub mod geom;
mod color;
pub mod input;
mod security;
pub mod settings;
mod vnc;
mod touch;
pub mod config;
mod auth;
pub mod processing;

pub use crate::framebuffer::image::ReadonlyPixmap;
use crate::framebuffer::{Framebuffer, KoboFramebuffer1, KoboFramebuffer2, Pixmap, UpdateMode};
use crate::geom::Rectangle;
use crate::vnc::{client, Client, Encoding, Rect};
use crate::touch::{Touch, TouchEventListener, mouse_btn_to_vnc, MOUSE_UNKNOWN};
use crate::config::Config;
use crate::processing::PostProcBin;

use config::Connection;
use log::{debug, error, info};
use clap::ArgMatches;
use std::str::FromStr;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use std::sync::mpsc::Receiver;
use std::sync::mpsc;
use anyhow::{Context as ResultExt, Error};

use crate::device::CURRENT_DEVICE;

pub fn connect(con: Connection) -> Client {
    info!("connecting to {}:{}", con.host, con.port);
    let stream = match std::net::TcpStream::connect((con.host, con.port)) {
        Ok(stream) => stream,
        Err(error) => {
            error!("cannot connect to {}:{}: {}", con.host, con.port, error);
            std::process::exit(1)
        }
    };

    let mut vnc = match Client::from_tcp_stream(stream, !con.exclusive, |methods| auth::authenticate(&con, methods)) {
        Ok(vnc) => vnc,
        Err(error) => {
            error!("cannot initialize VNC session: {}", error);
            std::process::exit(1)
        }
    };

    let (width, height) = vnc.size();
    info!(
        "connected to \"{}\", {}x{} framebuffer",
        vnc.name(),
        width,
        height
    );

    let vnc_format = vnc.format();
    info!("received {:?}", vnc_format);

    vnc.set_encodings(&[Encoding::CopyRect, Encoding::Zrle])
        .unwrap();

    vnc.request_update(full_rect(vnc.size()), false)
        .unwrap();

    #[cfg(feature = "eink_device")]
    debug!(
        "running on device model=\"{}\" /dpi={} /dims={}x{}", 
        CURRENT_DEVICE.model,
        CURRENT_DEVICE.dpi,
        CURRENT_DEVICE.dims.0,
        CURRENT_DEVICE.dims.1
    );

    vnc
}


pub fn run(mut vnc: &mut Client, mut fb: &mut Box<dyn Framebuffer>, config: &Config) -> Result<(), Error> {
    let (width, height) = vnc.size();
    let vnc_format = vnc.format();
    const FRAME_MS: u64 = 1000 / 30;
    
    const MAX_DIRTY_REFRESHES: usize = 500;
    
    let mut dirty_rects: Vec<Rectangle> = Vec::new();
    let mut dirty_rects_since_refresh: Vec<Rectangle> = Vec::new();
    let mut has_drawn_once = false;
    let mut dirty_update_count = 0;
    
    let mut time_at_last_draw = Instant::now();
    
    let fb_rect = rect![0, 0, width as i32, height as i32];
    
    let post_proc_bin = PostProcBin::new(&config.processing);
    
    let touch_enabled: bool = !config.view_only;
    let rx: Receiver<Touch> = if touch_enabled {
        record_touch_events(config.touch_input.to_string())
    } else {
        mpsc::channel().1 // no-op; never sending anything
    };

    let mut last_button: u8 = MOUSE_UNKNOWN;
    let mut last_button_pen: u8 = MOUSE_UNKNOWN;
    let mut last_full_touch: Option<Touch> = None;

    'running: loop {
        let time_at_sol = Instant::now();
    
        for t in rx.try_iter() {
            last_button = mouse_btn_to_vnc(t.button).unwrap_or(last_button);
            let send_button: u8 = if t.distance.is_some() && t.distance.unwrap().is_positive() {
                MOUSE_UNKNOWN // not-touching; keep mouse up (pre-serving any passed last_button state)
            } else {
                last_button
            };
            vnc.send_pointer_event(send_button,
                t.position.x.try_into().unwrap(),
                t.position.y.try_into().unwrap()
            ).unwrap();
            if (t.stylus_back.is_some() && t.stylus_back.unwrap().eq(&1)) {
                info!("full update due to stylus back-button-touch");
                vnc.request_update(full_rect(vnc.size()), false).unwrap();
            }
        }

        for event in vnc.poll_iter() {
            use client::Event;

            match event {
                Event::Disconnected(None) => break 'running,
                Event::Disconnected(Some(error)) => {
                    error!("server disconnected: {:?}", error);
                    break 'running;
                }
                Event::PutPixels(vnc_rect, ref pixels) => {
                    debug!("Put pixels");

                    let elapsed_ms = time_at_sol.elapsed().as_millis();
                    debug!("network Δt: {}", elapsed_ms);

                    let post_process = 
                        pixels
                            .iter()
                            .step_by(4)
                            .map(|&c| post_proc_bin.data[c as usize])
                            .collect();
                    let pixels = &post_process;

                    let w = vnc_rect.width as u32;
                    let h = vnc_rect.height as u32;
                    let l = vnc_rect.left as u32;
                    let t = vnc_rect.top as u32;

                    let pixmap = ReadonlyPixmap {
                        width: w as u32,
                        height: h as u32,
                        data: pixels,
                    };
                    debug!("Put pixels {} {} {} size {}",w,h,w*h,pixels.len());

                    let elapsed_ms = time_at_sol.elapsed().as_millis();
                    debug!("postproc Δt: {}", elapsed_ms);

                    #[cfg(feature = "eink_device")]
                    {
                        for y in 0..pixmap.height {
                            for x in 0..pixmap.width {
                                let px = x + l;
                                let py = y + t;
                                let color = pixmap.get_pixel(x, y);
                                fb.set_pixel(px, py, color);
                            }
                        }
                    }

                    let elapsed_ms = time_at_sol.elapsed().as_millis();
                    debug!("draw Δt: {}", elapsed_ms);

                    let w = vnc_rect.width as i32;
                    let h = vnc_rect.height as i32;
                    let l = vnc_rect.left as i32;
                    let t = vnc_rect.top as i32;

                    let delta_rect = rect![l, t, l + w, t + h];
                    if delta_rect == fb_rect {
                        dirty_rects.clear();
                        dirty_rects_since_refresh.clear();
                        #[cfg(feature = "eink_device")]
                        {
                            if !has_drawn_once || dirty_update_count > MAX_DIRTY_REFRESHES {
                                fb.update(&fb_rect, UpdateMode::Full).ok();
                                dirty_update_count = 0;
                                has_drawn_once = true;
                            } else {
                                fb.update(&fb_rect, UpdateMode::Partial).ok();
                            }
                        }
                    } else {
                        push_to_dirty_rect_list(&mut dirty_rects, delta_rect);
                    }

                    let elapsed_ms = time_at_sol.elapsed().as_millis();
                    debug!("rects Δt: {}", elapsed_ms);
                }
                Event::CopyPixels { src, dst } => {
                    debug!("Copy pixels!");

                    #[cfg(feature = "eink_device")]
                    {
                        let src_left = src.left as u32;
                        let src_top = src.top as u32;

                        let dst_left = dst.left as u32;
                        let dst_top = dst.top as u32;

                        let mut intermediary_pixmap =
                            Pixmap::new(dst.width as u32, dst.height as u32);

                        for y in 0..intermediary_pixmap.height {
                            for x in 0..intermediary_pixmap.width {
                                let color = fb.get_pixel(src_left + x, src_top + y);
                                intermediary_pixmap.set_pixel(x, y, color);
                            }
                        }

                        for y in 0..intermediary_pixmap.height {
                            for x in 0..intermediary_pixmap.width {
                                let color = intermediary_pixmap.get_pixel(x, y);
                                fb.set_pixel(dst_left + x, dst_top + y, color);
                            }
                        }
                    }

                    let delta_rect = rect![
                        dst.left as i32,
                        dst.top as i32,
                        (dst.left + dst.width) as i32,
                        (dst.top + dst.height) as i32
                    ];
                    push_to_dirty_rect_list(&mut dirty_rects, delta_rect);
                }
                Event::EndOfFrame => {
                    debug!("End of frame!");

                    if !has_drawn_once {
                        has_drawn_once = dirty_rects.len() > 0;
                    }

                    dirty_update_count += 1;

                    if dirty_update_count > MAX_DIRTY_REFRESHES {
                        info!("Full refresh!");
                        for dr in &dirty_rects_since_refresh {
                            #[cfg(feature = "eink_device")]
                            {
                                fb.update(&dr, UpdateMode::Full).ok();
                            }
                        }
                        dirty_update_count = 0;
                        dirty_rects_since_refresh.clear();
                    } else {
                        for dr in &dirty_rects {
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

                            push_to_dirty_rect_list(&mut dirty_rects_since_refresh, *dr);
                        }

                        time_at_last_draw = Instant::now();
                    }

                    dirty_rects.clear();
                }
                // x => info!("{:?}", x), /* ignore unsupported events */
                _ => (),
            }
        }

        if FRAME_MS > time_at_sol.elapsed().as_millis() as u64 {
            if dirty_rects_since_refresh.len() > 0 && time_at_last_draw.elapsed().as_secs() > 3 {
                for dr in &dirty_rects_since_refresh {
                    #[cfg(feature = "eink_device")]
                    {
                        fb.update(&dr, UpdateMode::Full).ok();
                    }
                }
                dirty_update_count = 0;
                dirty_rects_since_refresh.clear();
            }

            if FRAME_MS > time_at_sol.elapsed().as_millis() as u64 {
                thread::sleep(Duration::from_millis(
                    FRAME_MS - time_at_sol.elapsed().as_millis() as u64,
                ));
            }
        } else {
            info!(
                "Missed frame, excess Δt: {}ms",
                time_at_sol.elapsed().as_millis() as u64 - FRAME_MS
            );
        }

        vnc.request_update(
            Rect {
                left: 0,
                top: 0,
                width,
                height,
            },
            true,
        )
        .unwrap();
    }

    Ok(())
}

fn push_to_dirty_rect_list(list: &mut Vec<Rectangle>, rect: Rectangle) {
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

fn record_touch_events(touch_input: String) -> Receiver<Touch> {
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

pub fn full_rect(size: (u16,u16)) -> Rect {
    Rect {
        left: 0,
        top: 0,
        width: size.0,
        height: size.1,
    }
}
