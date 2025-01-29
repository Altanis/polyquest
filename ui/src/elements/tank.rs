use shared::{fuzzy_compare, game::{body::{get_body_base_identity, BodyIdentity, BodyRenderingHints}, entity::FICTITIOUS_TANK_RADIUS, theme::{PLAYER_FILL, PLAYER_STROKE, SMASHER_GUARD_FILL, SMASHER_GUARD_STROKE, STROKE_SIZE, TURRET_FILL, TURRET_STROKE}, turret::{get_turret_base_identity, TurretRenderingHints, TurretStructure}}, lerp, utils::{color::Color, interpolatable::Interpolatable, vec2::Vector2D}};
use web_sys::MouseEvent;

use crate::{canvas2d::{Canvas2d, Transform}, core::{BoundingRect, DeletionEffects, ElementType, Events, HoverEffects, UiElement}, DEBUG};

/// This element is used to render tanks with specific bodies and weaponry.
pub struct Tank {
    id: String,
    transform: Transform,
    raw_transform: Transform,
    angle: f32,
    radius: f32,
    z_index: i32,
    children: Vec<Box<dyn UiElement>>,
    events: Events,
    is_animating: bool,
    opacity: Interpolatable<f32>,
    destroyed: bool,
    body_identity: BodyIdentity,
    turret_structure: TurretStructure,
    stroke: f32,
    ticks: u64
}

impl Default for Tank {
    fn default() -> Self {
        Self {
            id: String::default(),
            transform: Default::default(),
            raw_transform: Default::default(),
            angle: 0.0,
            radius: Default::default(),
            z_index: Default::default(),
            children: Default::default(),
            events: Default::default(),
            is_animating: Default::default(),
            opacity: Interpolatable::new(1.0),
            destroyed: Default::default(),
            body_identity: get_body_base_identity(),
            turret_structure: get_turret_base_identity(),
            stroke: STROKE_SIZE,
            ticks: Default::default(),
        }
    }
}

impl UiElement for Tank {
    fn get_identity(&self) -> ElementType {
        ElementType::Tank
    }

    fn get_id(&self) -> String {
        self.id.clone()
    }

    fn get_events(&self) -> &Events {
        &self.events
    }
    
    fn get_mut_events(&mut self) -> &mut Events {
        &mut self.events
    }

    fn set_transform(&mut self, transform: Transform) {
        self.transform = transform;
    }

    fn get_transform(&self) -> &Transform {
        &self.transform
    }

    fn set_opacity(&mut self, opacity: f32) {
        self.opacity.target = opacity;
    }

    fn get_z_index(&self) -> i32 {
        self.z_index
    }

    fn set_hovering(&mut self, val: bool, event: &MouseEvent) -> bool {
        self.events.is_hovering = val;
        for child in self.children.iter_mut() {
            child.set_hovering(val, event);
        }

        val
    }

    fn set_clicked(&mut self, val: bool, _: &MouseEvent) {
        self.events.is_clicked = val;
    }

    // Checkboxes are not meant to have children.
    fn get_mut_children(&mut self) -> &mut Vec<Box<dyn UiElement>> {
        &mut self.children
    }

    fn get_element_by_id(&mut self, id: &str) -> Option<(usize, &mut Box<dyn UiElement>)> {
        self.children
            .iter_mut()
            .enumerate()
            .find(|(_, child)| child.get_id() == id)
    }

    fn delete_element_by_id(&mut self, id: &str, destroy: bool) {
        if let Some((i, child)) = self.children
            .iter_mut()
            .enumerate()
            .find(|(_, child)| child.get_id() == id) 
        {
            if destroy {
                child.destroy();
            } else {
                self.children.remove(i);
            }
        }
    }

    fn set_children(&mut self, _: Vec<Box<dyn UiElement>>) {}

    fn get_bounding_rect(&self) -> BoundingRect {
        let mut position = Vector2D::from_scalar(-self.radius);
        let mut dimensions = Vector2D::from_scalar(self.radius * 2.0);

        self.raw_transform.transform_point(&mut position);

        let scale = self.raw_transform.get_scale();
        dimensions.x *= scale.x;
        dimensions.y *= scale.y;

        BoundingRect::new(
            position,
            dimensions
        )
    }

