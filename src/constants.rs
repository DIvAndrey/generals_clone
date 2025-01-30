use egui_macroquad::macroquad::prelude::*;

#[cfg(target_arch = "wasm32")]
pub const UI_SCALE_COEFFICIENT: f32 = 1.0 / 800.0;
#[cfg(not(target_arch = "wasm32"))]
pub const UI_SCALE_COEFFICIENT: f32 = 1.0 / 600.0;

pub const PLAYER_COLORS: [Color; 16] = [
    color_u8!(204, 0, 0, 255),
    color_u8!(0, 89, 181, 255),
    color_u8!(0, 102, 0, 255),
    color_u8!(0, 102, 102, 255),
    color_u8!(204, 114, 0, 255),
    color_u8!(178, 12, 165, 255),
    color_u8!(102, 0, 102, 255),
    color_u8!(127, 0, 0, 255),
    color_u8!(153, 153, 38, 255),
    color_u8!(127, 76, 25, 255),
    color_u8!(0, 25, 191, 255),
    color_u8!(76, 63, 127, 255),
    color_u8!(102, 127, 25, 255),
    color_u8!(153, 0, 0, 255),
    color_u8!(89, 51, 114, 255),
    color_u8!(114, 102, 63, 255),
];

pub const DIRECTIONS: [(usize, usize); 4] =
    [((-1i32) as usize, 0), (0, (-1i32) as usize), (0, 1), (1, 0)];

pub const DELAY_BETWEEN_TICKS: f64 = 0.2;
