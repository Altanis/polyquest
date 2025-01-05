use gloo::console::console;
use shared::{connection::packets::ServerboundPackets, game::entity::InputFlags, utils::{codec::BinaryCodec, consts::ARENA_SIZE, vec2::Vector2D}};

use crate::{game::entity::base::{Entity, HealthState}, world::{get_world, World}};

pub fn form_spawn_packet(
    name: String
) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Spawn as u64);

    codec.encode_string(name);

    codec
}

pub fn form_input_packet(
    flags: InputFlags,
    mouse: Vector2D<f32>
) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Input as u64);

    codec.encode_varuint(flags.get_value() as u64);
    codec.encode_f32(mouse.x);
    codec.encode_f32(mouse.y);

    codec
}

pub fn form_stats_packet(stat: usize) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Stats as u64);
    codec.encode_varuint(stat as u64);

    codec
}

pub fn form_upgrade_packet(upgrade_type: usize, stat: usize) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Upgrade as u64);
    codec.encode_varuint(upgrade_type as u64);
    codec.encode_varuint(stat as u64);

    codec
}

pub fn handle_update_packet(
    world: &mut World,
    mut codec: BinaryCodec
) {
    // parse world information later
    world.game.arena_size = ARENA_SIZE;

    Entity::parse_census(world, &mut codec, true);

    let entities = codec.decode_varuint().unwrap();
    let mut entity_ids = Vec::with_capacity(entities as usize);

    for _ in 0..entities {
        entity_ids.push(Entity::parse_census(world, &mut codec, false));
    }

    let mut deletion_ids = vec![];
    for (&id, entity) in world.game.surroundings.iter_mut().filter(|(id, _)| !entity_ids.contains(id)) {
        match entity.stats.health_state {
            HealthState::Dead => deletion_ids.push(id),
            _ => entity.stats.health_state = HealthState::Dying
        }
    }

    world.game.surroundings.retain(|id, _| !deletion_ids.contains(id));
}