    fn render(&mut self, context: &mut Canvas2d, dimensions: Vector2D) -> bool {
        self.ticks += 1;
        self.is_animating = false;

        if let Some(t) = (self.transform.generate_translation)(dimensions) {
            self.transform.set_translation(t);
        }

        if self.events.is_hovering {
            self.on_hover();

            for child in self.events.hovering_elements.iter_mut() {
                if self.events.is_hovering {
                    child.render(context, dimensions);
                } else {
                    child.destroy();
                }
            }
        } else if !self.destroyed {
            self.opacity.target = self.opacity.original;
            if !fuzzy_compare!(self.opacity.value, self.opacity.target, 1e-1) {
                self.is_animating = true;
            }
        }

        self.opacity.value = lerp!(self.opacity.value, self.opacity.target, 0.2);

        context.save();
        context.set_transform(&self.transform);
        self.raw_transform = context.get_transform();
        context.global_alpha(self.opacity.value);
        context.rotate(self.angle);

        Tank::render_turrets(context, self.stroke, self.radius, &self.turret_structure, &vec![Interpolatable::new(1.0); self.turret_structure.turrets.len()]);
        Tank::render_body(context, self.stroke, &self.body_identity, self.radius, PLAYER_FILL, PLAYER_STROKE);
        
        if DEBUG {
            context.save();
            context.reset_transform();
            self.get_bounding_rect().render(context);
            context.restore();
        }

        context.restore();

        self.destroyed && fuzzy_compare!(self.opacity.value, self.opacity.target, 1e-1)
    }

    fn destroy(&mut self) {
        self.destroyed = true;
        if self.events.deletion_effects.contains(&DeletionEffects::Disappear) {
            self.opacity.target = 0.0;
        }

        for child in self.children.iter_mut() {
            child.destroy();
        }
    }

    fn has_animation_state(&self) -> bool {
        self.is_animating
    }
}

impl Tank {
    pub fn new() -> Tank {
        Tank::default()
    }
    
    pub fn on_hover(&mut self) {
        if let Some(hover_opacity) = self.events.hover_effects.iter().find_map(
            |item| match item {
                HoverEffects::Opacity(a) => Some(*a),
                _ => None,
            })
        {
            self.opacity.target = hover_opacity;
        }
    }

    pub fn render_turrets(context: &mut Canvas2d, stroke: f32, radius: f32, turret_structure: &TurretStructure, turret_lengths: &[Interpolatable<f32>]) {
        context.save();

        context.fill_style(TURRET_FILL);
        context.stroke_style(TURRET_STROKE);
        context.set_stroke_size(stroke);

        let size_factor = radius / FICTITIOUS_TANK_RADIUS;

        for (i, turret) in turret_structure.turrets.iter().enumerate() {
            context.save();
            context.rotate(turret.angle);
            context.translate(
                turret.x_offset * size_factor,
                turret.y_offset * size_factor,
            );

            let (length, width) = (turret_lengths[i].value * (turret.length * size_factor), turret.width * size_factor);

            if turret.rendering_hints.is_empty() {
                context.fill_rect(0.0, -width / 2.0, length, width);
                context.stroke_rect(0.0, -width / 2.0, length, width);
            } else {
                for &hint in turret.rendering_hints.iter() {
                    match hint {
                        TurretRenderingHints::Trapezoidal(angle) => {
                            let height = length;
                            let bottom_width = width;
                            let top_width = width * 2.0;
                        
                            let center_x = height / 2.0;
                            let center_y = 0.0; // symmetric about the y axis?
                        
                            context.save();
                            context.translate(center_x, center_y);
                            context.rotate(angle);
                            context.translate(-center_x, -center_y);
                        
                            context.begin_path();
                            context.move_to(0.0, -bottom_width / 2.0);
                            context.line_to(height, -top_width / 2.0);
                            context.line_to(height, top_width / 2.0);
                            context.line_to(0.0, bottom_width / 2.0);
                            context.close_path();
                        
                            context.fill();
                            context.stroke();
                        
                            context.restore();
                        },                        
                        TurretRenderingHints::Trapper => {
                            context.fill_rect(0.0, -width / 2.0, length / turret_lengths[i].value, width);
                            context.stroke_rect(0.0, -width / 2.0, length / turret_lengths[i].value, width);

                            context.translate(36.0 * size_factor, 0.0);

                            let height = 12.0 * size_factor * turret_lengths[i].value;
                            let bottom_width = 24.0 * size_factor;
                            let top_width = width * 2.0;
            
                            context.save();
            
                            context.begin_path();
                            context.move_to(0.0, -bottom_width / 2.0);
                            context.line_to(height, -top_width / 2.0);
                            context.line_to(height, top_width / 2.0);
                            context.line_to(0.0, bottom_width / 2.0);
                            context.line_to(0.0, -bottom_width / 2.0);
            
                            context.fill();
                            context.stroke();
                            context.restore();
                        },
                        TurretRenderingHints::Ranger => {
                            context.fill_rect(0.0, -width / 2.0, length, width);
                            context.stroke_rect(0.0, -width / 2.0, length, width);

                            let height = 37.0 * size_factor;
                            let bottom_width = 25.0 * size_factor;
                            let top_width = width * 2.0;
                    
                            let center_x = height / 2.0;
                            let center_y = 0.0; // symmetric about the y axis?
                        
                            context.save();
                            context.translate(center_x, center_y);
                            context.rotate(std::f32::consts::PI);
                            context.translate(-center_x, -center_y);
                        
                            context.begin_path();
                            context.move_to(0.0, -bottom_width / 2.0);
                            context.line_to(height, -top_width / 2.0);
                            context.line_to(height, top_width / 2.0);
                            context.line_to(0.0, bottom_width / 2.0);
                            context.close_path();
                        
                            context.fill();
                            context.stroke();
                        
                            context.restore();
                        }
                    }
                }
            }

            context.restore();
        }
        context.restore();
    }

