use ggez::graphics::Color;
use lazy_static::lazy_static;

pub const BLACK: Color = Color::BLACK;
pub const WHITE: Color = Color::WHITE;
lazy_static! {
    pub static ref RED: Color = Color::from_rgb_u32(0xF44336);
    pub static ref GREEN: Color = Color::from_rgb_u32(0x4CAF50);
}
