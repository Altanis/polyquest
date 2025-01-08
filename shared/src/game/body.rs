use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyRenderingHints {
    SmasherGuard {
        /// The thickness of the guard.
        thickness: f32,
        /// The number of sides on the guard.
        sides: usize
    }
}

#[derive(Default, Debug, Clone)]
pub struct BodyIdentity {
    /// The ID of the body identity.
    pub id: BodyIdentityIds, 
    /// Hints as to how to render the body.
    pub render_hints: Vec<BodyRenderingHints>,
    /// Message to notify with upon upgrade.
    pub upgrade_message: &'static str,
    /// The level requirement for the body.
    pub level_requirement: usize,
    /// The bodies the current body can upgrade to.
    pub upgrades: Vec<BodyIdentityIds>,
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
    pub body_damage: f32,
    /// The absorption factor of the tank.
    pub absorption_factor: f32
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum BodyIdentityIds {
    #[default]
    Base       = 0,
    Smasher    = 1
}

impl TryInto<BodyIdentityIds> for usize {
    type Error = ();

    fn try_into(self) -> Result<BodyIdentityIds, Self::Error> {
        match self {
            0 => Ok(BodyIdentityIds::Base),
            1 => Ok(BodyIdentityIds::Smasher),
            _ => Err(())
        }
    }
}

impl TryInto<BodyIdentity> for BodyIdentityIds {
    type Error = ();

    fn try_into(self) -> Result<BodyIdentity, Self::Error> {
        match self {
            BodyIdentityIds::Base => Ok(get_body_base_identity()),
            BodyIdentityIds::Smasher => Ok(get_body_smasher_identity())
        }
    }
}

impl Display for BodyIdentityIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Base => write!(f, "Base"),
            Self::Smasher => write!(f, "Smasher"),
        }
    }
}

pub fn get_body_base_identity() -> BodyIdentity {
    BodyIdentity {
        id: BodyIdentityIds::Base,
        render_hints: vec![],
        upgrade_message: "",
        level_requirement: 0,
        upgrades: vec![BodyIdentityIds::Smasher],
        invisibility_rate: -1.0,
        fov: 1.0,
        speed: 1.0,
        knockback: 1.0,
        max_health: 50.0,
        body_damage: 1.0,
        absorption_factor: 1.0
    }
}

pub fn get_body_smasher_identity() -> BodyIdentity {
    BodyIdentity {
        id: BodyIdentityIds::Smasher,
        render_hints: vec![BodyRenderingHints::SmasherGuard {
            thickness: 1.15,
            sides: 6
        }],
        upgrade_message: "",
        level_requirement: 0,
        upgrades: vec![],
        invisibility_rate: -1.0,
        fov: 1.0,
        speed: 1.0,
        knockback: 1.0,
        max_health: 50.0,
        body_damage: 1.0,
        absorption_factor: 1.0
    }
}
