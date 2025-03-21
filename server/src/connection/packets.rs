use shared::{connection::packets::{ClanPacketOpcode, ClientboundPackets}, game::{body::{BodyIdentity, BodyIdentityIds}, entity::{get_min_score_from_level, ClanInformation, InputFlags, Notification, MAX_STAT_INVESTMENT}, turret::{TurretIdentityIds, TurretStructure}}, utils::{codec::BinaryCodec, color::Color, consts::{CLAN_DESC_LENGTH, CLAN_MAX_MEMBERS, CLAN_NAME_LENGTH, SCREEN_HEIGHT, SCREEN_WIDTH}, vec2::Vector2D}};
use crate::{game::{clans::ClanState, entity::base::{AliveState, Entity}, state::{EntityDataStructure, GameState}}, server::{ServerGuard, LEADER_ARROW_VIEW}};

pub fn handle_spawn_packet(
    full_server: &mut ServerGuard, 
    id: u32, 
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();
    let mut name = codec.decode_string().ok_or(true)?;
    name.truncate(16);

    let random_position = game_server.get_random_position();
    if let Some(mut entity) = game_server.get_entity(id) && entity.stats.alive != AliveState::Alive {
        let old_level = entity.display.level;
        *entity = Entity::from_id(entity.id);

        entity.physics.position = random_position;
        entity.display.name = name;
        entity.stats.alive = AliveState::Alive;
    
        entity.stats.health = entity.display.body_identity.max_health;
        entity.stats.max_health = entity.display.body_identity.max_health;

        entity.display.score = get_min_score_from_level((old_level / 2).max(1));
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

    if let Some(mut entity) = game_server.get_entity(id) && entity.stats.alive == AliveState::Alive {
        entity.physics.inputs = InputFlags::new(flags);

        let (screen_width, screen_height) = ((SCREEN_WIDTH / entity.display.fov) / 0.9, (SCREEN_HEIGHT / entity.display.fov) / 0.9);
        let screen_top_left = entity.physics.position - Vector2D::new(screen_width / 2.0, screen_height / 2.0);
        let screen_bottom_right = entity.physics.position + Vector2D::new(screen_width / 2.0, screen_height / 2.0);

        let mouse_in_bounds = mouse.x >= screen_top_left.x 
            && mouse.x <= screen_bottom_right.x
            && mouse.y >= screen_top_left.y
            && mouse.y <= screen_bottom_right.y;

        if mouse_in_bounds {
            entity.physics.mouse = mouse;
            entity.physics.angle = (mouse - entity.physics.position).angle();
        }
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
        && entity.stats.alive == AliveState::Alive 
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

    let deletions = if let Some(mut entity) = game_server.get_entity(id) 
        && entity.stats.alive == AliveState::Alive
        && upgrade_type == 1 {
            std::mem::take(&mut entity.display.owned_entities)
        } else {
            vec![]
        };

    for deletion in deletions {
        game_server.delete_entity(deletion);
    }

    if let Some(mut entity) = game_server.get_entity(id) 
        && entity.stats.alive == AliveState::Alive
    {
        if upgrade_type == 0 {
            if let Some(upgrade) = entity.display.upgrades.body.get(upgrade_idx) {
                let upgrade: BodyIdentity = (*upgrade).try_into().unwrap();
                entity.display.body_identity = upgrade;
                entity.physics.absorption_factor = entity.display.body_identity.absorption_factor;
                entity.display.upgrades.body.clear();
            }
        } else if upgrade_type == 1 {
            if let Some(upgrade) = entity.display.upgrades.turret.get(upgrade_idx).cloned() {
                let upgrade: TurretStructure = upgrade.try_into().unwrap();
                entity.display.turret_identity = upgrade;
                entity.display.upgrades.turret.clear();
            }
        }
    }
    
    Ok(())
}

pub fn handle_chat_packet(
    full_server: &mut ServerGuard,
    id: u32,
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();
    let Some(mut entity) = game_server.get_entity(id) else { return Ok(()); };

    let packet_type = codec.decode_varuint().ok_or(true)?;
    
    match packet_type {
        0x0 => entity.display.typing = codec.decode_bool().ok_or(true)?,
        0x1 => {
            let message = codec.decode_string().ok_or(true)?.chars().take(72).collect::<String>();

            if entity.display.messages.len() >= 3 {
                entity.display.notifications.push(Notification {
                    message: "You can send another message in a few seconds.".to_string(),
                    color: Color::RED,
                    lifetime: 150,
                    ..Default::default()
                })
            } else {
                let ticks = entity.time.ticks;
                entity.display.messages.push((message, ticks));
            }

            entity.display.typing = false;
        },
        _ => return Err(true)
    }

    Ok(())
}

pub fn handle_clan_packet(
    full_server: &mut ServerGuard,
    id: u32,
    mut codec: BinaryCodec
) -> Result<(), bool> {
    let game_server = full_server.game_server.get_server();
    let entity_id = {
        let Some(entity) = game_server.get_entity(id) else { return Ok(()); };
        entity.id
    };

    let packet_type: ClanPacketOpcode = codec.decode_varuint().ok_or(true)?.try_into().map_err(|_| true)?;
    
    match packet_type {
        ClanPacketOpcode::Create => {
            let (name, description, max_members) = {
                let mut name = codec.decode_string().ok_or(true)?;
                let mut description = codec.decode_string().ok_or(true)?;
                let max_members = (codec.decode_varuint().ok_or(true)? as usize).min(CLAN_MAX_MEMBERS);

                name.truncate(CLAN_NAME_LENGTH as usize);
                description.truncate(CLAN_DESC_LENGTH as usize);

                (name, description, max_members)
            };

            let id = game_server.clan_state.create_clan(name, description, max_members, entity_id);
            game_server.get_entity(entity_id).unwrap().display.clan_id = Some(id);
       },
       ClanPacketOpcode::Join => {
            let id = codec.decode_varuint().ok_or(true)? as usize;
            if game_server.get_entity(entity_id).unwrap().display.pending_clan_id.is_some() {
                return Ok(());
            }

            let clan = game_server.clan_state.clans.get_mut(id).ok_or(false)?;

            if clan.members.len() < clan.max_members {
                clan.pending_members.push(entity_id);
                game_server.get_entity(entity_id).unwrap().display.pending_clan_id = Some(id as u32);
            }
       },
        _ => return Ok(())
    }

    Ok(())
}

pub fn form_update_packet(
    self_entity: &mut Entity, 
    entities: &EntityDataStructure,
    clans: &ClanState
) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ClientboundPackets::Update as u64);

    // CLANS //
    codec.encode_varuint(clans.clans.len() as u64);
    for clan in clans.clans.iter() {
        codec.encode_varuint(clan.id as u64);
        codec.encode_varuint(clan.owner as u64);
        codec.encode_string(clan.name.clone());
        codec.encode_string(clan.description.clone());
        codec.encode_varuint(clan.max_members as u64);

        codec.encode_varuint(clan.members.len() as u64);
        codec.encode_varuint(clan.pending_members.len() as u64);

        for &member in clan.members.iter() {
            codec.encode_varuint(member as u64);
        }

        for &member in clan.pending_members.iter() {
            codec.encode_varuint(member as u64);
        }
    }

    // ENTITIES //
    self_entity.take_census(&mut codec, true);

    let ids: Vec<u32> = self_entity.display.surroundings.clone().into_iter().filter(|&id| {
        if id == self_entity.id { return false; }

        if let Some(entity) = entities.get(&id) {
            entity.borrow_mut().stats.alive != AliveState::Uninitialized
        } else {
            false
        }
    }).collect();

    codec.encode_varuint(ids.len() as u64);
    for id in ids.iter() {
        let entity = &entities.get(id).unwrap().borrow_mut();
        entity.take_census(&mut codec, false);
    }

    codec
}

