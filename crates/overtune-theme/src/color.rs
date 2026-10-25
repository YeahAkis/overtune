#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
        Rgba { r, g, b, a: 255 }
    }

    pub fn parse(text: &str) -> Option<Rgba> {
        let hex = text.trim().strip_prefix('#')?;
        if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        let digit = |i: usize| u8::from_str_radix(&hex[i..i + 1], 16).ok();
        let pair = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        match hex.len() {
            3 => Some(Rgba::rgb(digit(0)? * 17, digit(1)? * 17, digit(2)? * 17)),
            6 => Some(Rgba::rgb(pair(0)?, pair(2)?, pair(4)?)),
            8 => Some(Rgba {
                r: pair(0)?,
                g: pair(2)?,
                b: pair(4)?,
                a: pair(6)?,
            }),
            _ => None,
        }
    }

    pub fn hex(&self) -> String {
        if self.a == 255 {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", self.r, self.g, self.b, self.a)
        }
    }

    pub fn css(&self) -> String {
        if self.a == 255 {
            self.hex()
        } else {
            format!(
                "rgba({}, {}, {}, {:.3})",
                self.r,
                self.g,
                self.b,
                f32::from(self.a) / 255.0
            )
        }
    }
}
