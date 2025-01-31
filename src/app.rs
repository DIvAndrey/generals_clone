use std::collections::VecDeque;

use bot::{Bot, PathFinderBot};
use egui_macroquad::{egui::{self, Align2, Slider, Window}, macroquad::prelude::*};
use map::{cell::CellType, GameMap, Move};

use crate::constants::*;

pub mod map;
pub mod bot;

#[derive(Clone)]
struct GameParams {
    n: usize,
    m: usize,
    players_num: usize,
    ui_scale: f32,
    new_ui_scale: f32,
    screen_width: f32,
    screen_height: f32,
    screen_min_res: f32,
    mountain_texture: Texture2D,
    general_texture: Texture2D,
    disable_fog_of_war: bool,
}

impl GameParams {
    pub fn update_screen_info(&mut self) {
        self.screen_width = screen_width();
        self.screen_height = screen_height();
        self.screen_min_res = self.screen_width.min(self.screen_height);
    }
}

impl Default for GameParams {
    fn default() -> Self {
        Self { 
            n: 30, 
            m: 30, 
            players_num: 2, 
            ui_scale: 1.0, 
            new_ui_scale: 1.0,
            screen_width: 100.0,
            screen_height: 100.0,
            screen_min_res: 100.0,
            mountain_texture: Texture2D::from_file_with_format(include_bytes!("../assets/sprites/mountain.png"), Some(ImageFormat::Png)),
            general_texture: Texture2D::from_file_with_format(include_bytes!("../assets/sprites/general.png"), Some(ImageFormat::Png)),
            disable_fog_of_war: false,
        }
    }
}

trait Scene {
    fn process_frame_and_get_next_scene(&mut self) -> Option<Box<dyn Scene>>;
}

struct GameScene {
    map: GameMap,
    params: GameParams,
    selected_cell: Option<(usize, usize)>,
    moves_queue: VecDeque<Move>,
    bots: Vec<Box<dyn Bot>>,
    player_color: usize,
    last_tick_time: f64,
}

impl GameScene {
    pub fn new(params: GameParams) -> GameScene {
        let player_color = 0;
        // let player_color = fastrand::usize(0..params.players_num);
        let map = GameMap::new_random(params.n, params.m, params.players_num);
        GameScene {
            bots: (0..params.players_num).map(|i| Box::new(PathFinderBot::from_map(&map, i)) as Box<dyn Bot>).collect(),
            player_color,
            map,
            params: params,
            selected_cell: None,
            moves_queue: VecDeque::new(),
            last_tick_time: -100.0,
        }
    }

