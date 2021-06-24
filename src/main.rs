use chunk::Cell;
use chunk::Chunk;
use chunk::ChunkGrid;

mod chunk;

fn main() {
    let chunk_grid = ChunkGrid::new();
    chunk_grid.compute_next_generation(true);
}