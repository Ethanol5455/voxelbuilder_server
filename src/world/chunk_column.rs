use crate::vector_types::{Vec3, Vec2};

pub struct CompressedSet {
    pub id: i32,
    pub number: i32,
}

pub struct Chunk {
    pub position: Vec3<i32>,
    blocks: [i32; 4096],
}

impl Chunk {
    /// Creates a new chunk and fills it with the block of id
    pub fn new(position: Vec3<i32>, id: i32) -> Chunk {
        Chunk {
            position,
            blocks: [id; 4096],
        }
    }

    /// Fills the chunk with the given block id
    pub fn fill(&mut self, id: i32) {
        self.blocks = [id; 4096];
    }

    fn xyz_to_i(x: u8, y: u8, z: u8) -> u16{
        256 * z as u16 + 16 * y as u16 + x as u16
    }

    /// Sets the block at position `i` to `id`
    pub fn set_block_i(&mut self, i: u16, id: i32) {
        self.blocks[i as usize] = id;
    }

    /// Sets the block at position (`x`,`y`,`z`) to `id`
    pub fn set_block(&mut self, x: u8, y: u8, z: u8, id: i32) {
        self.blocks[Chunk::xyz_to_i(x, y, z) as usize] = id;
    }

    /// Gets the block at position `i`
    pub fn get_block_i(&self, i: u16) -> i32 {
        self.blocks[i as usize]
    }

    /// Gets the block at position (`x`,`y`,`z`)
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> i32 {
        self.blocks[Chunk::xyz_to_i(x, y, z) as usize]
    }

    /// Compresses the chunk data using run-length encoding
    pub fn compress(&self) -> Vec<CompressedSet> {
        let mut set = Vec::<CompressedSet>::new();

        let mut number = 0;
        let mut id = 0;

        for i in 0..4096 {
            let block = self.blocks[i];
            if block == id {
                number += 1;
            } else {
                if id != -1 {
                    let new_set = CompressedSet {
                        id,
                        number
                    };
                    set.push(new_set);
                }
                id = block;
                number = 1;
            }

            if i == 4095 && id != -1 {
                let new_set = CompressedSet {
                    id,
                    number
                };
                set.push(new_set);
            }
        }

        set
    }
}

pub struct ChunkColumn {
    chunks: Vec<Chunk>,
}

impl ChunkColumn {
    /// Creates a new ChunkColumn filled with `id` (16 chunks tall)
    pub fn new(position: &Vec2<i32>, id: i32) -> ChunkColumn {
        let mut col = ChunkColumn {
            chunks: Vec::new(),
        };

        for y in 0..16 {
            col.chunks.push(Chunk::new(Vec3::new(position.x, y, position.y), id));
        }

        col
    }

    pub fn get_chunks(&self) -> &Vec<Chunk>{
        &self.chunks
    }

    pub fn get_chunks_mut(&mut self) -> &mut Vec<Chunk>{
        &mut self.chunks
    }

    pub fn get_chunk<'a>(&'a mut self, i: i8) -> &'a mut Chunk {
        &mut self.chunks[i as usize]
    }
}