    pub fn render_body(context: &mut Canvas2d, stroke_size: f32, body_identity: &BodyIdentity, radius: f32, fill: Color, stroke: Color) {
        context.save();

        context.set_stroke_size(stroke_size);

        for &hint in body_identity.render_hints.iter() {
            match hint {
                BodyRenderingHints::SmasherGuard { thickness, sides } => {
                    let radius = thickness * radius;

                    context.save();
                    
                    context.fill_style(SMASHER_GUARD_FILL);
                    context.stroke_style(SMASHER_GUARD_STROKE);

                    context.begin_path();
                    context.move_to(radius, 0.0);
                    for i in 0..=sides {
                        let (x_angle, y_angle) = (std::f32::consts::TAU * i as f32 / sides as f32).sin_cos();
                        context.line_to(radius * y_angle, radius * x_angle);
                    }
                    context.close_path();
                    context.fill();
                    context.stroke();

                    context.restore();
                }
            }
        }

        context.fill_style(fill);
        context.stroke_style(stroke);
        
        context.begin_arc(0.0, 0.0, radius, std::f32::consts::TAU);
        context.fill();
        context.stroke();

        context.restore();
    }

    pub fn with_id(mut self, id: &str) -> Tank {
        self.id = id.to_string();
        self
    }

    pub fn with_transform(mut self, transform: Transform) -> Tank {
        self.transform = transform;
        self
    }

    pub fn with_raw_transform(mut self, raw_transform: Transform) -> Tank {
        self.raw_transform = raw_transform;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Tank {
        self.radius = radius;
        self
    }

    pub fn with_angle(mut self, angle: f32) -> Tank {
        self.angle = angle;
        self
    }

    pub fn with_stroke(mut self, stroke: f32) -> Tank {
        self.stroke = stroke;
        self
    }

    pub fn with_z_index(mut self, z_index: i32) -> Tank {
        self.z_index = z_index;
        self
    }

    pub fn with_children(mut self, children: Vec<Box<dyn UiElement>>) -> Tank {
        self.children = children;
        self
    }

    pub fn with_events(mut self, events: Events) -> Tank {
        self.events = events;
        self
    }

    pub fn with_opacity(mut self, opacity: f32) -> Tank {
        self.opacity = Interpolatable::new(opacity);
        self
    }

    pub fn with_body_identity(mut self, identity: BodyIdentity) -> Tank {
        self.body_identity = identity;
        self
    }

    pub fn with_turret_structure(mut self, identity: TurretStructure) -> Tank {
        self.turret_structure = identity;
        self
    }
}