    fn draw_game_map(&self) {
        let cell_size = (self.params.screen_height / self.params.n as f32).min(self.params.screen_width / self.params.m as f32) * 0.95;
        let map_height = cell_size * self.params.n as f32;
        let map_width = cell_size * self.params.m as f32;
        let map_y_offset = (self.params.screen_height - map_height) * 0.5;
        let map_x_offset = (self.params.screen_width - map_width) * 0.5;

        for y in 0..self.params.n {
            for x in 0..self.params.m {
                let x1 = x as f32 * cell_size + map_x_offset;
                let y1 = y as f32 * cell_size + map_y_offset;
                let cell = if self.params.disable_fog_of_war { 
                    self.map.grid[y][x] 
                } else { 
                    self.map.get_with_fog(y, x, self.player_color)
                };
                let color = match cell.owner {
                    Some(id) => PLAYER_COLORS[id % PLAYER_COLORS.len()],
                    None => WHITE,
                };
                draw_rectangle(x1, y1, cell_size, cell_size, color);
                match cell.cell_type {
                    CellType::Empty => {},
                    CellType::Mountains => draw_texture_ex(self.params.mountain_texture, x1, y1, WHITE, DrawTextureParams {
                        dest_size: Some(Vec2::splat(cell_size)),
                        ..Default::default()
                    }),
                    CellType::City => draw_circle(x1 + cell_size * 0.5, y1 + cell_size * 0.5, cell_size * 0.4, BLACK),
                    CellType::General => draw_texture_ex(self.params.general_texture, x1, y1, WHITE, DrawTextureParams {
                        dest_size: Some(Vec2::splat(cell_size)),
                        ..Default::default()
                    }),
                }
                if cell.army_size > 0 {
                    let text_x_offset = if cell.army_size <= 9 {
                        cell_size * 0.35
                    } else if cell.army_size < 100 {
                        cell_size * 0.2
                    } else {
                        0.0
                    };
                    draw_text_ex(&format!("{}", cell.army_size), x1 - 1.0 + text_x_offset, y1 + 1.0 + cell_size * 0.7, TextParams {
                        font_size: (cell_size * 0.7) as u16,
                        color: BLACK,
                        ..Default::default()
                    });
                    draw_text_ex(&format!("{}", cell.army_size), x1 - 1.0 + text_x_offset, y1 + 1.0 + cell_size * 0.7, TextParams {
                        font_size: (cell_size * 0.7) as u16,
                        color: BLACK,
                        ..Default::default()
                    });
                    draw_text_ex(&format!("{}", cell.army_size), x1 + text_x_offset, y1 + cell_size * 0.7, TextParams {
                        font_size: (cell_size * 0.7) as u16,
                        color: Color::new(0.7, 0.9, 1.0, 1.0),
                        ..Default::default()
                    });
                }
                if !self.map.is_visible_to(y, x, self.player_color) {
                    draw_rectangle(x1, y1, cell_size, cell_size, Color::new(0.3, 0.3, 0.3, 0.5));
                }
                if Some((y, x)) == self.selected_cell {
                    draw_rectangle(x1, y1, cell_size, cell_size, Color::new(0.2, 0.4, 0.8, 0.25));
                }
            }
        }

        // Drawing black lines between cells
        for y in 0..=self.params.n {
            let x1 = map_x_offset;
            let y1 = y as f32 * cell_size + map_y_offset;
            draw_line(x1, y1, x1 + map_width, y1, 3.0, BLACK);
        }
        for x in 0..=self.params.m {
            let x1 = x as f32 * cell_size + map_x_offset;
            let y1 = map_y_offset;
            draw_line(x1, y1, x1, y1 + map_height, 3.0, BLACK);
        }
    }

    fn process_input(&mut self) {
        if is_key_released(KeyCode::E) {
            self.selected_cell = None;
            return;
        }
        if is_key_released(KeyCode::Q) {
            self.moves_queue.clear();
            return;
        }
        if let Some((cy, cx)) = self.selected_cell {
            let delta = if is_key_released(KeyCode::W) {
                Some((usize::MAX, 0))
            } else if is_key_released(KeyCode::S) {
                Some((1, 0))
            } else if is_key_released(KeyCode::A) {
                Some((0, usize::MAX))
            } else if is_key_released(KeyCode::D) {
                Some((0, 1))
            } else {
                None
            };
            if let Some((dy, dx)) = delta {
                let ny = cy.wrapping_add(dy);
                let nx = cx.wrapping_add(dx);
                let mv = Move::new(cy, cx, ny, nx);
                if self.map.could_become_a_valid_move(mv) {
                    self.moves_queue.push_back(mv); 
                }
                self.selected_cell = Some((ny, nx));
            }
        }
        if !is_mouse_button_released(MouseButton::Left) {
            return;
        }
        let cell_size = (self.params.screen_height / self.params.n as f32).min(self.params.screen_width / self.params.m as f32) * 0.95;
        let map_height = cell_size * self.params.n as f32;
        let map_width = cell_size * self.params.m as f32;
        let map_y_offset = (self.params.screen_height - map_height) * 0.5;
        let map_x_offset = (self.params.screen_width - map_width) * 0.5;
        let mut selected_cell = None;
        let (mouse_x, mouse_y) = mouse_position();
        for y in 0..self.params.n {
            for x in 0..self.params.m {
                let y1 = map_y_offset + cell_size * y as f32;
                let x1 = map_x_offset + cell_size * x as f32;
                if x1 <= mouse_x && mouse_x < x1 + cell_size && y1 <= mouse_y && mouse_y < y1 + cell_size {
                    selected_cell = Some((y, x));
                }
            }
        }
        let Some(selected_cell) = selected_cell else {
            return;
        };
        // First selection
        if self.selected_cell.is_none() {
            self.selected_cell = Some(selected_cell);
            return;
        }
        // Second selection
        let mv = Move {
            from: self.selected_cell.unwrap(),
            to: selected_cell,
        };
        if self.map.could_become_a_valid_move(mv) {
            self.moves_queue.push_back(mv); 
        }
        self.selected_cell = Some(selected_cell);
    }

