use super::GameMap;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum CellType {
    Empty,
    Mountains,
    City,
    General,
}

impl Default for CellType {
    fn default() -> Self {
        Self::Empty
    }
}

#[derive(Default, Clone, Copy)]
pub struct GameCell {
    pub army_size: i64,
    pub owner: Option<usize>,
    pub cell_type: CellType,
    pub is_friend: bool,
    pub last_seen_type: CellType,
    pub last_update_time: u32,
}
 
impl GameCell {
    pub fn is_empty_not_owned(&self) -> bool {
        self.cell_type == CellType::Empty && self.owner.is_none()
    }

    pub fn city_or_general(&self) -> bool {
        self.cell_type == CellType::City || self.cell_type == CellType::General
    }

    pub fn army_after_time(&self, map: &GameMap, path_len: i64) -> i64 {
        self.army_size + match self.cell_type {
            CellType::City | CellType::General => path_len as i64 + (map.turn - self.last_update_time) as i64,
            CellType::Empty | CellType::Mountains => 0,
        }
    }
}