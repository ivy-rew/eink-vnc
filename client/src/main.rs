#[macro_use]
extern crate log;
extern crate byteorder;
extern crate flate2;

//use sdl2::pixels::{Color, PixelMasks, PixelFormatEnum as SdlPixelFormat};
//use sdl2::rect::Rect as SdlRect;

mod device;
mod framebuffer;
#[macro_use]
mod geom;
mod color;
mod input;
mod security;
mod settings;
mod vnc;

pub use crate::framebuffer::image::ReadonlyPixmap;
use crate::framebuffer::{Framebuffer, KoboFramebuffer1, KoboFramebuffer2, Pixmap, UpdateMode};
use crate::geom::Rectangle;
use crate::vnc::{client, Client, Encoding, Rect};
use clap::{value_t, App, Arg};
use log::{debug, error, info};
use std::thread;
use std::time::Duration;
use std::time::Instant;

use anyhow::{Context as ResultExt, Error};

use crate::device::CURRENT_DEVICE;

const FB_DEVICE: &str = "/dev/fb0";

#[repr(align(256))]
pub struct PostProcBin {
    data: [u8; 256],
}

fn main() -> Result<(), Error> {
    env_logger::init();

    let matches = App::new("einkvnc")
        .about("VNC client")
        .arg(
            Arg::with_name("HOST")
                .help("server hostname or IP")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("PORT")
                .help("server port (default: 5900)")
                .index(2),
        )
        .arg(
            Arg::with_name("USERNAME")
                .help("server username")
                .long("username")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("PASSWORD")
                .help("server password")
                .long("password")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("EXCLUSIVE")
                .help("request a non-shared session")
                .long("exclusive"),
        )
        .arg(
            Arg::with_name("CONTRAST")
                .help("apply a post processing contrast filter")
                .long("contrast")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("GRAYPOINT")
                .help("the gray point of the post processing contrast filter")
                .long("graypoint")
                .takes_value(true),
        )
          .arg(
            Arg::with_name("WHITECUTOFF")
                .help("apply a post processing filter to turn colors greater than the specified value to white (255)")
                .long("whitecutoff")
                .takes_value(true),
        )
        .get_matches();

    let host = matches.value_of("HOST").unwrap();
    let port = value_t!(matches.value_of("PORT"), u16).unwrap_or(5900);
    let username = matches.value_of("USERNAME");
    let password = matches.value_of("PASSWORD");
    let contrast_exp = value_t!(matches.value_of("CONTRAST"), f32).unwrap_or(1.0);
    let contrast_gray_point = value_t!(matches.value_of("GRAYPOINT"), f32).unwrap_or(224.0);
    let white_cutoff = value_t!(matches.value_of("WHITECUTOFF"), u8).unwrap_or(255);
    let exclusive = matches.is_present("EXCLUSIVE");

    //let sdl_context = sdl2::init().unwrap();
    //let sdl_video = sdl_context.video().unwrap();
    //let mut sdl_timer = sdl_context.timer().unwrap();
    //let mut sdl_events = sdl_context.event_pump().unwrap();

    info!("connecting to {}:{}", host, port);
    let stream = match std::net::TcpStream::connect((host, port)) {
        Ok(stream) => stream,
        Err(error) => {
            error!("cannot connect to {}:{}: {}", host, port, error);
            std::process::exit(1)
        }
    };

    let mut vnc = match Client::from_tcp_stream(stream, !exclusive, |methods| {
        debug!("available authentication methods: {:?}", methods);
        for method in methods {
            match method {
                client::AuthMethod::None => return Some(client::AuthChoice::None),
                client::AuthMethod::Password => {
                    return match password {
                        None => None,
                        Some(ref password) => {
                            let mut key = [0; 8];
                            for (i, byte) in password.bytes().enumerate() {
                                if i == 8 {
                                    break;
                                }
                                key[i] = byte
                            }
                            Some(client::AuthChoice::Password(key))
                        }
                    }
                }
                client::AuthMethod::AppleRemoteDesktop => match (username, password) {
                    (Some(username), Some(password)) => {
                        return Some(client::AuthChoice::AppleRemoteDesktop(
                            username.to_owned(),
                            password.to_owned(),
                        ))
                    }
                    _ => (),
                },
            }
        }
        None
    }) {
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

    vnc.request_update(
        Rect {
            left: 0,
            top: 0,
            width,
            height,
        },
        false,
    )
    .unwrap();

    #[cfg(feature = "eink_device")]
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
        let startup_rotation = 0;
        fb.set_rotation(startup_rotation).ok();
    }

    let post_proc_bin = PostProcBin {
        data: (0..=255)
            .map(|i| {
                if contrast_exp == 1.0 {
                    i
                } else {
                    let gray = contrast_gray_point;

                    let rem_gray = 255.0 - gray;
                    let inv_exponent = 1.0 / contrast_exp;

                    let raw_color = i as f32;
                    if raw_color < gray {
                        (gray * (raw_color / gray).powf(contrast_exp)) as u8
                    } else if raw_color > gray {
                        (gray + rem_gray * ((raw_color - gray) / rem_gray).powf(inv_exponent)) as u8
                    } else {
                        gray as u8
                    }
                }
            })
            .map(|i| -> u8 {
                if i > white_cutoff {
                    255
                } else {
                    i
                }
            })
            .collect::<Vec<u8>>()
            .try_into()
            .unwrap(),
    };

    const FRAME_MS: u64 = 1000 / 30;
    //const FRAME_MS_U32: u32 = 1000 / 30;

    const MAX_DIRTY_REFRESHES: usize = 500;

    let mut dirty_rects: Vec<Rectangle> = Vec::new();
    let mut dirty_rects_since_refresh: Vec<Rectangle> = Vec::new();
    let mut has_drawn_once = false;
    let mut dirty_update_count = 0;

    let mut time_at_last_draw = Instant::now();

    let fb_rect = rect![0, 0, width as i32, height as i32];

    let post_proc_enabled = contrast_exp != 1.0;
    //let key_ctrl = false;
    //let (mouse_x,   mouse_y)   = (0u16, 0u16);
    //let mouse_buttons = 0u8;

    'running: loop {
        let time_at_sol = Instant::now();
        //let ticks = sdl_timer.ticks();

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

                    let scale_down = 
                        pixels
                            .iter()
                            .step_by(4)
                            .map(|&c| post_proc_bin.data[c as usize])
                            .collect();

                    let post_proc_pixels = if post_proc_enabled {
                        pixels
                            .iter()
                            .step_by(4)
                            .map(|&c| post_proc_bin.data[c as usize])
                            .collect()
                    } else {
                        Vec::new()
                    };

                    let pixels = if post_proc_enabled {
                        &post_proc_pixels
                    } else {
                        &scale_down
                    };

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

/*
    for event in sdl_events.wait_timeout_iter(sdl_timer.ticks() - ticks + FRAME_MS_U32 ) {
            //use sdl2::event::{Event, WindowEventId};

            match event {
                Event::Quit { .. } => break 'running,
                Event::Window { win_event_id: WindowEventId::SizeChanged, .. } => {
                    let screen_rect = SdlRect::new_unwrap(
                        0, 0, width as u32, height as u32);
                    renderer.copy(&screen, None, Some(screen_rect));
                    renderer.present()
                },
                _ => ()
            }

            if view_only { continue }
*/

/*
            match event {
                Event::KeyDown { keycode: Some(keycode), .. } |
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    use sdl2::keyboard::Keycode;
                    let down = match event { Event::KeyDown { .. } => true, _ => false };
                    match keycode {
                        Keycode::LCtrl | Keycode::RCtrl => key_ctrl = down,
                        _ => ()
                    }
                    match map_special_key(key_ctrl, keycode) {
                        Some(keysym) => { vnc.send_key_event(down, keysym).unwrap() },
                        None => ()
                    }
                },
                Event::TextInput { text, .. } => {
                    let chr = 0x01000000 + text.chars().next().unwrap() as u32;
                    vnc.send_key_event(true, chr).unwrap();
                    vnc.send_key_event(false, chr).unwrap()
                }
                Event::MouseMotion { x, y, .. } => {
                    mouse_x = x as u16;
                    mouse_y = y as u16;
                    //if !qemu_hacks {
                        vnc.send_pointer_event(mouse_buttons, mouse_x, mouse_y).unwrap()
                    //}
                },
                Event::MouseButtonDown { x, y, mouse_btn, .. } |
                Event::MouseButtonUp { x, y, mouse_btn, .. } => {
                    use sdl2::mouse::Mouse;
                    mouse_x = x as u16;
                    mouse_y = y as u16;
                    let mouse_button =
                        match mouse_btn {
                            Mouse::Left       => 0x01,
                            Mouse::Middle     => 0x02,
                            Mouse::Right      => 0x04,
                            Mouse::X1         => 0x20,
                            Mouse::X2         => 0x40,
                            Mouse::Unknown(_) => 0x00
                        };
                    match event {
                        Event::MouseButtonDown { .. } => mouse_buttons |= mouse_button,
                        Event::MouseButtonUp   { .. } => mouse_buttons &= !mouse_button,
                        _ => unreachable!()
                    };
                    vnc.send_pointer_event(mouse_buttons, mouse_x, mouse_y).unwrap()
                },
                Event::MouseWheel { y, .. } => {
                    if y == 1 {
                        vnc.send_pointer_event(mouse_buttons | 0x08, mouse_x, mouse_y).unwrap();
                        vnc.send_pointer_event(mouse_buttons, mouse_x, mouse_y).unwrap();
                    } else if y == -1 {
                        vnc.send_pointer_event(mouse_buttons | 0x10, mouse_x, mouse_y).unwrap();
                        vnc.send_pointer_event(mouse_buttons, mouse_x, mouse_y).unwrap();
                    }
                }
                Event::ClipboardUpdate { .. } => {
                    vnc.update_clipboard(&sdl_video.clipboard().clipboard_text().unwrap()).unwrap()
                },
                _ => ()
            }
        }
*/

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

/*
fn map_special_key(alnum_ok: bool, keycode: sdl2::keyboard::Keycode) -> Option<u32> {
    use sdl2::keyboard::Keycode::*;
    use x11::keysym::*;

    let x11code = match keycode {
        Space => XK_space,
        Exclaim => XK_exclam,
        Quotedbl => XK_quotedbl,
        Hash => XK_numbersign,
        Dollar => XK_dollar,
        Percent => XK_percent,
        Ampersand => XK_ampersand,
        Quote => XK_apostrophe,
        LeftParen => XK_parenleft,
        RightParen => XK_parenright,
        Asterisk => XK_asterisk,
        Plus => XK_plus,
        Comma => XK_comma,
        Minus => XK_minus,
        Period => XK_period,
        Slash => XK_slash,
        Num0 => XK_0,
        Num1 => XK_1,
        Num2 => XK_2,
        Num3 => XK_3,
        Num4 => XK_4,
        Num5 => XK_5,
        Num6 => XK_6,
        Num7 => XK_7,
        Num8 => XK_8,
        Num9 => XK_9,
        Colon => XK_colon,
        Semicolon => XK_semicolon,
        Less => XK_less,
        Equals => XK_equal,
        Greater => XK_greater,
        Question => XK_question,
        At => XK_at,
        LeftBracket => XK_bracketleft,
        Backslash => XK_backslash,
        RightBracket => XK_bracketright,
        Caret => XK_caret,
        Underscore => XK_underscore,
        Backquote => XK_grave,
        A => XK_a,
        B => XK_b,
        C => XK_c,
        D => XK_d,
        E => XK_e,
        F => XK_f,
        G => XK_g,
        H => XK_h,
        I => XK_i,
        J => XK_j,
        K => XK_k,
        L => XK_l,
        M => XK_m,
        N => XK_n,
        O => XK_o,
        P => XK_p,
        Q => XK_q,
        R => XK_r,
        S => XK_s,
        T => XK_t,
        U => XK_u,
        V => XK_v,
        W => XK_w,
        X => XK_x,
        Y => XK_y,
        Z => XK_z,
        _ => 0
    };
    if x11code != 0 && alnum_ok { return Some(x11code as u32) }

    let x11code = match keycode {
        Backspace => XK_BackSpace,
        Tab => XK_Tab,
        Return => XK_Return,
        Escape => XK_Escape,
        Delete => XK_Delete,
        CapsLock => XK_Caps_Lock,
        F1 => XK_F1,
        F2 => XK_F2,
        F3 => XK_F3,
        F4 => XK_F4,
        F5 => XK_F5,
        F6 => XK_F6,
        F7 => XK_F7,
        F8 => XK_F8,
        F9 => XK_F9,
        F10 => XK_F10,
        F11 => XK_F11,
        F12 => XK_F12,
        PrintScreen => XK_Print,
        ScrollLock => XK_Scroll_Lock,
        Pause => XK_Pause,
        Insert => XK_Insert,
        Home => XK_Home,
        PageUp => XK_Page_Up,
        End => XK_End,
        PageDown => XK_Page_Down,
        Right => XK_Right,
        Left => XK_Left,
        Down => XK_Down,
        Up => XK_Up,
        NumLockClear => XK_Num_Lock,
        KpDivide => XK_KP_Divide,
        KpMultiply => XK_KP_Multiply,
        KpMinus => XK_KP_Subtract,
        KpPlus => XK_KP_Add,
        KpEnter => XK_KP_Enter,
        Kp1 => XK_KP_1,
        Kp2 => XK_KP_2,
        Kp3 => XK_KP_3,
        Kp4 => XK_KP_4,
        Kp5 => XK_KP_5,
        Kp6 => XK_KP_6,
        Kp7 => XK_KP_7,
        Kp8 => XK_KP_8,
        Kp9 => XK_KP_9,
        Kp0 => XK_KP_0,
        KpPeriod => XK_KP_Separator,
        F13 => XK_F13,
        F14 => XK_F14,
        F15 => XK_F15,
        F16 => XK_F16,
        F17 => XK_F17,
        F18 => XK_F18,
        F19 => XK_F19,
        F20 => XK_F20,
        F21 => XK_F21,
        F22 => XK_F22,
        F23 => XK_F23,
        F24 => XK_F24,
        Menu => XK_Menu,
        Sysreq => XK_Sys_Req,
        LCtrl => XK_Control_L,
        LShift => XK_Shift_L,
        LAlt => XK_Alt_L,
        LGui => XK_Super_L,
        RCtrl => XK_Control_R,
        RShift => XK_Shift_R,
        RAlt => XK_Alt_R,
        RGui => XK_Super_R,
        _ => 0
    };
    if x11code != 0 { Some(x11code as u32) } else { None }
}
*/

