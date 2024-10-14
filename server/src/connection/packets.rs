use shared::utils::{codec::BinaryCodec, vec2::Vector2D};
use tokio::sync::MutexGuard;

use crate::{game::entity::{Entity, InputFlags}, server::Server};

pub fn handle_spawn_packet(
    full_server: &mut MutexGuard<'_, Server>, id: u32, mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();

    let name = codec.decode_string().ok_or(true)?;

    if game_server.get_entity(id).is_none() {
        let mut entity = Entity::default();
        entity.nametag.name = name;

        game_server.insert_entity(entity);
    }

    Ok(())
}

pub fn handle_input_packet(
    full_server: &mut MutexGuard<'_, Server>, id: u32, mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();

    let flags = codec.decode_varuint().ok_or(true)? as u32;
    let mouse = Vector2D::new(
        codec.decode_f32().ok_or(true)?, 
        codec.decode_f32().ok_or(true)?
    );

    if let Some(entity) = game_server.get_mut_entity(id) {
        entity.physics.inputs = InputFlags::new(flags);
        entity.physics.mouse = mouse;
    }

    Ok(())
}