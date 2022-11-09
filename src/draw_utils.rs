use crate::prelude::*;
use ggez::graphics::Color;
use hex_color::HexColor;

pub const BLACK: Color = Color::BLACK;
pub const WHITE: Color = Color::WHITE;
lazy_static! {
    pub static ref RED: Color = Color::from_rgb_u32(0xF44336);
    pub static ref GREEN: Color = Color::from_rgb_u32(0x4CAF50);
}

pub fn str_to_color(input: &str) -> Option<Color> {
    match input {
        "black" => Some(BLACK),
        "white" => Some(WHITE),
        "red" => Some(RED.to_owned()),
        "green" => Some(GREEN.to_owned()),
        hex if hex.starts_with("#") => {
            let parsed = HexColor::parse(hex).ok()?;
            Some(Color::from_rgba(parsed.r, parsed.g, parsed.b, parsed.a))
        }
        _ => None,
    }
}
