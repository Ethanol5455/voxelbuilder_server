use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, Write};

use crate::player_data::Player;

use crate::vector_types::{Vec2, Vec3};
use crate::world::chunk_column::CompressedSet;
use crate::world::{BlockToPlace, Chunk};

pub struct ChunkInfo {
    pub position: Vec3<i32>,
    pub data: Vec<CompressedSet>,
}

pub struct SaveFile {
    pub filepath: String,
    pub world_seed: i32,
    chunk_data: Vec<ChunkInfo>,
    block_to_place: Vec<BlockToPlace>,
    players: HashMap<String, Player>,
}

impl SaveFile {
    pub fn load_save_file(path: String) -> SaveFile {
        let mut save = SaveFile {
            filepath: path,
            world_seed: rand::random(),
            chunk_data: Vec::<ChunkInfo>::new(),
            block_to_place: Vec::<BlockToPlace>::new(),
            players: HashMap::new(),
        };

        save.read_save();

        save
    }

    pub fn get_chunk(&self, position: Vec3<i32>) -> Option<&ChunkInfo> {
        for chunk in self.chunk_data.as_slice() {
            if chunk.position == position {
                return Some(chunk);
            }
        }

        None
    }

    pub fn get_user_data(&mut self, username: &String) -> &mut Player {
        if self.players.contains_key(username) {
            return self.players.get_mut(username).unwrap();
        }

        let player = Player {
            username: username.to_string(),
            position: Vec3::new(0.0, 80.0, 0.0),
            rotation: Vec2::new(0.0, 0.0),
        };
        self.players.insert(username.to_string(), player);

        self.players.get_mut(username).unwrap()
    }

    pub fn save_chunk_data(&mut self, chunk: &Chunk) {
        let data = ChunkInfo {
            position: chunk.position,
            data: chunk.compress(),
        };

        let mut remove_index = usize::MAX;
        for i in 0..self.chunk_data.len() {
            if self.chunk_data[i].position == chunk.position {
                remove_index = i;
            }
        }

        if remove_index != usize::MAX {
            self.chunk_data.remove(remove_index);
        }

        self.chunk_data.push(data)
    }

    pub fn write_save(&self) {
        if self.filepath == "" {
            println!("Save path not provided, file will not be written");
        }

        let dir_result = fs::create_dir_all(self.filepath.to_string() + "/players");
        if dir_result.is_err() {
            println!(
                "Unable to create save directory structure with error \"{}\".",
                dir_result.err().unwrap()
            );
            return;
        }

        // Player data
        for (username, player) in &self.players {
            let filepath = self.filepath.to_string() + "/players/" + username.as_str() + ".vbdat";
            let file = File::create(filepath);
            if file.is_err() {
                println!(
                    "Unable to write save file for player \"{}\" with error \"{}\"",
                    username,
                    file.err().unwrap()
                );
                continue;
            }
            let mut file = file.unwrap();
            // Player Name
            let name_string = "PlayerName ".to_string() + username + "\n";
            file.write(name_string.as_bytes()).unwrap();
            // Player Position
            let pos_string = "PlayerPosition ".to_string()
                + &player.position.x.to_string()
                + " "
                + &player.position.y.to_string()
                + " "
                + &player.position.z.to_string()
                + "\n";
            file.write(pos_string.as_bytes()).unwrap();
            // Player Rotation
            let rot_string = "PlayerRotation ".to_string()
                + &player.rotation.x.to_string()
                + " "
                + &player.rotation.y.to_string()
                + "\n";
            file.write(rot_string.as_bytes()).unwrap();
        }

        // World data
        let file = File::create(self.filepath.to_string() + "/worldData.vbdat");
        if file.is_err() {
            println!(
                "Unable to create world save file with error \"{}\"",
                dir_result.err().unwrap()
            );
            return;
        }
        let mut file = file.unwrap();

        // World seed
        file.write(("Seed ".to_string() + &self.world_seed.to_string() + "\n").as_bytes())
            .unwrap();

        // Compressed chunk data
        for chunk in &self.chunk_data {
            // Chunk Position
            let pos_string = "C ".to_string()
                + &chunk.position.x.to_string()
                + " "
                + &chunk.position.y.to_string()
                + " "
                + &chunk.position.z.to_string()
                + " ";
            file.write(pos_string.as_bytes()).unwrap();

            // Compressed data
            for set in &chunk.data {
                file.write((set.id.to_string() + " " + &set.count.to_string() + " ").as_bytes())
                    .unwrap();
            }

            // End of chunk data
            file.write("-111\n".as_bytes()).unwrap();
        }

        // Block to place
        for block in &self.block_to_place {
            file.write(
                ("N ".to_string()
                    + &block.column_position.x.to_string()
                    + " "
                    + &block.column_position.y.to_string()
                    + " "
                    + &block.position_in_column.x.to_string()
                    + " "
                    + &block.position_in_column.y.to_string()
                    + " "
                    + &block.position_in_column.z.to_string()
                    + " "
                    + &block.block_id.to_string()
                    + "\n")
                    .as_bytes(),
            )
            .unwrap();
        }
    }

