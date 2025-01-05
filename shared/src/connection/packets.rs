#[derive(Debug, Clone)]
pub enum ServerboundPackets {
    Spawn,
    Input,
    Stats,
    Upgrade
}

impl TryInto<ServerboundPackets> for u8 {
    type Error = bool;

    fn try_into(self) -> Result<ServerboundPackets, Self::Error> {
        match self {
            0x0 => Ok(ServerboundPackets::Spawn),
            0x1 => Ok(ServerboundPackets::Input),
            0x2 => Ok(ServerboundPackets::Stats),
            0x3 => Ok(ServerboundPackets::Upgrade),
            _ => Err(true)
        }
    }
}

#[derive(Clone)]
pub enum ClientboundPackets {
    Update
}

impl TryInto<ClientboundPackets> for u8 {
    type Error = bool;

    fn try_into(self) -> Result<ClientboundPackets, Self::Error> {
        match self {
            0x0 => Ok(ClientboundPackets::Update),
            _ => Err(true)
        }
    }
}

#[derive(Debug, Clone, Copy, strum_macros::EnumIter)]
pub enum Inputs {
    Shoot    = 0b10,
    Up       = 0b100,
    Down     = 0b1000,
    Left     = 0b10000,
    Right    = 0b100000,
    LevelUp  = 0b1000000
}

#[derive(Debug, strum_macros::EnumIter, Clone)]
pub enum CensusProperties {
    Position,
    Velocity,
    Angle,
    Name,
    Score,
    Health,
    MaxHealth,
    Energy,
    MaxEnergy,
    Stats,
    Upgrades,
    Opacity,
    Fov,
    Radius,
    Identity,
    Owners
}

impl TryInto<CensusProperties> for u8 {
    type Error = bool;

    fn try_into(self) -> Result<CensusProperties, Self::Error> {
        match self {
            0 => Ok(CensusProperties::Position),
            1 => Ok(CensusProperties::Velocity),
            2 => Ok(CensusProperties::Angle),
            3 => Ok(CensusProperties::Name),
            4 => Ok(CensusProperties::Score),
            5 => Ok(CensusProperties::Health),
            6 => Ok(CensusProperties::MaxHealth),
            7 => Ok(CensusProperties::Energy),
            8 => Ok(CensusProperties::MaxEnergy),
            9 => Ok(CensusProperties::Stats),
            10 => Ok(CensusProperties::Upgrades),
            11 => Ok(CensusProperties::Opacity),
            12 => Ok(CensusProperties::Fov),
            13 => Ok(CensusProperties::Radius),
            14 => Ok(CensusProperties::Identity),
            15 => Ok(CensusProperties::Owners),
            _ => Err(true),
        }
    }
}