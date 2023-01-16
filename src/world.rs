pub mod chunk_column;
pub use chunk_column::{ChunkColumn,Chunk};

use std::collections::BTreeMap;

use crate::items::ItemManager;

use crate::save_file::SaveFile;
use crate::vector_types::{Vec2, Vec3};

pub struct BlockToPlace {
    pub column_position: Vec2<i32>,
    pub position_in_column: Vec3<i32>,
    pub block_id: i32,
}

pub struct World {
    column_map: BTreeMap<i32, BTreeMap<i32, ChunkColumn>>,
    item_manager: ItemManager,
}

impl World {
    /// Creates a new world with no chunks
    pub fn new(item_manager: ItemManager) -> World {
        World {
            column_map: BTreeMap::new(),
            item_manager,
        }
    }

    /// Generates a new column at the given position (`x`,`y`)
    fn generate_column(&mut self, pos: &Vec2<i32>) {
        let mut col = ChunkColumn::new(pos, 1);

        // TODO: Insert generation code here
        for i in 4..16 {
            col.get_chunk(i).fill(0);
        }
        

        if !self.column_map.contains_key(&pos.x) {
            self.column_map.insert(pos.x, BTreeMap::new());
        }

        self.column_map.get_mut(&pos.x)
        .unwrap()
        .insert(pos.y, col);
    }

    /// Returns whether the column at `pos` exists
    pub fn does_column_exist(&self, pos: &Vec2<i32>) -> bool {
        self.column_map.contains_key(&pos.x) && self.column_map.get(&pos.x).unwrap().contains_key(&pos.y)
    }

    /// Gets the column at `pos` and generates the column if it doesn't exist
    pub fn get_column(&mut self, pos: &Vec2<i32>) -> &mut ChunkColumn {

        if !self.does_column_exist(pos) {
            self.generate_column(pos);
        }

        self.column_map.get_mut(&pos.x).unwrap().get_mut(&pos.y).unwrap()
    }

    /// Translates absolute world position to absolute column position
    pub fn world_to_column_position(pos: &Vec2<i32>) -> Vec2<i32> {
        let mut column_position = Vec2::new(pos.x / 16, pos.y / 16);

        if pos.x < 0 && -pos.x % 16 != 0 {
            column_position.x -= 1;
        }
        if pos.y < 0 && -pos.y % 16 != 0 {
            column_position.y -= 1;
        }

        column_position
    }

    /// Translates absolute world position to absolute chunk position
    fn world_to_chunk_position(pos: &Vec3<i32>) -> Vec3<i32> {
        let mut chunk_position = Vec3::new(pos.x / 16, pos.y / 16, pos.z / 16);

        if pos.x < 0 && -pos.x % 16 != 0 {
            chunk_position.x -= 1;
        }
        if pos.y < 0 && -pos.y % 16 != 0 {
            chunk_position.y -= 1;
        }
        if pos.z < 0 && -pos.z % 16 != 0 {
            chunk_position.z -= 1;
        }

        chunk_position
    }

    /// Translates absolute world position into internal chunk position
    pub fn world_to_position_in_chunk(pos: &Vec3<i32>) -> Vec3<i32> {
        let mut block_pos_in_chunk = Vec3::new(pos.x % 16, pos.y % 16, pos.z % 16);

        if block_pos_in_chunk.x < 0 {
            block_pos_in_chunk.x += 16;
        }
        if block_pos_in_chunk.y < 0 {
            block_pos_in_chunk.y += 16;
        }
        if block_pos_in_chunk.z < 0 {
            block_pos_in_chunk.z += 16;
        }

        block_pos_in_chunk
    }

    /// Gets the block at `pos`
    pub fn get_block(&mut self, position: &Vec3<i32>) -> i32 {

        let chunk_position = World::world_to_chunk_position(position);
        let block_position_in_chunk = World::world_to_position_in_chunk(position);

        if chunk_position.y >= 0 && chunk_position.y <= 15 && 
            self.does_column_exist(&Vec2::new(chunk_position.x, chunk_position.z)) {
            let column = self.get_column(&Vec2::new(chunk_position.x, chunk_position.z));
            return column.get_chunk(chunk_position.y as u8)
                .get_block(block_position_in_chunk.x as u8, block_position_in_chunk.y as u8, block_position_in_chunk.z as u8);
        }

        -1
    }

