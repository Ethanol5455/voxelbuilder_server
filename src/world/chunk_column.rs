use crate::vector_types::{Vec3, Vec2};

pub struct CompressedSet {
    pub id: i32,
    pub count: i32,
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
    // pub fn fill(&mut self, id: i32) {
    //     self.blocks = [id; 4096];
    // }

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
    // pub fn get_block_i(&self, i: u16) -> i32 {
        // self.blocks[i as usize]
    // }

    /// Gets the block at position (`x`,`y`,`z`)
    pub fn get_block(&self, x: u8, y: u8, z: u8) -> i32 {
        self.blocks[Chunk::xyz_to_i(x, y, z) as usize]
    }

    /// Compresses the chunk data using run-length encoding
    pub fn compress(&self) -> Vec<CompressedSet> {
        let mut set = Vec::<CompressedSet>::new();

        let mut number = 0;
        let mut id = -1;

        for i in 0..4096 {
            let block = self.blocks[i];
            if block == id {
                number += 1;
            } else {
                if id != -1 {
                    let new_set = CompressedSet {
                        id,
                        count: number
                    };
                    set.push(new_set);
                }
                id = block;
                number = 1;
            }

            if i == 4095 && id != -1 {
                let new_set = CompressedSet {
                    id,
                    count: number
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

    pub fn get_chunk<'a>(&'a mut self, i: u8) -> &'a mut Chunk {
        &mut self.chunks[i as usize]
    }

    pub fn set_block(&mut self, position: &Vec3<i32>, id: i32) {
        self.chunks.get_mut((position.y / 16) as usize).unwrap().set_block(position.x as u8, (position.y % 16) as u8, position.z as u8, id);
    }

    pub fn set_layers(&mut self, lower: u32, upper: u32, id: i32) {
        for y in lower..(upper + 1) {
            for x in 0..16 {
                for z in 0..16 {
                    self.chunks.get_mut((y / 16) as usize).unwrap().set_block(x, (y % 16) as u8, z, id);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xyz_to_i() {
        let mut i = 0;
        for z in 0..16 {
            for y in 0..16 {
                for x in 0..16 {
                    assert_eq!(Chunk::xyz_to_i(x, y, z), i);
                    i += 1;
                }
            }
        }
    }

    #[test]
    fn test_chunk_compression() {
        let mut chunk = Chunk::new(Vec3::new(0, 0, 0), 0);
        // All air
        let set = chunk.compress();
        assert_eq!(set.len(), 1);
        assert_eq!(set[0].count, 4096);
        assert_eq!(set[0].id, 0);

        // Single block
        chunk.set_block_i(0, 1);
        let set = chunk.compress();
        assert_eq!(set.len(), 2);
        assert_eq!(set[0].count, 1);
        assert_eq!(set[0].id, 1);
        assert_eq!(set[1].count, 4095);
        assert_eq!(set[1].id, 0);

        // Alternating double
        for i in 0..4096 {
            if i % 4 == 0 || i % 4 == 1 {
                chunk.set_block_i(i, 1);
            }
        }

        let set = chunk.compress();
        assert_eq!(set.len(), 2048);
        for i in 0..2048 {
            assert_eq!(set[i].count, 2);
            if i % 2 == 1 {
                assert_eq!(set[i].id, 0);
            } else {
                assert_eq!(set[i].id, 1);
            }
        }
    }
}
