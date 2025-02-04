use gloo::console::console;
use rand::{Rng, SeedableRng};
use shared::{connection::packets::CensusProperties, fuzzy_compare, game::{entity::EntityType, orb::OrbIdentityIds, theme::{ORB_BASIC_FILL, ORB_BASIC_STROKE, ORB_CELESTIAL_FILL, ORB_CELESTIAL_STROKE, ORB_FLICKERING_FILL, ORB_FLICKERING_STROKE, ORB_HEAVY_FILL, ORB_HEAVY_STROKE, ORB_RADIANT_FILL, ORB_RADIANT_STROKE, ORB_STABLE_FILL, ORB_STABLE_STROKE, STROKE_SIZE}}, utils::{codec::BinaryCodec, color::Color, vec2::Vector2D}};
use ui::canvas2d::Canvas2d;
use web_sys::js_sys;

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
        let (x, y, mut r) = (self.physics.position.value.x, self.physics.position.value.y, self.display.radius.value);

        context.save();

        context.rotate(std::f32::consts::FRAC_PI_2);

        match self.display.orb_identity.id {
            OrbIdentityIds::Basic => {
                
            },
            OrbIdentityIds::Flickering => {

            },
            OrbIdentityIds::Heavy => {
                let (time, frequency) = (self.time.ticks as f32, 0.03);
                let oscillation = ((time * frequency).sin() + 1.0) / 2.0;
                let (min_radius, max_radius) = (r, r + r / 1.4);
                let radius = min_radius + (max_radius - min_radius) * oscillation;
                
                let (min_opacity, max_opacity) = (0.1, 0.7);
                let opacity = max_opacity - ((radius - min_radius) / (max_radius - min_radius)) * (max_opacity - min_opacity);

                context.save();
                context.stroke_style(stroke);
                context.global_alpha(opacity);
                context.set_stroke_size(STROKE_SIZE);
                context.begin_arc(0.0, 0.0, radius, std::f32::consts::TAU);
                context.stroke();
                context.restore();
            },
            OrbIdentityIds::Radiant => {
                let seed: [u8; 32] = u64::to_le_bytes(self.id as u64)
                    .iter().cloned().chain(std::iter::repeat(0)).take(32)
                    .collect::<Vec<u8>>().try_into().unwrap();
                let mut rng = rand::rngs::StdRng::from_seed(seed);

                let num_particles = 60;
                let max_speed = 2.4;
                let lifetime = 90.0;
                let base_radius = r;

                for _ in 0..num_particles {
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                    let speed = rng.gen_range(0.5..max_speed);
                    let phase = rng.gen_range(0.0..lifetime);
                    
                    let age = ((self.time.ticks as f32 + phase) % lifetime) / lifetime;
                    
                    let distance = base_radius + age.powf(0.7) * base_radius * 4.0;
                    let opacity = (1.0 - age).powf(3.0);
                    let size = 5.0 * (1.0 - age) + 1.0;

                    let pos = Vector2D::from_polar(distance, angle);

                    context.save();
                    context.fill_style(fill);
                    context.stroke_style(stroke);
                    context.global_alpha(opacity);
                    context.begin_arc(pos.x, pos.y, size, std::f32::consts::TAU);
                    context.fill();
                    context.restore();
                }
            },
            OrbIdentityIds::Celestial => {
                let seed: [u8; 32] = u64::to_le_bytes(self.id as u64)
                    .iter().cloned().chain(std::iter::repeat(0)).take(32)
                    .collect::<Vec<u8>>().try_into().unwrap();
                let mut rng = rand::rngs::StdRng::from_seed(seed);
            
                let num_flares = 8;
                let base_radius = r;
                let period = 200.0;
                let max_length = base_radius * 2.0;
            
                for _ in 0..num_flares {
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                    let phase = rng.gen_range(0.0..period);
                    let speed_factor = rng.gen_range(0.8..1.2);
                    
                    let progress = ((self.time.ticks as f32 + phase) % (period * speed_factor)) / (period * speed_factor);
                    let flare_progress = if progress < 0.5 {
                        progress * 2.0
                    } else {
                        (1.0 - progress) * 2.0
                    };
            
                    let current_length = flare_progress * max_length;
                    let opacity = (flare_progress * 0.8).clamp(0.0, 1.0);
                    let width = 2.0 + flare_progress * 4.0;
            
                    let start = Vector2D::from_polar(base_radius, angle);
                    let end = Vector2D::from_polar(base_radius + current_length, angle);
            
                    context.save();
                    context.stroke_style(Color::blend_colors(fill, Color::WHITE, 0.3));
                    context.global_alpha(opacity);
                    context.set_stroke_size(width);
                    
                    context.begin_path();
                    context.move_to(start.x, start.y);
                    context.line_to(end.x, end.y);
                    context.stroke();
                    
                    context.restore();
                }
                
                context.save();
                context.global_alpha(0.3);
                context.fill_style(fill);
                context.begin_arc(0.0, 0.0, base_radius * 1.2, std::f32::consts::TAU);
                context.fill();
                context.restore();
            },
            _ => {}
        }

        let inner_fill = fill;
        let outer_fill = Color::blend_colors(fill, Color::WHITE, 0.4);
        let (outer_r, outer_g, outer_b) = outer_fill.to_rgb();

        let glow_stop_0 = format!("rgba({}, {}, {}, 0.6)", outer_r, outer_g, outer_b);
        let glow_stop_1 = format!("rgba({}, {}, {}, 0.0)", outer_r, outer_g, outer_b);

        let glow = context.create_radial_gradient(0.0, 0.0, 0.0, 0.0, 0.0, r + r / 1.5);
        glow.add_color_stop(0.0, &glow_stop_0);
        glow.add_color_stop(1.0, &glow_stop_1);
        context.fill_style_gradient(&glow);
        context.begin_arc(0.0, 0.0, r + r / 1.5, std::f32::consts::TAU);
        context.fill();

        let fill_gradient = context.create_radial_gradient(0.0, 0.0, 0.0, 0.0, 0.0, r);
        fill_gradient.add_color_stop(0.0, &inner_fill.css());
        fill_gradient.add_color_stop(1.0, &outer_fill.css());

        context.fill_style_gradient(&fill_gradient);
        context.stroke_style(stroke);
        context.set_stroke_size(STROKE_SIZE);

        context.begin_arc(0.0, 0.0, r, std::f32::consts::TAU);

        context.stroke();
        context.fill();

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