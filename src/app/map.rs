pub mod cell;

use cell::{CellType, GameCell};

use crate::constants::DIRECTIONS;

#[derive(Clone, Copy, Debug)]
pub struct Move {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

impl Move {
    pub fn new(y1: usize, x1: usize, y2: usize, x2: usize) -> Move {
        Move {
            from: (y1, x1),
            to: (y2, x2),
        }
    }
}

#[derive(Default, Clone, Eq, PartialEq)]
pub struct PlayerStatistics {
    pub total_army: i32,
    pub total_fields: i32,
}

#[derive(Default, Clone)]
pub struct GameMap {
    pub n: usize,
    pub m: usize,
    pub players_num: usize,
    pub curr_color: usize,
    pub turn: u32,
    pub grid: Vec<Vec<GameCell>>,
}

impl GameMap {
    pub fn new_random(n: usize, m: usize, k: usize) -> GameMap {
        let mut grid = vec![vec![GameCell::default(); m]; n];
        for y in 0..n {
            for x in 0..m {
                if fastrand::f32() < 0.15 {
                    grid[y][x].cell_type = CellType::Mountains;
                    if !Self::is_connected(n, m, &grid) {
                        grid[y][x].cell_type = CellType::Empty;
                    }
                }
            }
        }
        for id in 0..k {
            loop {
                let (y, x) = (fastrand::usize(0..n), fastrand::usize(0..m));
                let cell = &mut grid[y][x];
                if cell.is_empty_not_owned() {
                    cell.owner = Some(id);
                    cell.cell_type = CellType::General;
                    cell.army_size = 1;
                    break;
                }
            }
        }
        for y in 0..n {
            for x in 0..m {
                if fastrand::f32() < 0.05 && grid[y][x].is_empty_not_owned() {
                    grid[y][x].cell_type = CellType::City;
                    grid[y][x].army_size = fastrand::i64(20..=50);
                    if !Self::is_connected(n, m, &grid) {
                        grid[y][x].cell_type = CellType::Empty;
                    }
                }
            }
        }
        GameMap {
            n,
            m,
            players_num: k,
            curr_color: 0,
            grid,
            turn: 0,
        }
    }

    fn is_connected(n: usize, m: usize, grid: &Vec<Vec<GameCell>>) -> bool {
        let mut used = vec![vec![false; m]; n];
        let mut start_cell = (0, 0);
        'a: for y in 0..n {
            for x in 0..m {
                if grid[y][x].cell_type != CellType::Mountains {
                    start_cell = (y, x);
                    break 'a;
                }
            }
        }
        let mut st = vec![start_cell];
        used[start_cell.0][start_cell.1] = true;
        while !st.is_empty() {
            let &(y, x) = st.last().unwrap();
            st.pop();
            for (dy, dx) in DIRECTIONS {
                let ny = y.wrapping_add(dy);
                let nx = x.wrapping_add(dx);
                if ny < n && nx < m && !used[ny][nx] && grid[ny][nx].cell_type != CellType::Mountains {
                    st.push((ny, nx));
                    used[ny][nx] = true;
                }
            }
        }
        for y in 0..n {
            for x in 0..m {
                if !used[y][x] && grid[y][x].cell_type != CellType::Mountains {
                    return false;
                }
            }
        }
        true
    }

    fn next_turn(&mut self) {
        self.turn += 1;
        for y in 0..self.n {
            for x in 0..self.m {
                let cell = &mut self.grid[y][x];
                if cell.owner.is_some() && (self.turn % 50 == 0 || cell.city_or_general() && self.turn % 2 == 0) {
                    cell.army_size += 1;
                }
            }
        }
    }

    fn destroy_player(&mut self, player_id: usize, new_owner: Option<usize>) {
        for y in 0..self.n {
            for x in 0..self.m {
                let cell = &mut self.grid[y][x];
                if cell.owner == Some(player_id) {
                    cell.owner = new_owner;
                }
            }
        }
    }

    pub fn make_move(&mut self, game_move: Move) {
        let Move { from: (y1, x1), to: (y2, x2) } = game_move;
        let mut cell1 = self.grid[y1][x1];
        let mut cell2 = self.grid[y2][x2];
        if cell2.owner == cell1.owner {
            cell2.army_size += cell1.army_size - 1;
        } else {
            cell2.army_size -= cell1.army_size - 1;
            if cell2.army_size < 0 {
                cell2.army_size *= -1;
                if cell2.cell_type == CellType::General {
                    self.destroy_player(cell2.owner.expect("General must have an owner"), cell1.owner);
                    cell2.cell_type = CellType::City;
                }
                cell2.owner = cell1.owner;
            }
        }
        cell1.army_size = 1;
        self.grid[y1][x1] = cell1;
        self.grid[y2][x2] = cell2;
        self.skip_turn();
    }

