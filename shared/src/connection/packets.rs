#[derive(Clone)]
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

#[derive(Clone, Copy)]
pub enum Inputs {
    Shoot,
    Up,
    Down,
    Left,
    Right
}

#[derive(strum_macros::EnumIter, Clone)]
pub enum CensusProperties {
    Position,
    Velocity,
    Angle,
    Health,
    MaxHealth,
}