use std::collections::VecDeque;

use crate::constants::DIRECTIONS;

use super::map::{GameMap, Move};
use super::map::cell::{CellType, GameCell};

pub trait Bot {
    fn get_best_move(&mut self, strength: f64) -> Option<Move>;

    fn update_from_map(&mut self, map: &GameMap);
}

#[derive(Default, Clone, Copy)]
struct VertexData {
    pub dist: i64,
    pub value: i64,
    pub coords: (usize, usize),
    pub parent: (usize, usize),
}

impl VertexData {
    pub fn merge(&mut self, b: VertexData) -> bool {
        let add_to_queue = b.dist < self.dist;
        if (b.dist, -b.value) < (self.dist, -self.value) {
            self.dist = b.dist;
            self.value = b.value;
            self.parent = b.parent;
        }
        add_to_queue
    }
}

const INF: i64 = 1e10 as i64;

#[derive(Default)]
pub struct PathFinderBot {
    pub map: GameMap,
}

impl PathFinderBot {
    pub fn from_map(map: &GameMap, color: usize) -> Self {
        let mut map = map.clone();
        map.curr_color = color;
        map.grid = vec![vec![GameCell::default(); map.m]; map.n];
        Self { map }
    }

    fn find_paths(&self, map: &GameMap, start: (usize, usize)) -> Vec<Vec<VertexData>> {
        let mut result = vec![vec![VertexData::default(); map.m]; map.n];
        for y in 0..map.n {
            for x in 0..map.m {
                result[y][x] = VertexData {
                    dist: INF,
                    value: 0,
                    coords: (y, x),
                    parent: (usize::MAX, usize::MAX),
                };
            }
        }
        result[start.0][start.1].dist = 0;
        result[start.0][start.1].value = map.grid[start.0][start.1].army_size;
        let mut queue = VecDeque::new();
        queue.push_back(start);
        while let Some((y, x)) = queue.pop_front() {
            if result[y][x].value <= 1 {
                continue;
            }
            for (dy, dx) in DIRECTIONS {
                let ny = y.wrapping_add(dy);
                let nx = x.wrapping_add(dx);
                if !map.could_become_a_valid_move(Move::new(y, x, ny, nx)) {
                    continue;
                }
                let mut value_delta = 0;
                let to = map.grid[ny][nx];
                let new_dist = result[y][x].dist + 1;
                let army_size = to.army_after_time(map, new_dist);
                if to.owner == Some(map.curr_color) {
                    value_delta += army_size;
                } else {
                    value_delta -= army_size;
                }
                let new_value = result[y][x].value - 1 + value_delta;
                if new_value <= 1 {
                    continue;
                }
                if result[ny][nx].merge(VertexData {
                    dist: new_dist,
                    value: new_value,
                    coords: (ny, nx),
                    parent: (y, x),
                }) {
                    queue.push_back((ny, nx));
                }
            }
        }
        result
    }

    fn eval_target_cell(
        &self,
        map: &GameMap,
        coords: (usize, usize),
    ) -> f64 {
        let cell = map.grid[coords.0][coords.1];
        if cell.cell_type == CellType::Mountains || cell.is_friend || cell.last_update_time != self.map.turn {
            return -1e9;
        }
        let priority = if cell.owner == None {
            // Without owner
            match cell.cell_type {
                CellType::Empty => 6.0,
                CellType::City => 250.0,
                CellType::General => unreachable!(),
                CellType::Mountains => unreachable!(),
            }
        } else if cell.owner != Some(map.curr_color) {
            // Enemy
            match cell.cell_type {
                CellType::Empty => 100.0,
                CellType::City => 1500.0,
                CellType::General => 1e18,
                CellType::Mountains => unreachable!(),
            }
        } else {
            // Me
            -1e9
        };
        priority
    }

    fn get_all_moves(&self) -> Vec<Move> {
        let mut moves = vec![];
        for y in 0..self.map.n {
            for x in 0..self.map.m {
                for (dy, dx) in DIRECTIONS {
                    let ny = y.wrapping_add(dy);
                    let nx = x.wrapping_add(dx);
                    let mv = Move::new(y, x, ny, nx);
                    if self.map.is_a_valid_move(mv) {
                        moves.push(mv);
                    }
                }
            }
        }
        moves
    }

    fn get_random_move(&self) -> Option<Move> {
        let moves = self.get_all_moves();
        fastrand::choice(&moves).copied()
    }
}

impl Bot for PathFinderBot {
    fn get_best_move(&mut self, strength: f64) -> Option<Move> {
        if fastrand::f64() * 100.0 > strength {
            return self.get_random_move();
        }
        let mut best_score = -1e9;
        let mut best_move = None;
        let mut start_cells = vec![];
        for y in 0..self.map.n {
            for x in 0..self.map.m {
                let cell = self.map.grid[y][x];
                if cell.owner != Some(self.map.curr_color) || cell.army_size <= 1 || cell.last_update_time != self.map.turn {
                    continue;
                }
                let mut priority = cell.army_size;
                if cell.cell_type == CellType::General && cell.owner == Some(self.map.curr_color) {
                    priority = ((priority as f64 - 10.0) * 0.5) as i64;
                }
                start_cells.push((-priority, fastrand::u32(0..=u32::MAX), y, x));
            }
        }
        start_cells.sort_unstable();
        for &(_, _, y, x) in &start_cells[..5.min(start_cells.len())] {
            let grid = self.find_paths(&self.map, (y, x));
            for y1 in 0..self.map.n {
                for x1 in 0..self.map.m {
                    let priority = self.eval_target_cell(&self.map, (y1, x1));
                    let info = grid[y1][x1];
                    if info.value < 1 {
                        continue;
                    };
                    let score = priority / info.dist as f64;
                    if score > best_score {
                        let mut curr_coords = info.coords;
                        if curr_coords == (y, x) || info.parent.0 == usize::MAX {
                            continue;
                        }
                        let new_move = loop {
                            let prev_coords = grid[curr_coords.0][curr_coords.1].parent;
                            if prev_coords == (y, x) {
                                let (y2, x2) = curr_coords;
                                let curr_move = Move::new(y, x, y2, x2);
                                break curr_move;
                            }
                            curr_coords = prev_coords;
                        };
                        assert!(self.map.is_a_valid_move(new_move));
                        best_move = Some(new_move);
                        best_score = score;
                    }
                }
            }
        }
        best_move
    }
    
    fn update_from_map(&mut self, map: &GameMap) {
        self.map.update_from(map);
    }
}