    pub fn skip_turn(&mut self) {
        self.curr_color += 1;
        if self.curr_color >= self.players_num {
            self.curr_color = 0;
            self.next_turn();
        }
    }

    pub fn could_become_a_valid_move(&self, m: Move) -> bool {
        let Move {
            from: (y1, x1),
            to: (y2, x2),
        } = m;
        if y2 >= self.n || x2 >= self.m || x1.abs_diff(x2) + y1.abs_diff(y2) != 1 {
            return false;
        }
        let from = self.grid[y1][x1];
        let to = self.grid[y2][x2];
        from.last_seen_type != CellType::Mountains && to.last_seen_type != CellType::Mountains
    }

    pub fn is_a_valid_move(&self, m: Move) -> bool {
        let Move {
            from: (y1, x1),
            to: (y2, x2),
        } = m;
        if y2 >= self.n || x2 >= self.m || x1.abs_diff(x2) + y1.abs_diff(y2) != 1 {
            return false;
        }
        let from = self.grid[y1][x1];
        let to = self.grid[y2][x2];
        from.army_size > 1
            && from.cell_type != CellType::Mountains
            && from.owner == Some(self.curr_color)
            && to.cell_type != CellType::Mountains
            && from.last_update_time == self.turn
    }

    pub fn get_all_moves(&self) -> Vec<Move> {
        let mut all_moves = vec![];
        for y in 0..self.n {
            for x in 0..self.m {
                let from = &self.grid[y][x];
                if from.army_size <= 1
                    || from.cell_type == CellType::Mountains
                    || from.owner != Some(self.curr_color)
                {
                    continue;
                }
                for (dy, dx) in DIRECTIONS {
                    let ny = y.wrapping_add(dy);
                    let nx = x.wrapping_add(dx);
                    if ny >= self.n || nx >= self.m {
                        continue;
                    }
                    let to = &self.grid[ny][nx];
                    if to.cell_type == CellType::Mountains {
                        continue;
                    }
                    all_moves.push(Move::new(y, x, ny, nx));
                }
            }
        }
        all_moves
    }

    pub fn is_visible_to(&self, y: usize, x: usize, id: usize) -> bool {
        for dy in (-1)..=1 {
            for dx in (-1)..=1 {
                let ny = (y as i32 + dy) as usize;
                let nx = (x as i32 + dx) as usize;
                if ny >= self.n || nx >= self.m {
                    continue;
                }
                if self.grid[ny][nx].owner == Some(id) {
                    return true;
                }
            }
        }
        false
    }

    pub fn get_with_fog(&self, y: usize, x: usize, id: usize) -> GameCell {
        let cell = self.grid[y][x];
        if self.is_visible_to(y, x, id) {
            return cell;
        }
        if cell.cell_type == CellType::Mountains || cell.cell_type == CellType::City {
            return GameCell {
                army_size: 0,
                owner: None,
                cell_type: CellType::Mountains,
                is_friend: false,
                last_seen_type: cell.last_seen_type,
                last_update_time: cell.last_update_time,
            };
        } else {
            return GameCell {
                army_size: 0,
                owner: None,
                cell_type: CellType::Empty,
                is_friend: false,
                last_seen_type: cell.last_seen_type,
                last_update_time: cell.last_update_time,
            };
        }
    }

    pub fn update_from(&mut self, other: &GameMap) {
        let old_grid = self.grid.clone();
        let old_color = self.curr_color;
        *self = other.clone();
        self.curr_color = old_color;
        for y in 0..self.n {
            for x in 0..self.m {
                let visible = self.is_visible_to(y, x, self.curr_color);
                self.grid[y][x] = self.get_with_fog(y, x, self.curr_color);
                let cell = &mut self.grid[y][x];
                if visible {
                    cell.last_update_time = self.turn;
                    cell.last_seen_type = cell.cell_type;
                } else if old_grid[y][x].last_update_time > 0 {
                    cell.last_update_time = old_grid[y][x].last_update_time;
                    cell.last_seen_type = old_grid[y][x].last_seen_type;
                } else {
                    cell.last_seen_type = cell.cell_type;
                }
            }
        }
    }
}
