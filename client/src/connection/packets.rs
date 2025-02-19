use std::fmt::Binary;

use gloo::console::console;
use gloo_utils::window;
use shared::{connection::packets::{ClanPacketOpcode, ServerboundPackets}, game::{body::BodyIdentityIds, entity::{InputFlags, Notification}, turret::TurretIdentityIds}, normalize_angle, utils::{codec::BinaryCodec, color::Color, consts::ARENA_SIZE, vec2::Vector2D}};

use crate::{game::entity::base::{Entity, HealthState}, storage_set, world::{get_world, World}};

pub fn form_spawn_packet(
    name: String
) -> BinaryCodec {
    storage_set!("last_name", &name);

    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Spawn as u64);

    codec.encode_string(name);

    codec
}

pub fn form_input_packet(
    flags: InputFlags,
    mouse: Vector2D
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

pub fn form_upgrade_packet(upgrade_type: usize, tank: usize) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Upgrade as u64);
    codec.encode_varuint(upgrade_type as u64);
    codec.encode_varuint(tank as u64);

    codec
}

pub fn form_ping_packet() -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Ping as u64);

    codec
}

pub fn form_chat_packet(typing: Option<bool>, message: String) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Chat as u64);

    if let Some(typing) = typing {
        codec.encode_varuint(0);
        codec.encode_bool(typing);
    } else {
        codec.encode_varuint(1);
        codec.encode_string(message);
    }

    codec
}

pub fn form_clan_packet_create(name: String, description: String, max_members: u64) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ServerboundPackets::Clan as u64);
    codec.encode_varuint(ClanPacketOpcode::Create as u64);

    codec.encode_string(name);
    codec.encode_string(description);
    codec.encode_varuint(max_members);

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

pub fn handle_notification_packet(
    world: &mut World,
    mut codec: BinaryCodec
) {
    let length = codec.decode_varuint().unwrap();
    for _ in 0..length {
        let message = codec.decode_string().unwrap();
        let (r, g, b) = (codec.decode_varuint().unwrap(), codec.decode_varuint().unwrap(), codec.decode_varuint().unwrap());
        let lifetime = codec.decode_varuint().unwrap();
        
        if message.contains("You killed") {
            world.game.self_entity.display.kills += 1;
        }

        world.game.self_entity.display.notifications.push(Notification {
            message,
            color: Color(r as u8, g as u8, b as u8),
            lifetime,
            ..Default::default()
        });
    }
}

pub fn handle_server_info_packet(
    world: &mut World,
    mut codec: BinaryCodec
) {
    let mspt = codec.decode_f32().unwrap();
    world.connection.mspt.target = mspt;

    let leaderboard_length = codec.decode_varuint().unwrap() as usize;
    world.game.leaderboard.entries = Vec::with_capacity(leaderboard_length);

    for _ in 0..leaderboard_length {
        let score = codec.decode_varuint().unwrap() as usize;
        let name = codec.decode_string().unwrap();
        let body_identity: BodyIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
        let turret_identity: TurretIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();

        world.game.leaderboard.entries.push((score, name, body_identity, turret_identity));
    }

    let angle = codec.decode_f32().unwrap();

    if angle != -13.0 {
        world.game.leaderboard.angle.target = normalize_angle!(angle + std::f32::consts::PI);
    }

    world.game.leaderboard.arrow_opacity.target = if angle == -13.0 { 0.0 } else { 1.0 };
}