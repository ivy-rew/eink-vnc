
#[repr(align(256))]
pub struct PostProcBin {
    pub data: [u8; 256],
}

impl PostProcBin {
    pub fn new(config: &PostProcConfig) -> PostProcBin {
        return PostProcBin {
            data: (0..=255)
                .map(|i| {
                    if config.contrast_exp == 1.0 {
                        i
                    } else {
                        let gray = config.contrast_gray_point;
    
                        let rem_gray = 255.0 - gray;
                        let inv_exponent = 1.0 / config.contrast_exp;
    
                        let raw_color = i as f32;
                        if raw_color < gray {
                            (gray * (raw_color / gray).powf(config.contrast_exp)) as u8
                        } else if raw_color > gray {
                            (gray + rem_gray * ((raw_color - gray) / rem_gray).powf(inv_exponent)) as u8
                        } else {
                            gray as u8
                        }
                    }
                })
                .map(|i| -> u8 {
                    if i > config.white_cutoff {
                        255
                    } else {
                        i
                    }
                })
                .collect::<Vec<u8>>()
                .try_into()
                .unwrap(),
        };

    }
}

#[derive(Debug, Copy, Clone)]
pub struct PostProcConfig {
    pub contrast_exp: f32,
    pub contrast_gray_point: f32,
    pub white_cutoff: u8,
}
