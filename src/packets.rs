#[allow(dead_code)]
pub enum PacketType {
    PlayerConnect,
    PlayerDisconnect,
    PlayerInfoRequest, // Get saved player data from file (if available)
    PlayerInfoData,    // Data about a player to save, sent at a fixed interval from the client
    ChunkRequest,      // Request from the client to send data about a chunk
    ChunkUpdate,       // Request from the client to update a chunk
    ChunkContents,     // The contents of a chunk as requested by the client
                       // TODO: Add server message to client // Send a message from the server to the client
                       // TODO: Add client command to server // Send a command from the client to the server
}

pub enum ChunkUpdateType {
    PlaceBlockEvent,
    DestroyBlockEvent,
}

use crate::{player_data::Player, world::ChunkColumn};

pub fn assemble_player_info_data(player: &Player) -> Vec<u8> {
    let mut packet_data = Vec::<u8>::new();

    packet_data.push(PacketType::PlayerInfoData as u8);

    // username
    for c in player.username.as_bytes() {
        packet_data.push(*c);
    }
    packet_data.push('\0' as u8);

    // position
    let mut pos = bincode::serialize(&player.position).unwrap();
    packet_data.append(&mut pos);

    // rotation
    let mut rot = bincode::serialize(&player.rotation).unwrap();
    packet_data.append(&mut rot);

    packet_data
}

pub fn assemble_chunk_contents_packet(col: &mut ChunkColumn) -> Vec<u8> {
    let mut packet_data = Vec::<u8>::new();

    packet_data.push(PacketType::ChunkContents as u8);

    let chunks = col.get_chunks();
    for chunk in chunks {
        let mut pos = bincode::serialize(&chunk.position).unwrap();
        packet_data.append(&mut pos);

        let compressed_data = chunk.compress();

        for set in compressed_data {
            let mut id = bincode::serialize(&set.id).unwrap();
            packet_data.append(&mut id);
            let mut count = bincode::serialize(&set.count).unwrap();
            packet_data.append(&mut count);
        }

        let mut end_indicator = bincode::serialize(&(-1)).unwrap();
        packet_data.append(&mut end_indicator);
    }

    packet_data
}