    fn next_tick(&mut self) {
        for id in 0..self.params.players_num {
            assert_eq!(self.map.curr_color, id);
            if id == self.player_color {
                let Some(&next_move) = self.moves_queue.front() else {
                    self.map.skip_turn();
                    continue;
                };
                self.moves_queue.pop_front().unwrap();
                if self.map.is_a_valid_move(next_move) {
                    self.map.make_move(next_move);
                } else {
                    self.map.skip_turn();
                    println!("Incorrect move {:?}", next_move);
                }
            } else {
                let bot = &mut self.bots[id];
                let Some(best_move) = bot.get_best_move() else {
                    self.map.skip_turn();
                    continue;
                };
                if self.map.is_a_valid_move(best_move) {
                    self.map.make_move(best_move);
                } else {
                    println!("Bad bot move! {:?}", best_move);
                    self.map.skip_turn();
                }
            }
        }
        for y in 0..self.map.n {
            for x in 0..self.map.m {
                self.map.grid[y][x].last_update_time = self.map.turn;
            }
        }
        for id in 0..self.params.players_num {
            if id != self.player_color {
                self.bots[id].update_from_map(&self.map);
            }
        }
    }
}

impl Scene for GameScene {
    fn process_frame_and_get_next_scene(&mut self) -> Option<Box<dyn Scene>> {
        let mut next_scene: Option<Box<dyn Scene>> = None;
        self.params.update_screen_info();
        egui_macroquad::ui(|egui_ctx| {
            egui_ctx.set_pixels_per_point(self.params.screen_min_res * UI_SCALE_COEFFICIENT * self.params.ui_scale);
            Window::new("Меню")
                .show(egui_ctx, |ui| {
                    if ui.button("Новая игра").clicked() {
                        next_scene = Some(Box::new(MenuScene { params: self.params.clone() }));
                    }
                    ui.checkbox(&mut self.params.disable_fog_of_war, "Отключить туман войны")
                });
        });

        self.draw_game_map();
        self.process_input();

        if get_time() - self.last_tick_time > DELAY_BETWEEN_TICKS {
            self.next_tick();
            self.last_tick_time = get_time();
        }
        
        egui_macroquad::draw();
        next_scene
    }
}

#[derive(Default)]
struct MenuScene {
    params: GameParams,
}

impl Scene for MenuScene {
    fn process_frame_and_get_next_scene(&mut self) -> Option<Box<dyn Scene>> {
        let mut next_scene: Option<Box<dyn Scene>> = None;
        self.params.update_screen_info();
        egui_macroquad::ui(|egui_ctx| {
            egui_ctx.set_pixels_per_point(self.params.screen_min_res * UI_SCALE_COEFFICIENT * self.params.ui_scale);
            Window::new("Игра \"Колонизатор\"")
                .anchor(Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .collapsible(false)
                .resizable(false)
                .show(egui_ctx, |ui| {
                    ui.label("Ширина поля");
                    ui.add(Slider::new(&mut self.params.m, 10..=50));
                    ui.label("Высота поля");
                    ui.add(Slider::new(&mut self.params.n, 10..=50));
                    ui.label("Количество игроков");
                    ui.add(Slider::new(&mut self.params.players_num, 2..=16));
                    // Ui scale slider
                    ui.label("Масштаб интерфейса");
                    let response = ui.add(Slider::new(&mut self.params.new_ui_scale, 0.3..=2.0));
                    if response.drag_released() {
                        self.params.ui_scale = self.params.new_ui_scale;
                    }
                    // Start game button
                    if ui.button("Начать игру!").clicked() {
                        next_scene = Some(Box::new(GameScene::new(self.params.clone())));
                    }
                });
        });

        egui_macroquad::draw();
        next_scene
    }
}

pub struct App {
    scene: Box<dyn Scene>,
}

impl App {
    pub fn new() -> App {
        App {
            scene: Box::new(MenuScene::default()),
        }
    }

    pub fn process_frame(&mut self) {
        if let Some(next_scene) = self.scene.process_frame_and_get_next_scene() {
            self.scene = next_scene;
        }
    }
}
