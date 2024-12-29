#[derive(Default, Debug, Clone)]
pub struct BodyIdentity {
    /// The ID of the body identity.
    pub id: usize, 
    /// Message to notify with upon upgrade.
    pub upgrade_message: &'static str,
    /// The level requirement for the body.
    pub level_requirement: usize,
    /// The bodies the current body can upgrade to.
    pub upgrades: Vec<BodyIdentity>,
    /// The rate at which the opacity decreases per tick.
    pub invisibility_rate: f32,
    /// The FoV factor of the body.
    pub fov: f32,
    /// The inherent speed of the tank.
    pub speed: f32,
    /// The knockback the body receives upon collision.
    pub knockback: f32,
    /// The (base) maximum health of the body.
    pub max_health: f32,
    /// The multiplier for body damage.
    pub body_damage: f32
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum BodyIdentityIds {
    #[default]
    Base       = 0,
}

impl TryInto<BodyIdentityIds> for usize {
    type Error = ();

    fn try_into(self) -> Result<BodyIdentityIds, Self::Error> {
        match self {
            0 => Ok(BodyIdentityIds::Base),
            _ => Err(())
        }
    }
}

impl TryInto<BodyIdentity> for BodyIdentityIds {
    type Error = ();

    fn try_into(self) -> Result<BodyIdentity, Self::Error> {
        match self {
            BodyIdentityIds::Base => Ok(get_body_base_identity()),
        }
    }
}

pub fn get_body_base_identity() -> BodyIdentity {
    BodyIdentity {
        id: 0,
        upgrade_message: "",
        level_requirement: 0,
        upgrades: vec![],
        invisibility_rate: -1.0,
        fov: 1.0,
        speed: 1.0,
        knockback: 1.0,
        max_health: 50.0,
        body_damage: 1.0
    }
}