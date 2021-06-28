use std::collections::HashMap;

pub struct Grid {
    first_hash_map: HashMap<(i64, i64), bool>,
    second_hash_map: HashMap<(i64, i64), bool>,
    generation: bool
}

impl Grid {
    pub fn new() -> Grid {
        return Grid {
            first_hash_map: HashMap::new(),
            second_hash_map: HashMap::new(),
            generation: false
        }
    }

    pub fn get_grid(&self, row: i64, col: i64, len: usize) -> Vec<bool> {
        let mut result: Vec<bool> = Vec::with_capacity(len*len);
        for r in row..(row + len as i64) {
            for c in col..(col + len as i64) {
                result.push(self.get_cell(r, c));
            }
        }

        return result
    }

    pub fn calc_next_generation(&mut self) {
        if self.generation {
            for ((row, col), val) in &self.first_hash_map {
                if *val {
                    for neighbor_row in -1..2 {
                        for neighbor_col in -1..2 {
                            if neighbor_col == 0 && neighbor_row == 0 {
                                continue;
                            }
                                
                            let neighbor_row_index = row + neighbor_row;
                            let neighbor_col_index = col + neighbor_col;
                            
                            if !self.get_cell(neighbor_row_index,  neighbor_col_index) {
                                let neighbor_next_value = self.get_cell_next_generation(neighbor_row_index,  neighbor_col_index, false);
                                set_cell_in_hashmap(neighbor_row_index,  neighbor_col_index, neighbor_next_value, &mut self.second_hash_map);
                            }
                        }
                    }
                }
                    
                let next_value = self.get_cell_next_generation(*row, *col, *val);
                set_cell_in_hashmap(*row, *col, next_value, &mut self.second_hash_map);
            }
            self.first_hash_map.clear();
        } else {
            for ((row, col), val) in &self.second_hash_map {
                if *val {
                    for neighbor_row in -1..2 {
                        for neighbor_col in -1..2 {
                            if neighbor_col == 0 && neighbor_row == 0 {
                                continue;
                            }
                            
                            let neighbor_row_index = row + neighbor_row;
                            let neighbor_col_index = col + neighbor_col;
                            
                            if !self.get_cell(neighbor_row_index,  neighbor_col_index) {
                                let neighbor_next_value = self.get_cell_next_generation(neighbor_row_index,  neighbor_col_index, false);
                                set_cell_in_hashmap(neighbor_row_index,  neighbor_col_index, neighbor_next_value, &mut self.first_hash_map);
                            }
                        }
                    }
                }
                
                let next_value = self.get_cell_next_generation(*row, *col, *val);
                set_cell_in_hashmap(*row, *col, next_value, &mut self.first_hash_map);
            }
            self.second_hash_map.clear();
        }
        

        self.generation = !self.generation;
    }

    pub fn get_cell(&self, row: i64, col: i64) -> bool {
        if self.generation {
            return get_cell_from_hashmap(row, col, &self.first_hash_map)
        } else {
            return get_cell_from_hashmap(row, col, &self.second_hash_map)
        }
    }

    pub fn set_cell(&mut self, row: i64, col: i64, value: bool) {
        if self.generation {
            set_cell_in_hashmap(row, col, value, &mut self.first_hash_map);
        } else {
            set_cell_in_hashmap(row, col, value, &mut self.second_hash_map);
        }
    }

    fn get_cell_next_generation(&self, row: i64, col: i64, val: bool) -> bool {
        let mut neighbor_counter = 0;
    
        for neighbor_row in -1..2 {
            for neighbor_col in -1..2 {
                if neighbor_col == 0 && neighbor_row == 0 {
                    continue;
                }
                
                if self.get_cell(row + neighbor_row,  col + neighbor_col) {
                    neighbor_counter += 1;
                }
    
                if neighbor_counter > 3 {
                    return false;
                }
            }
        }

        if !val {
            return neighbor_counter == 3
        }
    
        if neighbor_counter < 2 {
            return false
        }

        return true;
    }
}

fn get_cell_from_hashmap(row: i64, col: i64, map: &HashMap<(i64, i64), bool>) -> bool {
    match map.get(&(row, col)) {
        Some(b) => *b,
        None => false
    }
}

fn set_cell_in_hashmap(row: i64, col: i64, value: bool, map: &mut HashMap<(i64, i64), bool>) {
    if value {
        map.insert((row, col), value);
    } else {
        map.remove(&(row, col));
    }
}