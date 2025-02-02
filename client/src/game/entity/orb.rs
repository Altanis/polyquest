use gloo::console::console;
use shared::{connection::packets::CensusProperties, fuzzy_compare, game::{entity::EntityType, orb::OrbIdentityIds, theme::{ORB_BASIC_FILL, ORB_BASIC_STROKE, ORB_CELESTIAL_FILL, ORB_CELESTIAL_STROKE, ORB_FLICKERING_FILL, ORB_FLICKERING_STROKE, ORB_HEAVY_FILL, ORB_HEAVY_STROKE, ORB_RADIANT_FILL, ORB_RADIANT_STROKE, ORB_STABLE_FILL, ORB_STABLE_STROKE, STROKE_SIZE}}, utils::{codec::BinaryCodec, color::Color, vec2::Vector2D}};
use ui::canvas2d::Canvas2d;

use super::base::{Entity, HealthState};

impl Entity {
    pub fn parse_orb_census(&mut self, codec: &mut BinaryCodec) {
        self.display.z_index = 0;

        let properties = codec.decode_varuint().unwrap();
        for _ in 0..properties {
            let property: CensusProperties = (codec.decode_varuint().unwrap() as u8).try_into().unwrap();

            match property {
                CensusProperties::Position => {
                    self.physics.position.target = Vector2D::new(
                        codec.decode_f32().unwrap(),
                        codec.decode_f32().unwrap()
                    );
                },
                CensusProperties::Velocity => {
                    self.physics.velocity.target = Vector2D::new(
                        codec.decode_f32().unwrap(),
                        codec.decode_f32().unwrap()
                    );
                },
                CensusProperties::Angle => self.physics.angle.target = codec.decode_f32().unwrap(),
                CensusProperties::Health => {
                    let health = codec.decode_f32().unwrap();
                    console!(health.to_string());
                    if health < self.stats.health.target {
                        self.display.damage_blend.target = 0.9;
                    }

                    self.stats.health.target = health;
                    self.stats.health_state = if self.stats.health.target >= 0.0 {
                        HealthState::Alive
                    } else {
                        HealthState::Dying
                    };
                },
                CensusProperties::MaxHealth => self.stats.max_health.target = codec.decode_f32().unwrap(),
                CensusProperties::Opacity => self.display.opacity.target = codec.decode_f32().unwrap(),
                CensusProperties::Radius => self.display.radius.target = codec.decode_f32().unwrap(),
                CensusProperties::Ticks => self.time.server_ticks = codec.decode_varuint().unwrap(),
                CensusProperties::Identity => {
                    let orb_identity_id: OrbIdentityIds = (codec.decode_varuint().unwrap() as usize).try_into().unwrap();
                    self.display.orb_identity = orb_identity_id.try_into().unwrap();
                },
                _ => {}
            }
        }
    }

    fn compute_orb_fill(&self) -> (Color, Color) {
        match self.display.orb_identity.id {
            OrbIdentityIds::Flickering => (ORB_FLICKERING_FILL, ORB_FLICKERING_STROKE),
            OrbIdentityIds::Basic => (ORB_BASIC_FILL, ORB_BASIC_STROKE),
            OrbIdentityIds::Stable => (ORB_STABLE_FILL, ORB_STABLE_STROKE),
            OrbIdentityIds::Heavy => (ORB_HEAVY_FILL, ORB_HEAVY_STROKE),
            OrbIdentityIds::Radiant => (ORB_RADIANT_FILL, ORB_RADIANT_STROKE),
            OrbIdentityIds::Celestial => (ORB_CELESTIAL_FILL, ORB_CELESTIAL_STROKE)
        }
    }

    fn render_orb_body(&self, context: &mut Canvas2d) {
        let (fill, stroke) = self.compute_orb_fill();

        context.save();

        context.fill_style(fill);
        context.stroke_style(stroke);
        context.set_stroke_size(STROKE_SIZE);

        context.rotate(std::f32::consts::FRAC_PI_2);

        context.begin_arc(0.0, 0.0, self.display.orb_identity.radius, std::f32::consts::TAU);

        // match self.display.orb_identity.id {
        //     OrbIdentityIds::Flickering => {
        //     },
        //     OrbIdentityIds::Basic => {

        //     },
        //     OrbIdentityIds::Stable => {

        //     },
        //     OrbIdentityIds::Heavy => {

        //     },
        //     OrbIdentityIds::Radiant => {

        //     },
        //     OrbIdentityIds::Celestial => {

        //     }
        // }

        context.fill();
        context.stroke();

        context.restore();
    }

    pub fn render_orb(&mut self, context: &mut Canvas2d, dt: f32) {
        self.time.ticks += 1;
        if matches!(self.stats.health_state, HealthState::Dying | HealthState::Dead) {
            self.destroy_orb(context, dt);
        }

        context.save();
        
        context.translate(
            self.physics.position.value.x + self.physics.velocity.value.x, 
            self.physics.position.value.y + self.physics.velocity.value.y
        );
        context.rotate(self.physics.angle.value);
        context.global_alpha(self.display.opacity.value);
        context.set_stroke_size(STROKE_SIZE);

        self.render_orb_body(context);

        context.restore();
    }

    fn destroy_orb(&mut self, context: &mut Canvas2d, dt: f32) {
        if fuzzy_compare!(self.display.opacity.value, 0.0, 1e-1) {
            self.stats.health_state = HealthState::Dead;
            return;
        }

        self.display.opacity.target = 0.0;
        self.display.radius.target *= 1.05;
    }
}