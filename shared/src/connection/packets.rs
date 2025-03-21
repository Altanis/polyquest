#[derive(Debug, Clone, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum ServerboundPackets {
    Spawn    = 0x0,
    Input    = 0x1,
    Stats    = 0x2,
    Upgrade  = 0x3,
    Ping     = 0x4,
    Chat     = 0x5,
    Clan     = 0x6
}

#[derive(Debug, Clone, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum ClientboundPackets {
    Update         = 0x0,
    Notifications  = 0x1,
    Pong           = 0x2,
    ServerInfo     = 0x3
}

#[derive(Debug, Clone, PartialEq, num_enum::TryFromPrimitive)]
#[repr(u64)]
pub enum ClanPacketOpcode {
    Create    = 0x0,
    Join      = 0x1,
    Leave     = 0x2,
    Kick      = 0x3,
    Distress  = 0x4
}

#[derive(Debug, Clone, Copy, strum_macros::EnumIter)]
pub enum Inputs {
    Shoot    = 0b10,
    Up       = 0b100,
    Down     = 0b1000,
    Left     = 0b10000,
    Right    = 0b100000,
    LevelUp  = 0b1000000,
    Repel    = 0b10000000,
    Switch   = 0b100000000
}

#[derive(Debug, strum_macros::EnumIter, Clone, num_enum::TryFromPrimitive)]
#[repr(u8)]
pub enum CensusProperties {
    Position,
    Velocity,
    Angle,
    Name,
    Score,
    Health,
    MaxHealth,
    Stats,
    Upgrades,
    Opacity,
    Fov,
    Radius,
    Identity,
    Owners,
    Ticks,
    Invincibility,
    Messages,
    Clan
}