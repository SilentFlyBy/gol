pub const CHUNK_LENGTH: usize = 100;

pub struct Cell {
    pub x: i64,
    pub y: i64,
    pub current: bool
}

impl Cell {
    pub fn absolute_x(&self) -> usize {
        return absolute_val(self.x);
    }
    
    pub fn absolute_y(&self) -> usize {
        return absolute_val(self.y);
    }
}

pub struct Chunk {
    grid: [[(bool, bool); CHUNK_LENGTH]; CHUNK_LENGTH],
}

type ChunkRef = Option<Box<Chunk>>;
type ChunkRowDimension = Vec<ChunkRef>;
type ChunkRow = Vec<Option<(Option<ChunkRowDimension>, Option<ChunkRowDimension>)>>;

impl Chunk {
    pub fn new() -> Box<Chunk> {
        return Box::new(Chunk {
            grid: [[(false, false); CHUNK_LENGTH]; CHUNK_LENGTH],
        })
    }
}

pub struct ChunkGrid {
    positive_chunk_rows: ChunkRow,
    negative_chunk_rows: ChunkRow
}

impl ChunkGrid {
    pub fn new() -> ChunkGrid {
        return ChunkGrid {
            positive_chunk_rows: vec![Some((Some(Vec::new()), Some(vec![Some(Chunk::new())])))],
            negative_chunk_rows: vec![Some((Some(Vec::new()), Some(Vec::new())))]
        }
    }

    pub fn get_cell(&self, cell: Cell) -> Option<bool> {
        let y_dimension = match cell.y {
            y if y >= 0 => &self.positive_chunk_rows,
            y if y < 0 => &self.negative_chunk_rows,
            _ => panic!("Keine Ahnung man!")
        };

        let chunk_y = cell.absolute_y() / CHUNK_LENGTH;

        if chunk_y >= y_dimension.len() {
            return None;
        }

        let row = match &y_dimension[chunk_y] {
            Some(t) => t,
            None => {
                return None;
            }
        };

        let x_dimension = match cell.x {
            x if x >= 0 => &row.1,
            y if y < 0 => &row.0,
            _ => panic!(r#"¯\_(ツ)_/¯"#)
        };

        let arm = match x_dimension {
            Some(x) => x,
            None => {
                return None;
            }
        };

        let chunk_x = cell.absolute_x() / CHUNK_LENGTH;

        if chunk_x >= arm.len() {
            return None
        }

        let chunk = match &arm[chunk_x] {
            Some(c) => c.as_ref(),
            None => {
                return None;
            }
        };

        let x = match cell.x {
            x if x >= 0 => x as usize - (CHUNK_LENGTH * chunk_x),
            x if x < 0 => (-x as usize) - (CHUNK_LENGTH * chunk_x),
            _ => panic!(r#"¯\_(ツ)_/¯"#)
        };

        let y = match cell.y {
            y if y >= 0 => y as usize - (CHUNK_LENGTH * chunk_y),
            y if y < 0 => (-y as usize) - (CHUNK_LENGTH * chunk_y),
            _ => panic!(r#"¯\_(ツ)_/¯"#)
        };

        let cell_result = chunk.grid[x][y];

        return if cell.current {Some(cell_result.0)} else {Some(cell_result.1)}
    }

    pub fn compute_next_generation(&self, current: bool) {
        self.row_dimension_next_generation(&self.positive_chunk_rows, current);
        self.row_dimension_next_generation(&self.negative_chunk_rows, current);
    }

    fn row_dimension_next_generation(&self, row_dimension: &ChunkRow, current: bool) {
        for pos_row in row_dimension {

            let row = match pos_row {
                Some(row) => row,
                None => {
                    continue;
                }
            };

            self.col_dimension_next_generation(&row.0, current);
            self.col_dimension_next_generation(&row.1, current);
        }
    }

    fn col_dimension_next_generation(&self, col_dimension: &Option<ChunkRowDimension>, current: bool) {
        if let Some(neg_col) = &col_dimension {
            for col in neg_col {
                if let Some(chunk) = col {
                    let mut grid = chunk.grid;
                    for (iy, y) in grid.iter_mut().enumerate() {
                        for (ix, x) in y.iter_mut().enumerate() {
                            let next_value = self.cell_next_generation(Cell{x: ix as i64, y: iy as i64, current});
                            if current { x.1 = next_value} else {x.0 = next_value}
                        }
                    }
                }
            }
        };
    }

    fn cell_next_generation(&self, cell: Cell) -> bool {
        let mut neighbor_counter = 0;
    
        for y in -1..1 {
            for x in -1..1 {
                if x == 0 && y == 0 {
                    continue;
                }
    
                if self.get_cell(Cell{x, y, current: cell.current}) == Some(true) {
                    neighbor_counter += 1;
                }
    
                if neighbor_counter > 3 {
                    return false;
                }
            }
        }
    
        if neighbor_counter < 2 {
            return false
        }
    
        return true;
    }
}

fn absolute_val(val: i64) -> usize {
    match val {
        val if val >= 0 => val as usize,
        val if val < 0 => -(val + 1) as usize,
        _ => panic!("Keine Ahnung man!")
    }
}