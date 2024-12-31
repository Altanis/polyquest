use std::cell::RefMut;
use shared::{connection::packets::ClientboundPackets, game::{body::{get_body_base_identity, BodyIdentity}, entity::{InputFlags, BASE_TANK_RADIUS, MAX_STAT_INVESTMENT}, turret::{get_turret_base_identity, TurretIdentity, TurretStructure}}, utils::{codec::BinaryCodec, vec2::Vector2D}};
use crate::{game::{entity::Entity, state::EntityDataStructure}, server::ServerGuard};

pub fn handle_spawn_packet(
    full_server: &mut ServerGuard, 
    id: u32, 
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();

    let name = codec.decode_string().ok_or(true)?;

    if let Some(mut entity) = game_server.get_entity(id) && !entity.stats.alive {
        entity.display.name = name;
        entity.stats.alive = true;
    
        entity.display.radius = BASE_TANK_RADIUS;
        entity.display.body_identity = get_body_base_identity();
        entity.display.turret_identity = get_turret_base_identity();
        entity.stats.health = entity.display.body_identity.max_health;
        entity.stats.max_health = entity.display.body_identity.max_health;

        entity.update_level(1); // todo: base it off prev lvl
    }

    Ok(())
}

pub fn handle_input_packet(
    full_server: &mut ServerGuard, 
    id: u32,
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();

    let flags = codec.decode_varuint().ok_or(true)? as u32;
    let mouse = Vector2D::new(
        codec.decode_f32().ok_or(true)?, 
        codec.decode_f32().ok_or(true)?
    );

    if let Some(mut entity) = game_server.get_entity(id) && entity.stats.alive {
        entity.physics.inputs = InputFlags::new(flags);
        entity.physics.mouse = mouse;
    }

    Ok(())
}

pub fn handle_stats_packet(
    full_server: &mut ServerGuard, 
    id: u32, 
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();
    let stat = codec.decode_varuint().ok_or(true)? as usize;

    if let Some(mut entity) = game_server.get_entity(id) 
        && entity.stats.alive 
        && entity.display.available_stat_points > 0 
    {
        let stat = entity.display.stat_investments.get_mut(stat).ok_or(true)?;
        if *stat < MAX_STAT_INVESTMENT {
            *stat += 1;
            entity.display.available_stat_points -= 1;
        }
    }

    Ok(())
}

pub fn handle_upgrade_packet(
    full_server: &mut ServerGuard, 
    id: u32, 
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();
    let upgrade_type = codec.decode_varuint().ok_or(true)? as usize;
    let upgrade_idx = codec.decode_varuint().ok_or(true)? as usize;

    if !(0..=1).contains(&upgrade_type) {
        return Err(true);
    }

    if let Some(mut entity) = game_server.get_entity(id) 
        && entity.stats.alive
    {
        if upgrade_type == 0 {
            let upgrade: BodyIdentity = (*entity.display.upgrades.body.get(upgrade_idx).ok_or(true)?)
                .try_into().unwrap();

            entity.display.body_identity = upgrade;
            entity.display.upgrades.body.clear();
        } else if upgrade_type == 1 {
            let upgrade: TurretStructure = (*entity.display.upgrades.turret.get(upgrade_idx).ok_or(true)?)
                .try_into().unwrap();

            entity.display.turret_identity = upgrade;
            entity.display.upgrades.turret.clear();
        }
    }

    Ok(())
}

pub fn form_update_packet(
    self_entity: &RefMut<'_, Entity>, 
    entities: &EntityDataStructure
) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ClientboundPackets::Update as u64);

    self_entity.take_census(&mut codec, true);

    codec.encode_varuint((entities.len() - 1) as u64);
    for (id, entity) in entities.iter() {
        if self_entity.id == *id { continue; }

        let entity = &entity.borrow_mut();
        entity.take_census(&mut codec, false);
    }

    codec
}