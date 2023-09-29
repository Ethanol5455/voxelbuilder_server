mod items;

mod player_data;

mod save_file;
use cgmath::Vector2;
use save_file::SaveFile;

use anyhow::Result;

use enet::*;
use std::net::Ipv4Addr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{env, str};

mod packets;
use voxelbuilder_common::{ChunkUpdateType, PacketType};

use crate::packets::*;

mod world;
use world::World;

struct GameOptions {
    init_only: bool,
}

impl GameOptions {
    pub fn new() -> Self {
        GameOptions { init_only: false }
    }

    pub fn parse_cli(mut self) -> Self {
        let args: Vec<String> = env::args().collect();

        self.init_only = args.contains(&"--no_run".to_string());

        self
    }
}

struct Game {
    options: GameOptions,

    server: Host<()>,

    world: World,
}

impl Game {
    pub fn new() -> Result<Self> {
        let options = GameOptions::new().parse_cli();
        let enet = Enet::new().unwrap();
        let address = Address::new(Ipv4Addr::UNSPECIFIED, 1234);
        let server = enet
            .create_host::<()>(
                Some(&address),
                1,
                ChannelLimit::Limited(2),
                BandwidthLimit::Unlimited,
                BandwidthLimit::Unlimited,
            )
            .unwrap();

        let save_directory = "./save";
        let mut save = SaveFile::new(Some(save_directory.to_owned()));
        if save.load().is_err() {
            eprintln!("Save file could not be loaded with error \"{}\". The save file may not be generated yet!", save.load().unwrap_err());
        }

        let mut item_manager = items::ItemManager::new();
        item_manager.load_items(save.get_script_path("loadAssetInfo".to_string()));

        let world = World::new(item_manager, save);

        Ok(Game {
            options,
            server,
            world,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        if self.options.init_only {
            return Ok(());
        }
        println!("Running...");

        let term = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGINT, Arc::clone(&term)).unwrap();

        while !term.load(Ordering::Relaxed) {
            match self.server.service(1000).unwrap() {
                Some(Event::Connect(_)) => println!("Connected!"),
                Some(Event::Disconnect(..)) => {
                    println!("Disconnected!");
                }
                Some(Event::Receive {
                    ref mut sender,
                    channel_id,
                    ref packet,
                    ..
                }) => {
                    let data = packet.data();
                    if data[0] == PacketType::PlayerInfoRequest as u8 {
                        // [0: Type][1-(n-1): username][n: '\0']
                        let username = str::from_utf8(&data[1..(data.len() - 1)]).unwrap();
                        let player = self
                            .world
                            .get_save_file()
                            .get_user_data(&username.to_string());

                        let packet_data = assemble_player_info_data(&player);
                        let packet =
                            Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
                        sender.send_packet(packet, channel_id).unwrap();
                    } else if data[0] == PacketType::PlayerInfoData as u8 {
                        // [0: Type][1-12: position][13-20: rotation][21-: username]
                        let username = str::from_utf8(&data[21..(data.len() - 1)]).unwrap();
                        let position = bincode::deserialize(&data[1..13]).unwrap();
                        let rotation = bincode::deserialize(&data[13..21]).unwrap();

                        let player = self
                            .world
                            .get_save_file()
                            .get_user_data(&username.to_string());
                        player.position = position;
                        player.rotation = rotation;
                    } else if data[0] == PacketType::ChunkRequest as u8 {
                        // [0: Type][1-4: column X][5-8: column Z]
                        let col_x = bincode::deserialize(&data[1..5]).unwrap();
                        let col_z = bincode::deserialize(&data[5..9]).unwrap();
                        let col = self.world.get_column(&Vector2::new(col_x, col_z));

                        let packet_data = assemble_chunk_contents_packet(col);
                        let packet =
                            Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
                        sender.send_packet(packet, channel_id).unwrap();
                    } else if data[0] == PacketType::ChunkUpdate as u8 {
                        let block_pos = bincode::deserialize(&data[1..13]).unwrap();
                        let action_type = data[13];

                        let existing_id = self.world.get_block(&block_pos);
                        if action_type == ChunkUpdateType::PlaceBlockEvent as u8 {
                            if existing_id > 0 {
                                println!(
                                    "Cannot place block over id {} @ {},{},{}",
                                    existing_id, block_pos.x, block_pos.y, block_pos.z
                                );
                            } else {
                                let block_id: u32 = bincode::deserialize(&data[14..18]).unwrap();
                                self.world.set_block(&block_pos, block_id as i32);
                            }
                        } else if action_type == ChunkUpdateType::DestroyBlockEvent as u8 {
                            if existing_id < 1 {
                                println!(
                                    "Cannot destroy empty block id {} @ {},{},{}",
                                    existing_id, block_pos.x, block_pos.y, block_pos.z
                                );
                            } else {
                                self.world.set_block(&block_pos, 0);
                            }
                        } else {
                            println!(
                                "Received unknown chunk update type with value {}",
                                action_type
                            );
                        }

                        let col_position = World::world_to_column_position(&Vector2::new(
                            block_pos.x,
                            block_pos.z,
                        ));
                        let packet_data =
                            assemble_chunk_contents_packet(self.world.get_column(&col_position));
                        let packet =
                            Packet::new(&packet_data, PacketMode::ReliableSequenced).unwrap();
                        sender.send_packet(packet, channel_id).unwrap();
                    } else {
                        println!("Unknown packet id: {}", data[0])
                    }
                }
                _ => (),
            }
        }

        Ok(())
    }

    pub fn shutdown(&mut self) -> Result<()> {
        self.world.save_to_file();

        Ok(())
    }
}

fn main() -> Result<()> {
    let mut game = Game::new()?;
    game.run()?;
    game.shutdown()
}
