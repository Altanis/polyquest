use std::fmt::Display;

macro_rules! into_structure {
    (
        $( #[$enum_meta:meta] )*
        $enum_vis:vis enum $enum_name:ident {
            $(
                $( #[$variant_meta:meta] )*
                $variant:ident = $variant_index:expr,
            )*
        }
    ) => {
        $( #[$enum_meta] )*
        $enum_vis enum $enum_name {
            $(
                $( #[$variant_meta] )*
                $variant = $variant_index,
            )*
        }

        impl std::convert::TryFrom<BodyIdentityIds> for BodyIdentity {
            type Error = ();

            fn try_from(value: BodyIdentityIds) -> ::std::result::Result<BodyIdentity, Self::Error> {
                match value {
                    $(
                        BodyIdentityIds::$variant => Ok(::paste::paste! { [< get_body_ $variant:snake _identity >] () }),
                    )*
                }
            }
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BodyRenderingHints {
    SmasherGuard {
        /// The thickness of the guard.
        thickness: f32,
        /// The number of sides on the guard.
        sides: usize
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct BodyIdentity {
    /// The ID of the body identity.
    pub id: BodyIdentityIds, 
    /// Hints as to how to render the body.
    pub render_hints: Vec<BodyRenderingHints>,
    /// The level requirement for the body.
    pub level_requirement: usize,
    /// The bodies the current body can upgrade to.
    pub upgrades: Vec<BodyIdentityIds>,
    /// The inherent speed of the tank.
    pub speed: f32,
    /// The (base) maximum health of the body.
    pub max_health: f32,
    /// The multiplier for body damage.
    pub body_damage: f32,
    /// The absorption factor of the tank.
    pub absorption_factor: f32,
    /// A description of the body.
    pub description: &'static str
}

into_structure! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
    #[repr(usize)]
    pub enum BodyIdentityIds {
        #[default]
        Base       = 0,
        Smasher    = 1,
    }   
}

impl Display for BodyIdentityIds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let variant_name = format!("{:?}", self);
        let formatted_name: String = variant_name
            .chars()
            .enumerate()
            .flat_map(|(i, c)| {
                if i > 0 && c.is_uppercase() {
                    vec![' ', c]
                } else {
                    vec![c]
                }
            })
            .collect();

        write!(f, "{}", formatted_name)
    }
}

pub fn get_body_base_identity() -> BodyIdentity {
    BodyIdentity {
        id: BodyIdentityIds::Base,
        render_hints: vec![],
        level_requirement: 0,
        upgrades: vec![BodyIdentityIds::Smasher],
        speed: 1.0,
        max_health: 50.0,
        body_damage: 1.0,
        absorption_factor: 1.0,
        description: "Null and void."
    }
}

pub fn get_body_smasher_identity() -> BodyIdentity {
    BodyIdentity {
        id: BodyIdentityIds::Smasher,
        render_hints: vec![BodyRenderingHints::SmasherGuard {
            thickness: 1.15,
            sides: 6
        }],
        level_requirement: 0,
        upgrades: vec![],
        speed: 1.0,
        max_health: 55.0,
        body_damage: 1.2,
        absorption_factor: 0.95,
        description: "Takes less knockback and slightly increased health/body damage."
    }
}
