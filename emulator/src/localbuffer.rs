use std::mem;
use sdl2::rect::Point as SdlPoint;
use sdl2::rect::Rect as SdlRect;
use sdl2::pixels::{Color as SdlColor, PixelFormatEnum};
use sdl2::render::{WindowCanvas, BlendMode};
use display::framebuffer::{Framebuffer, UpdateMode};
use display::geom::{Rectangle, Axis};
use display::color::Color;
use anyhow::{Context as ResultExt, Error};
use chrono::Local;

const DEFAULT_ROTATION: i8 = 1;

pub fn new(title: &str, width: u32, height: u32) -> Box<dyn Framebuffer>{
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
                 .window(title, width, height)
                 .resizable()
                 .position_centered()
                 .build()
                 .unwrap();

    let mut fb: sdl2::render::Canvas<sdl2::video::Window> = window.into_canvas().software().build().unwrap();
    fb.set_blend_mode(BlendMode::Blend);
    Box::new(FBCanvas(fb))
}

pub struct FBCanvas(pub WindowCanvas);

impl Framebuffer for FBCanvas {
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        let [red, green, blue] = color.rgb();
        self.0.set_draw_color(SdlColor::RGB(red, green, blue));
        self.0.draw_point(SdlPoint::new(x as i32, y as i32)).unwrap();
    }
    fn set_blended_pixel(&mut self, x: u32, y: u32, color: Color, alpha: f32) {
        debug!("set blended pixel {}/{}", x, y);
        let [red, green, blue] = color.rgb();
        self.0.set_draw_color(SdlColor::RGBA(red, green, blue, (alpha * 255.0) as u8));
        self.0.draw_point(SdlPoint::new(x as i32, y as i32)).unwrap();
    }

    fn invert_region(&mut self, rect: &Rectangle) {
        let width = rect.width();
        let s_rect = Some(SdlRect::new(rect.min.x, rect.min.y,
                                       width, rect.height()));
        if let Ok(data) = self.0.read_pixels(s_rect, PixelFormatEnum::RGB24) {
            for y in rect.min.y..rect.max.y {
                let v = (y - rect.min.y) as u32;
                for x in rect.min.x..rect.max.x {
                    let u = (x - rect.min.x) as u32;
                    let addr = 3 * (v * width + u);
                    let red = data[addr as usize];
                    let green = data[(addr+1) as usize];
                    let blue = data[(addr+2) as usize];
                    let mut color = Color::Rgb(red, green, blue);
                    color.invert();
                    self.set_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    fn shift_region(&mut self, rect: &Rectangle, drift: u8) {
        let width = rect.width();
        let s_rect = Some(SdlRect::new(rect.min.x, rect.min.y,
                                       width, rect.height()));
        if let Ok(data) = self.0.read_pixels(s_rect, PixelFormatEnum::RGB24) {
            for y in rect.min.y..rect.max.y {
                let v = (y - rect.min.y) as u32;
                for x in rect.min.x..rect.max.x {
                    let u = (x - rect.min.x) as u32;
                    let addr = 3 * (v * width + u);
                    let red = data[addr as usize];
                    let green = data[(addr+1) as usize];
                    let blue = data[(addr+2) as usize];
                    let mut color = Color::Rgb(red, green, blue);
                    color.shift(drift);
                    self.set_pixel(x as u32, y as u32, color);
                }
            }
        }
    }

    fn update(&mut self, _rect: &Rectangle, _mode: UpdateMode) -> Result<u32, Error> {
        debug!("update {:}", _rect);
        self.0.present();
        Ok(Local::now().timestamp_subsec_millis())
    }

    fn wait(&self, _tok: u32) -> Result<i32, Error> {
        Ok(1)
    }

    fn save(&self, path: &str) -> Result<(), Error> {
        let (width, height) = self.dims();
        //let file = File::create(path).with_context(|| format!("can't create output file {}", path))?;
        // let mut encoder = png::Encoder::new(file, width, height);
        // encoder.set_depth(png::BitDepth::Eight);
        // encoder.set_color(png::ColorType::Rgb);
        // let mut writer = encoder.write_header().with_context(|| format!("can't write PNG header for {}", path))?;
        // let data = self.0.read_pixels(self.0.viewport(), PixelFormatEnum::RGB24).unwrap_or_default();
        // writer.write_image_data(&data).with_context(|| format!("can't write PNG data to {}", path))?;
        Ok(())
    }

    fn rotation(&self) -> i8 {
        DEFAULT_ROTATION
    }

    fn set_rotation(&mut self, n: i8) -> Result<(u32, u32), Error> {
        let (mut width, mut height) = self.dims();
        if (width < height && n % 2 == 0) || (width > height && n % 2 == 1) {
            mem::swap(&mut width, &mut height);
        }
        self.0.window_mut().set_size(width, height).ok();
        Ok((width, height))
    }

    // fn get_pixel(&self, x: u32, y: u32) -> u8 {
    //     debug!("virtualfb: get pixel {}/{}", x, y);
    //     1
    // }

    fn set_monochrome(&mut self, _enable: bool) {
        debug!("set mono {}", _enable)
    }

    fn set_dithered(&mut self, _enable: bool) {
        debug!("set dither {}", _enable)
    }

    fn set_inverted(&mut self, _enable: bool) {
        debug!("set invert {}", _enable)
    }

    fn monochrome(&self) -> bool {
        false
    }

    fn dithered(&self) -> bool {
        false
    }

    fn inverted(&self) -> bool {
        false
    }

    fn width(&self) -> u32 {
        self.0.window().size().0
    }

    fn height(&self) -> u32 {
        self.0.window().size().1
    }
}
