#[derive(Debug, Clone)]
pub enum ServerboundPackets {
    Spawn,
    Input
}

impl TryInto<ServerboundPackets> for u8 {
    type Error = bool;

    fn try_into(self) -> Result<ServerboundPackets, Self::Error> {
        match self {
            0x0 => Ok(ServerboundPackets::Spawn),
            0x1 => Ok(ServerboundPackets::Input),
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

#[derive(Clone, Copy)]
pub enum Inputs {
    Shoot,
    Up,
    Down,
    Left,
    Right
}

#[derive(Debug, strum_macros::EnumIter, Clone)]
pub enum CensusProperties {
    Position,
    Velocity,
    Angle,
    Health,
    MaxHealth,
    Name,
    Identity
}

impl TryInto<CensusProperties> for u8 {
    type Error = bool;

    fn try_into(self) -> Result<CensusProperties, Self::Error> {
        match self {
            0 => Ok(CensusProperties::Position),
            1 => Ok(CensusProperties::Velocity),
            2 => Ok(CensusProperties::Angle),
            3 => Ok(CensusProperties::Health),
            4 => Ok(CensusProperties::MaxHealth),
            5 => Ok(CensusProperties::Name),
            6 => Ok(CensusProperties::Identity),
            _ => Err(true)
        }
    }
}