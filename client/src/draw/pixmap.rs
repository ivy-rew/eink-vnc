use display::color::{Color, WHITE};
pub const RED: Color = Color::Rgb(255, 0, 0);

#[derive(Debug, Clone)]
pub struct ReadonlyPixmap<'a> {
    pub width: u32,
    pub height: u32,
    pub samples: usize,
    pub data: &'a Vec<u8>,
}

impl<'a> ReadonlyPixmap<'a> {
    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if self.data.is_empty() {
            return WHITE;
        }
        let addr = self.samples * (y * self.width + x) as usize;
        let max=self.data.len();
        if self.samples == 1 {
            if max < addr {
                return RED; // signal an invalid pixel request
            }
            let col = self.data[addr];
            Color::Gray(col)
        } else {
            if max < addr+4 {
                return RED; // signal an invalid pixel request
            }
            let bgr: &[u8] = &self.data[addr..addr+3];
            Color::Rgb(bgr[2], bgr[1], bgr[0])
        }
    }
}
