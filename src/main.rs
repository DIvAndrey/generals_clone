use egui_macroquad::macroquad;
use macroquad::prelude::*;

pub mod app;
pub mod constants;

use app::App;

#[macroquad::main("Колонизатор")]
async fn main() {
    let mut app = App::new();
    loop {
        clear_background(LIGHTGRAY);

        // Process keys, mouse etc.

        app.process_frame();
        
        // Draw things after egui

        next_frame().await;
    }
}