pub fn form_notification_packet(
    self_entity: &mut Entity
) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ClientboundPackets::Notifications as u64);

    codec.encode_varuint(self_entity.display.notifications.len() as u64);
    while let Some(notification) = self_entity.display.notifications.pop() {
        codec.encode_string(notification.message);
        codec.encode_varuint(notification.color.0 as u64);
        codec.encode_varuint(notification.color.1 as u64);
        codec.encode_varuint(notification.color.2 as u64);
        codec.encode_varuint(notification.lifetime);
    }

    codec
}

pub fn form_pong_packet() -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ClientboundPackets::Pong as u64);

    codec
}

pub fn form_server_info_packet(
    state: &GameState, 
    leaderboard: &[(usize, String, BodyIdentityIds, TurretIdentityIds, Vector2D)], 
    reference_position: Vector2D, reference_fov: f32
) -> BinaryCodec {
    let mut codec = BinaryCodec::new();
    codec.encode_varuint(ClientboundPackets::ServerInfo as u64);
    
    codec.encode_f32(state.mspt);

    codec.encode_varuint(leaderboard.len() as u64);
    for (score, name, body_identity, turret_identity, _) in leaderboard.iter() {
        codec.encode_varuint(*score as u64);
        codec.encode_string(name.clone());
        codec.encode_varuint(*body_identity as u64);
        codec.encode_varuint(*turret_identity as u64);
    }

    codec.encode_f32(if let Some((_, _, _, _, position)) = leaderboard.first() 
        && position.distance(reference_position) > LEADER_ARROW_VIEW * reference_fov
    {
        (reference_position - *position).angle()
    } else {
        -13.0
    });

    codec
}