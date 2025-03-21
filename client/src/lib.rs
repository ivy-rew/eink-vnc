#![allow(unused_variables, unused_mut)]

#[macro_use]
extern crate log;
extern crate byteorder;
extern crate flate2;

pub mod config;
mod draw;
pub mod processing;
mod touch;
pub mod vnc;

extern crate vnc as vnc_client;
use vnc_client::{client, Client, PixelFormat, Rect};

use crate::config::Config;
use crate::draw::Draw;
use crate::processing::PostProcBin;
use crate::touch::{mouse_btn_to_vnc, Touch, TouchEventListener, MOUSE_UNKNOWN};
use display::device::CURRENT_DEVICE;
use display::framebuffer::{Framebuffer, Pixmap};
use display::rect;

use anyhow::Error;
use log::{debug, error, info};
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use std::time::Instant;

pub const SD_COLOR_FORMAT: PixelFormat = PixelFormat {
    bits_per_pixel: 8,
    depth: 16,
    big_endian: false,
    true_colour: true,
    red_max: 255,
    green_max: 255,
    blue_max: 255,
    red_shift: 16,
    green_shift: 8,
    blue_shift: 0,
};

pub fn run(vnc: &mut Client, fb: &mut Box<dyn Framebuffer>, config: &Config) -> Result<(), Error> {
    #[cfg(feature = "eink_device")]
    debug!(
        "running on device model=\"{}\" /dpi={} /dims={}x{}",
        CURRENT_DEVICE.model, CURRENT_DEVICE.dpi, CURRENT_DEVICE.dims.0, CURRENT_DEVICE.dims.1
    );

    let (width, height) = vnc.size();
    vnc.format();

    vnc.set_format(SD_COLOR_FORMAT).unwrap();
    info!("enforced {:?}", SD_COLOR_FORMAT);

    const FRAME_MS: u64 = 1000 / 30;

    let mut draw: Draw = Draw::new();
    let post_proc_bin = PostProcBin::new(&config.processing);

    let touch_enabled: bool = !config.view_only;
    let touch_display: Receiver<Touch> = if touch_enabled {
        touch::record_screen(config.touch_input.to_string())
    } else {
        mpsc::channel().1 // no-op; never sending anything
    };
    let mut last_button: u8 = MOUSE_UNKNOWN;

    'running: loop {
        let time_at_sol = Instant::now();

        for touch in touch_display.try_iter() {
            last_button = mouse_btn_to_vnc(touch.button).unwrap_or(last_button);
            touch::touch_vnc(vnc, touch, last_button);
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

                    let steps = if CURRENT_DEVICE.color_samples() == 1 {
                        4
                    } else {
                        1
                    };
                    let post_process = pixels
                        .iter()
                        .step_by(steps)
                        .map(|&c| post_proc_bin.data[c as usize])
                        .collect();
                    let map = draw::util::to_map(&vnc_rect, &post_process);
                    debug!(
                        "Put pixels w={} h={} w*h={} size={}",
                        map.width,
                        map.height,
                        map.width * map.height,
                        map.data.len()
                    );
                    let elapsed_ms = time_at_sol.elapsed().as_millis();
                    debug!("postproc Δt: {}", elapsed_ms);

                    let delta = draw::util::to_delta_map(&vnc_rect);
                    draw::kobo::set_pixel_map_ro(fb, &delta, &map);
                    let elapsed_ms = time_at_sol.elapsed().as_millis();
                    debug!("draw Δt: {}", elapsed_ms);

                    let delta_rect = draw::util::to_delta_rect(&vnc_rect);
                    let fb_rect = rect![0, 0, width as i32, height as i32];
                    if delta_rect == fb_rect {
                        draw.update(fb, fb_rect);
                    } else {
                        draw::push_to_dirty_rect_list(&mut draw.dirty_rects, delta_rect);
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
                        let mut intermediary_pixmap = Pixmap::new(
                            dst.width as u32,
                            dst.height as u32,
                            CURRENT_DEVICE.color_samples(),
                        );
                        for y in 0..intermediary_pixmap.height {
                            for x in 0..intermediary_pixmap.width {
                                //let color = fb.get_pixel(src_left + x, src_top + y);
                                //intermediary_pixmap.set_pixel(x, y, color);
                            }
                        }
                        let delta = draw::util::to_delta_map(&dst);
                        draw::kobo::set_pixel_map(fb, &delta, &intermediary_pixmap);
                    }

                    let delta_rect = draw::util::to_delta_rect(&dst);
                    draw::push_to_dirty_rect_list(&mut draw.dirty_rects, delta_rect);
                }
                Event::EndOfFrame => {
                    debug!("End of frame!");
                    draw.draw_end(fb);
                }
                // x => info!("{:?}", x), /* ignore unsupported events */
                _ => (),
            }
        }

        if FRAME_MS > time_at_sol.elapsed().as_millis() as u64 {
            if draw.dirty_rects_since_refresh.len() > 0
                && draw.time_at_last_draw.elapsed().as_secs() > 3
            {
                draw.refresh(fb);
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

pub fn full_rect(size: (u16, u16)) -> Rect {
    Rect {
        left: 0,
        top: 0,
        width: size.0,
        height: size.1,
    }
}
