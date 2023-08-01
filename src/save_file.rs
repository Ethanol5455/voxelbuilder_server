use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::{fs, io};

use unwrap_or::unwrap_ok_or;

use crate::player_data::Player;

use crate::vector_types::{Vec2, Vec3};
use crate::world::chunk_column::CompressedSet;
use crate::world::{BlockToPlace, Chunk};

const DEFAULT_SCRIPT_SUBDIRECTORY: &str = "/default_scripts";
const SAVE_FILE_NAME: &str = "worldData";
const SAVE_FILE_EXTENSION: &str = "vbdat";
const PLAYER_SAVE_SUBDIRECTORY: &str = "/players";
const SCRIPT_SAVE_SUBDIRECTORY: &str = "/scripts";

pub struct ChunkInfo {
    pub position: Vec3<i32>,
    pub data: Vec<CompressedSet>,
}

pub struct SaveFile {
    // save_directory does not contain trailing slashes, if None do not save
    pub save_directory: Option<String>,
    pub world_seed: i32,
    chunk_data: Vec<ChunkInfo>,
    block_to_place: Vec<BlockToPlace>,
    players: HashMap<String, Player>,
}

impl SaveFile {
    pub fn new(directory: Option<String>) -> SaveFile {
        match directory.clone() {
            Some(dir) => {
                assert!(!dir.is_empty(), "Empty save directory entered!");
                assert!(
                    !dir.contains('\\'),
                    "Save directory may not contain \\ characters"
                );
                assert!(
                    !dir.ends_with('/'),
                    "Save directory may not end with / character"
                );
                unwrap_ok_or!(SaveFile::generate_save_structure(dir), e, {
                    panic!("Unable to generate save directory structure: {}", e)
                });
            }
            None => (),
        }

        SaveFile {
            save_directory: directory,
            world_seed: rand::random(),
            chunk_data: Vec::<ChunkInfo>::new(),
            block_to_place: Vec::<BlockToPlace>::new(),
            players: HashMap::new(),
        }
    }

    fn generate_save_structure(directory: String) -> io::Result<()> {
        fs::create_dir_all(format!("{}{}", directory, SCRIPT_SAVE_SUBDIRECTORY))?;
        fs::create_dir_all(format!("{}{}", directory, PLAYER_SAVE_SUBDIRECTORY))?;

        let script_files = ["loadAssetInfo.lua", "generateChunkColumn.lua"];

        for script_file in script_files {
            let to_path_str = format!("{}{}/{}", directory, SCRIPT_SAVE_SUBDIRECTORY, script_file);
            if !Path::new(&to_path_str).exists() {
                fs::copy(
                    format!(".{}/{}", DEFAULT_SCRIPT_SUBDIRECTORY, script_file),
                    to_path_str,
                )?;
            }
        }

        Ok(())
    }

    pub fn get_script_path(&self, script_name: String) -> String {
        match self.save_directory.clone() {
            Some(directory) => format!(
                "{}{}/{}.lua",
                directory, SCRIPT_SAVE_SUBDIRECTORY, script_name
            ),
            None => format!(".{}/{}.lua", DEFAULT_SCRIPT_SUBDIRECTORY, script_name),
        }
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
        if self.save_directory.is_none() {
            eprintln!("Save directory not provided, save will not be written");
            return;
        }
        let directory_str = self.save_directory.clone().unwrap();

        // Player data
        for (username, player) in &self.players {
            let mut file = unwrap_ok_or!(
                File::create(format!(
                    "{}{}/{}.{}",
                    directory_str, PLAYER_SAVE_SUBDIRECTORY, username, SAVE_FILE_EXTENSION
                )),
                e,
                {
                    eprintln!(
                        "Unable to write save file for player \"{}\" with error \"{}\"",
                        username, e
                    );
                    continue;
                }
            );
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
        let file = File::create(format!(
            "{}/{}.{}",
            directory_str, SAVE_FILE_NAME, SAVE_FILE_EXTENSION
        ));
        if file.is_err() {
            println!(
                "Unable to create world save file with error \"{}\"",
                file.err().unwrap()
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

    pub fn load(&mut self) -> io::Result<()> {
        // Load saved users
        assert!(self.save_directory.is_some(), "Cannot load temporary save!");
        let directory_str = self.save_directory.clone().unwrap();

        match fs::read_dir(format!("{}{}", directory_str, PLAYER_SAVE_SUBDIRECTORY)) {
            Ok(contents) => {
                for entry in contents {
                    let lines = BufReader::new(File::open(entry?.path()).unwrap()).lines();
                    // self.get_user_data(username);
                    let mut player = None;
                    for line in lines {
                        if let Ok(line) = line {
                            let (key, value) = line.split_at(line.find(' ').unwrap());
                            let value = value.trim();
                            match key {
                                "PlayerName" => {
                                    player = Some(self.get_user_data(&value.to_string()))
                                }
                                "PlayerPosition" => {
                                    let coords: Vec<&str> =
                                        value.split_ascii_whitespace().collect();
                                    player.as_mut().unwrap().position = Vec3::new(
                                        coords[0].parse().unwrap(),
                                        coords[1].parse().unwrap(),
                                        coords[2].parse().unwrap(),
                                    );
                                }
                                "PlayerRotation" => {
                                    let coords: Vec<&str> =
                                        value.split_ascii_whitespace().collect();
                                    player.as_mut().unwrap().rotation = Vec2::new(
                                        coords[0].parse().unwrap(),
                                        coords[1].parse().unwrap(),
                                    );
                                }
                                _ => {
                                    assert!(
                                        player.is_some(),
                                        "PlayerName must be the first line in player save file!"
                                    );
                                    eprintln!("Unknown player save file key \"{}\"", key);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Unable to open player save files with error \"{}\".", e),
        }

        // Load world
        let file = File::open(directory_str + "/" + SAVE_FILE_NAME + "." + SAVE_FILE_EXTENSION);
        if file.is_err() {
            println!(
                "Unable to load world save file with error \"{}\". The save may not be generated yet...",
                file.err().unwrap()
            );
            // return;
            panic!();
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

        Ok(())
    }
}
