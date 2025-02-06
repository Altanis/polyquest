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
            OrbIdentityIds::Radiant => (ORB_RADIANT_FILL, ORB_RADIANT_STROKE)
        }
    }

    fn render_orb_body(&self, context: &mut Canvas2d) {
        let (fill, stroke) = self.compute_orb_fill();
        let (x, y, mut r) = (self.physics.position.value.x, self.physics.position.value.y, self.display.radius.value);

        context.save();
        context.rotate(std::f32::consts::FRAC_PI_2);

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

        match self.display.orb_identity.id {
            OrbIdentityIds::Basic => {
                let pulse = ((self.time.ticks as f32 * 0.04).sin() + 1.0) / 2.0;
                context.save();
                context.fill_style(Color::blend_colors(fill, Color::WHITE, 0.4));
                context.global_alpha(0.3 * pulse);
                context.begin_arc(0.0, 0.0, r * 0.35, std::f32::consts::TAU);
                context.fill();
                context.restore();
            },
            OrbIdentityIds::Stable => {
                context.save();
                let angle = self.time.ticks as f32 * 0.015;
                context.rotate(angle);
                
                let (halo_r, halo_g, halo_b) = Color::blend_colors(fill, Color::WHITE, 0.2).to_rgb();
                let halo_color = format!("rgba({}, {}, {}, 0.3)", halo_r, halo_g, halo_b);

                let gradient = context.create_linear_gradient(-r * 1.5, 0.0, r * 1.5, 0.0);
                gradient.add_color_stop(0.0, "transparent");
                gradient.add_color_stop(0.4, &halo_color);
                gradient.add_color_stop(0.6, &halo_color);
                gradient.add_color_stop(1.0, "transparent");
                
                context.stroke_style_gradient(&gradient);
                context.set_stroke_size(10.0);
                context.begin_arc(0.0, 0.0, r * 1.1, std::f32::consts::TAU);
                context.stroke();
                context.restore();
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
                context.save();
            
                // Core parameters
                let time = self.time.ticks as f32;
                let base_glow = Color::blend_colors(fill, Color::WHITE, 0.4);
                let (r_base, g_base, b_base) = base_glow.to_rgb();
            
                // 1. Intense pulsating core
                let core_pulse = (time * 0.07).sin().abs();
                context.fill_style(base_glow);
                context.global_alpha(0.3 + core_pulse * 0.2);
                context.begin_arc(0.0, 0.0, r * 0.6, std::f32::consts::TAU);
                context.fill();
            
                // 2. Dynamic energy corona
                let corona_gradient = context.create_radial_gradient(0.0, 0.0, r * 0.8, 0.0, 0.0, r * 1.8);
                corona_gradient.add_color_stop(0.0, &format!("rgba({}, {}, {}, 0.8)", r_base, g_base, b_base));
                corona_gradient.add_color_stop(1.0, &format!("rgba({}, {}, {}, 0.0)", r_base, g_base, b_base));
                
                context.save();
                context.rotate(time * 0.02);
                context.global_alpha(0.6);
                context.fill_style_gradient(&corona_gradient);
                
                // Create 8 energy spikes using clipping
                // for i in 0..8 {
                //     context.save();
                //     context.rotate(i as f32 * std::f32::consts::FRAC_PI_4);
                //     context.begin_path();
                //     context.move_to(0.0, 0.0);
                //     context.line_to(r * 1.4, 0.0);
                //     context.arc(0.0, 0.0, r * 1.4, std::f32::consts::FRAC_PI_8);
                //     context.close_path();
                //     context.fill();
                //     context.restore();
                // }
                context.restore();
            
                // 3. Rotating prismatic halo
                let halo_rot = time * 0.015;
                context.save();
                context.rotate(halo_rot);
                
                let r_halo = r * 1.4;
                let halo = context.create_linear_gradient(-r_halo, 0.0, r_halo, 0.0);
                halo.add_color_stop(0.0, "rgba(255, 255, 255, 0.4)");
                halo.add_color_stop(0.3, &format!("rgba({}, {}, {}, 0.6)", r_base, g_base, b_base));
                halo.add_color_stop(0.7, &format!("rgba({}, {}, {}, 0.6)", r_base, g_base, b_base));
                halo.add_color_stop(1.0, "rgba(255, 255, 255, 0.4)");
                
                context.stroke_style_gradient(&halo);
                context.set_stroke_size(8.0);
                context.begin_arc(0.0, 0.0, r_halo, std::f32::consts::TAU);
                context.stroke();
                context.restore();
            
                // 4. Floating spark particles (limited count)
                let num_sparkles = 8;
                for i in 0..num_sparkles {
                    let angle = time * 0.01 + i as f32 * std::f32::consts::TAU / num_sparkles as f32;
                    let spark_dist = r * 1.8 + (time * 0.05 + i as f32).sin() * r * 0.2;
                    let spark_size = 3.0 + (time * 0.1 + i as f32).sin().abs() * 2.0;
                    let opacity = 0.5 + (time * 0.08 + i as f32).sin().abs() * 0.3;
                    
                    context.save();
                    context.rotate(angle);
                    context.translate(spark_dist, 0.0);
                    context.fill_style(Color::WHITE);
                    context.global_alpha(opacity);
                    context.begin_arc(0.0, 0.0, spark_size, std::f32::consts::TAU);
                    context.fill();
                    context.restore();
                }
            
                context.restore();
            },
            _ => {}
        }

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