pub const CHUNK_LENGTH: usize = 100;

pub struct Cell {
    pub x: i64,
    pub y: i64
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
    grid: [[bool; CHUNK_LENGTH]; CHUNK_LENGTH],
}

type ChunkRef<'a> = Option<Box<&'a Chunk>>;
type ChunkRowDimension<'a> = Vec<ChunkRef<'a>>;
type ChunkRow<'a> = Vec<Option<(Option<ChunkRowDimension<'a>>, Option<ChunkRowDimension<'a>>)>>;

impl Chunk {
    pub fn new() -> Box<Chunk> {
        return Box::new(Chunk {
            grid: [[false; CHUNK_LENGTH]; CHUNK_LENGTH],
        })
    }
}

pub struct ChunkGrid<'a> {
    positive_chunk_rows: ChunkRow<'a>,
    negative_chunk_rows: ChunkRow<'a>
}

impl ChunkGrid<'_> {
    pub fn new<'a>() -> ChunkGrid<'a> {
        return ChunkGrid {
            positive_chunk_rows: vec![Some((Some(Vec::new()), Some(Vec::new())))],
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

        let row = match &y_dimension[chunk_y] {
            Some(t) => t,
            None => {
                return None;
            }
        };

        let x_dimension = match cell.x {
            x if x >= 0 => &row.1,
            y if y < 0 => &row.0,
            _ => panic!("Keine Ahnung man!")
        };

        let arm = match x_dimension {
            Some(x) => x,
            None => {
                return None;
            }
        };

        let chunk_x = cell.absolute_x() / CHUNK_LENGTH;

        let chunk = match &arm[chunk_x] {
            Some(c) => c,
            None => {
                return None;
            }
        };
        

        return Some(false);
    }
}

fn absolute_val(val: i64) -> usize {
    match val {
        val if val >= 0 => val as usize,
        val if val < 0 => -(val + 1) as usize,
        _ => panic!("Keine Ahnung man!")
    }
}