    /// Sets the block at `pos` to `id`
    pub fn set_block(&mut self, position: &Vec3<i32>, id: i32) {
        if self.item_manager.get_item_by_id(id).is_none() {
            println!("Tried to set block of unknown id {}", id);
            return;
        }

        let chunk_position = World::world_to_chunk_position(position);
        let block_position_in_chunk = World::world_to_position_in_chunk(position);

        if self.does_column_exist(&Vec2::new(chunk_position.x, chunk_position.z)) {
            self.get_column(&Vec2::new(chunk_position.x, chunk_position.z))
            .get_chunk(chunk_position.y as u8)
            .set_block(block_position_in_chunk.x as u8, block_position_in_chunk.y as u8, block_position_in_chunk.z as u8, id);
        }
    }

    pub fn save_to_file(&self, save: &mut crate::save_file::SaveFile) {

        if save.filepath == "" {
            return;
        }

        println!("Saving world data");
        for column_x in self.column_map.values() {
            for column_z in column_x.values() {
                for chunk in column_z.get_chunks() {
                    save.save_chunk_data(chunk);
                }
            }
        }

        // TODO: Save block_to_place


        println!("Writing save file");
        save.write_save();
        println!("Save file written");

    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_column_position() {
        
        // Positive
        for x in 0..48 {
            for z in 0..48 {
                assert_eq!(World::world_to_column_position(&Vec2::new(x, z)), Vec2::new(x / 16, z / 16));
            }
        }

        // Negative
        for x in -49..0 {
            for z in -49..0 {
                let mut correct_x = x / 16;
                if x < 0 && -x % 16 != 0 {
                    correct_x -= 1;
                }
                let mut correct_z = z / 16;
                if z < 0 && -z % 16 != 0 {
                    correct_z -= 1;
                }
                assert_eq!(World::world_to_column_position(&Vec2::new(x, z)), Vec2::new(correct_x, correct_z));
            }
        }
    }

    #[test]
    fn test_world_to_chunk_position() {
        
        // Positive
        for x in 0..48 {
            for y in 0..48 {
                for z in 0..48 {
                    assert_eq!(World::world_to_chunk_position(&Vec3::new(x, y, z)), Vec3::new(x / 16, y / 16, z / 16));
                }
            }
        }

        // Negative
        for x in -49..0 {
            for y in -49..0 {
                for z in -49..0 {
                    let mut correct_x = x / 16;
                    if x < 0 && -x % 16 != 0 {
                        correct_x -= 1;
                    }
                    let mut correct_y = y / 16;
                    if y < 0 && -y % 16 != 0 {
                        correct_y -= 1;
                    }
                    let mut correct_z = z / 16;
                    if z < 0 && -z % 16 != 0 {
                        correct_z -= 1;
                    }
                    assert_eq!(World::world_to_chunk_position(&Vec3::new(x, y, z)), Vec3::new(correct_x, correct_y, correct_z));
                }
            }
        }
    }

    #[test]
    fn test_world_to_position_in_chunk() {

        // Chunk at 0,0,0
        for x in 0..16 {
            for y in 0..16 {
                for z in 0..16 {
                    assert_eq!(World::world_to_position_in_chunk(&Vec3::new(x, y, z)), Vec3::new(x, y, z));
                }
            }
        }

        // Chunk at 16,16,16
        for x in 16..32 {
            for y in 16..32 {
                for z in 16..32 {
                    assert_eq!(World::world_to_position_in_chunk(&Vec3::new(x, y, z)), Vec3::new(x - 16, y - 16, z - 16));
                }
            }
        }

        // Chunk at -15,-15,-15
        for x in -16..0 {
            for y in -16..0 {
                for z in -16..0 {
                    assert_eq!(World::world_to_position_in_chunk(&Vec3::new(x, y, z)), Vec3::new(x + 16, y + 16, z + 16));
                }
            }
        }
        
    }
}