    fn read_save(&mut self) {
        // Load saved users
        let mut user_files = Vec::<String>::new();

        let dir_iter = fs::read_dir(self.filepath.to_string() + "/players");
        if dir_iter.is_err() {
            println!(
                "Unable to open player save files with error \"{}\". The save may not be generated yet...",
                dir_iter.as_ref().err().unwrap()
            );
        } else {
            let dir_iter = dir_iter.unwrap();
            for e in dir_iter.into_iter() {
                user_files.push(String::from(e.unwrap().file_name().to_str().unwrap()));
            }
        }

        for user in user_files {
            let file = File::open(self.filepath.to_string() + "/players/" + &user).unwrap();
            let lines = std::io::BufReader::new(file).lines();
            let username = user.split_at(user.find('.').unwrap()).0.to_string();
            let mut user = self.get_user_data(&username);
            for line in lines {
                if let Ok(line) = line {
                    let mut values = line.split_at(line.find(' ').unwrap());
                    values.1 = values.1.trim();
                    if values.0 == "PlayerName" {
                        user.username = String::clone(&username);
                    } else if values.0 == "PlayerPosition" {
                        let x_pair = values.1.split_at(values.1.find(' ').unwrap());
                        let y_pair = x_pair.1.trim().split_at(x_pair.1.trim().find(' ').unwrap());
                        user.position.x = x_pair.0.parse().unwrap();
                        user.position.y = y_pair.0.parse().unwrap();
                        user.position.z = y_pair.1.trim().parse().unwrap();
                    } else if values.0 == "PlayerRotation" {
                        let x_pair = values.1.split_at(values.1.find(' ').unwrap());
                        user.rotation.x = x_pair.0.parse().unwrap();
                        user.rotation.y = x_pair.1.trim().parse().unwrap();
                    }
                }
            }
        }

        // Load world
        let file = File::open(self.filepath.to_string() + "/worldData.vbdat");
        if file.is_err() {
            println!(
                "Unable to load world save file with error \"{}\". The save may not be generated yet...",
                file.err().unwrap()
            );
            return;
        }
        let file = file.unwrap();

        let lines = std::io::BufReader::new(file).lines();
        for line in lines {
            if let Ok(line) = line {
                let mut key_val = line.split_at(line.find(' ').unwrap());
                key_val.1 = key_val.1.trim();

                if key_val.0 == "Seed" {
                    self.world_seed = key_val.1.parse().unwrap();
                } else if key_val.0 == "C" {
                    let x_pair = key_val.1.split_at(key_val.1.find(' ').unwrap());
                    let y_pair = x_pair.1.trim().split_at(x_pair.1.trim().find(' ').unwrap());
                    let z_pair = y_pair.1.trim().split_at(y_pair.1.trim().find(' ').unwrap());
                    let position = Vec3::<i32>::new(
                        x_pair.0.parse().unwrap(),
                        y_pair.0.parse().unwrap(),
                        z_pair.0.parse().unwrap(),
                    );

                    let mut data = ChunkInfo {
                        position,
                        data: Vec::new(),
                    };

                    let mut compressed_data_str = z_pair.1.trim();
                    loop {
                        if compressed_data_str == "-111" {
                            break;
                        }
                        let id_pair =
                            compressed_data_str.split_at(compressed_data_str.find(' ').unwrap());
                        let number_pair = id_pair
                            .1
                            .trim()
                            .split_at(id_pair.1.trim().find(' ').unwrap());
                        compressed_data_str = number_pair.1.trim();
                        let compressed_set = CompressedSet {
                            id: id_pair.0.parse().unwrap(),
                            count: number_pair.0.parse().unwrap(),
                        };
                        data.data.push(compressed_set);
                    }

                    self.chunk_data.push(data);
                } else if key_val.0 == "N" {
                    let x_pos_pair = key_val.1.split_at(key_val.1.find(' ').unwrap());
                    let y_pos_pair = x_pos_pair
                        .1
                        .trim()
                        .split_at(x_pos_pair.1.trim().find(' ').unwrap());

                    let column_position = Vec2::<i32>::new(
                        x_pos_pair.0.parse().unwrap(),
                        y_pos_pair.0.parse().unwrap(),
                    );

                    let x_pos_pair = y_pos_pair
                        .1
                        .trim()
                        .split_at(y_pos_pair.1.trim().find(' ').unwrap());
                    let y_pos_pair = x_pos_pair
                        .1
                        .trim()
                        .split_at(x_pos_pair.1.trim().find(' ').unwrap());
                    let z_pos_pair = y_pos_pair
                        .1
                        .trim()
                        .split_at(y_pos_pair.1.trim().find(' ').unwrap());
                    let position_in_column = Vec3::<i32>::new(
                        x_pos_pair.0.parse().unwrap(),
                        y_pos_pair.0.parse().unwrap(),
                        z_pos_pair.0.parse().unwrap(),
                    );

                    let block_id = z_pos_pair.1.trim().parse::<i32>().unwrap();

                    let to_place = BlockToPlace {
                        column_position,
                        position_in_column,
                        block_id,
                    };

                    self.block_to_place.push(to_place);
                }
            }
        }
    }
}
