use chunk::Cell;
use chunk::Chunk;
use chunk::ChunkGrid;

mod chunk;

fn main() {
    let chunk_grid = ChunkGrid::new();


    let root_chunk = Chunk::new();
    println!("Hello, world!");
}

fn next_generation(cell: Cell, chunk_grid: &ChunkGrid) -> bool {
    let mut neighbor_counter = 0;

    for y in -1..1 {
        for x in -1..1 {
            if x == 0 && y == 0 {
                continue;
            }

            if chunk_grid.get_cell(Cell{x, y}) == Some(true) {
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