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

        impl std::convert::TryFrom<OrbIdentityIds> for OrbIdentity {
            type Error = ();

            fn try_from(value: OrbIdentityIds) -> ::std::result::Result<OrbIdentity, Self::Error> {
                match value {
                    $(
                        OrbIdentityIds::$variant => Ok(::paste::paste! { [< get_orb_ $variant:snake _identity >] () }),
                    )*
                }
            }
        }
    };
}

#[derive(Default, Debug, Clone)]
pub struct OrbIdentity {
    /// The ID of the orb identity.
    pub id: OrbIdentityIds, 
    pub linear_speed: f32,
    pub angular_speed: f32,
    /// The (base) maximum health of the orb.
    pub max_health: f32,
    /// The multiplier for body damage.
    pub body_damage: f32,
    pub absorption_factor: f32,
    pub push_factor: f32,
    /// The radius of the orb.
    pub radius: f32,
    /// The EXP yield of the orb.
    pub exp_yield: usize
}

into_structure! {
    #[derive(Debug, Default, Clone, Copy, PartialEq, num_enum::TryFromPrimitive)]
    #[repr(usize)]
    pub enum OrbIdentityIds {
        #[default]
        Flickering  = 0,
        Basic       = 1,
        Stable      = 2,
        Heavy       = 3,
        Radiant     = 4,
    }   
}

impl Display for OrbIdentityIds {
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

pub fn get_orb_flickering_identity() -> OrbIdentity {
    OrbIdentity {
        id: OrbIdentityIds::Flickering,
        linear_speed: 7.5,
        angular_speed: 1.0,
        max_health: 0.5,
        body_damage: 0.0,
        absorption_factor: 0.0,
        push_factor: 0.01,
        radius: 15.0,
        exp_yield: 5
    }
}

pub fn get_orb_basic_identity() -> OrbIdentity {
    OrbIdentity {
        id: OrbIdentityIds::Basic,
        linear_speed: 3.0,
        angular_speed: 1.0,
        max_health: 10.0,
        body_damage: 8.0,
        absorption_factor: 1.0,
        push_factor: 8.0,
        radius: 35.0,
        exp_yield: 20
    }
}

pub fn get_orb_stable_identity() -> OrbIdentity {
    OrbIdentity {
        id: OrbIdentityIds::Stable,
        linear_speed: 3.0,
        angular_speed: 1.0,
        max_health: 30.0,
        body_damage: 8.0,
        absorption_factor: 1.0,
        push_factor: 8.0,
        radius: 55.0,
        exp_yield: 50
    }
}

pub fn get_orb_heavy_identity() -> OrbIdentity {
    OrbIdentity {
        id: OrbIdentityIds::Heavy,
        linear_speed: 2.0,
        angular_speed: 1.0,
        max_health: 100.0,
        body_damage: 12.0,
        absorption_factor: 0.5,
        push_factor: 11.0,
        radius: 75.0,
        exp_yield: 260
    }
}

pub fn get_orb_radiant_identity() -> OrbIdentity {
    OrbIdentity {
        id: OrbIdentityIds::Radiant,
        linear_speed: 0.5,
        angular_speed: 1.0,
        max_health: 400.0,
        body_damage: 20.0,
        absorption_factor: 0.25,
        push_factor: 11.0,
        radius: 145.0,
        exp_yield: 2000
    }
}