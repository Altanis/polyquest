use std::cell::RefMut;

use shared::{connection::packets::{ClientboundPackets, ServerboundPackets}, utils::{codec::BinaryCodec, vec2::Vector2D}};
use tokio::sync::MutexGuard;

use crate::{game::{entity::{Entity, InputFlags}, state::EntityDataStructure}, server::Server};

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
    full_server: &mut MutexGuard<'_, Server>, 
    id: u32, 
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();

    let flags = codec.decode_varuint().ok_or(true)? as u32;
    let mouse = Vector2D::new(
        codec.decode_f32().ok_or(true)?, 
        codec.decode_f32().ok_or(true)?
    );

    if let Some(mut entity) = game_server.get_entity(id) {
        entity.physics.inputs = InputFlags::new(flags);
        entity.physics.mouse = mouse;
    }

    Ok(())
}

pub fn form_update_packet(
    self_entity: &RefMut<'_, Entity>, 
    entities: &EntityDataStructure
) -> BinaryCodec {
    let mut codec = BinaryCodec::new();

    codec.encode_varuint(ClientboundPackets::Update as u64);

    self_entity.take_census(&mut codec, false);

    codec.encode_varuint((entities.len() - 1) as u64);
    for (id, entity) in entities.iter() {
        if self_entity.id == *id { continue; }

        let entity = &entity.borrow_mut();
        entity.take_census(&mut codec, false);
    }

    codec
}