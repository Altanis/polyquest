use gloo::utils::{document, window};
use shared::{rand, utils::{color::Color, vec2::Vector2D}};
use web_sys::{wasm_bindgen::JsCast, CanvasRenderingContext2d, DomMatrix, HtmlCanvasElement, TextMetrics, Window};
use rand::Rng;

use crate::core::GenerateTranslationScript;

#[macro_export]
macro_rules! translate {
    ($a:expr, $b:expr) => {
        Transform::default().translate($a, $b)
    }
}

#[macro_export]
macro_rules! scale {
    ($a:expr, $b:expr) => {
        Transform::default().scale($a, $b)
    }
}

#[macro_export]
macro_rules! rotate {
    ($a:expr) => {
        Transform::default().rotate($a)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShapeType {
    Circle,
    Square,
    Triangle,
    Pentagon
}

impl ShapeType {
    pub fn random() -> ShapeType {
        match rand!(0, 3) {
            0 => ShapeType::Circle,
            1 => ShapeType::Square,
            2 => ShapeType::Triangle,
            3 => ShapeType::Pentagon,
            _ => panic!("invalid shape?")
        }
    }

    pub fn render(&self, context: &Canvas2d, radius: f32, fill: bool, stroke: bool) {
        match *self {
            ShapeType::Circle => context.begin_arc(0.0, 0.0, radius, std::f64::consts::TAU),
            ShapeType::Square => context.begin_rect(-radius, -radius, radius * 2.0, radius * 2.0),
            ShapeType::Triangle => context.begin_triangle(radius),
            ShapeType::Pentagon => context.begin_pentagon(radius),
        }

        if fill {
            context.fill();
        }

        if stroke {
            context.stroke();
        }
    }
}

pub struct Transform {
    matrix: DomMatrix,
    pub generate_translation: Box<dyn GenerateTranslationScript>
}

impl Default for Transform {
    fn default() -> Self {
        Transform::new(DomMatrix::new().unwrap(), Box::default())
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        let epsilon = 1e-6; // Example tolerance value

        (self.a() - other.a()).abs() < epsilon &&
        (self.b() - other.b()).abs() < epsilon &&
        (self.c() - other.c()).abs() < epsilon &&
        (self.d() - other.d()).abs() < epsilon &&
        (self.e() - other.e()).abs() < epsilon &&
        (self.f() - other.f()).abs() < epsilon
    }
}

impl Clone for Transform {
    fn clone(&self) -> Self {
        let matrix = DomMatrix::new().unwrap();
        matrix.set_a(self.a());
        matrix.set_b(self.b());
        matrix.set_c(self.c());
        matrix.set_d(self.d());
        matrix.set_e(self.e());
        matrix.set_f(self.f());

        Transform {
            matrix,
            generate_translation: self.generate_translation.clone_box()
        }
    }
}

impl Transform {
    pub fn new(matrix: DomMatrix, generate_translation: Box<dyn GenerateTranslationScript>) -> Transform {
        Transform {
            matrix,
            generate_translation
        }
    }

    pub fn a(&self) -> f64 { self.matrix.a() }
    pub fn b(&self) -> f64 { self.matrix.b() }
    pub fn c(&self) -> f64 { self.matrix.c() }
    pub fn d(&self) -> f64 { self.matrix.d() }
    pub fn e(&self) -> f64 { self.matrix.e() }
    pub fn f(&self) -> f64 { self.matrix.f() }

    pub fn scale(&self, sx: f64, sy: f64) -> Transform {
        Transform::new(
            self.matrix.scale_non_uniform_self_with_scale_y(sx, sy), 
            self.generate_translation.clone_box()
        )
    }

    pub fn get_scale(&self) -> Vector2D<f32> {
        let scale_x = (self.matrix.a().powi(2) + self.matrix.b().powi(2)).sqrt();
        let scale_y = (self.matrix.c().powi(2) + self.matrix.d().powi(2)).sqrt();
        Vector2D::new(scale_x as f32, scale_y as f32)
    }

    pub fn rotate(&self, angle: f64) -> Transform {
        Transform::new(
            self.matrix.rotate_self(angle), 
            self.generate_translation.clone_box()
        )
    }

    pub fn get_rotation(&self) -> f64 {
        self.a().atan2(self.b())
    }

    pub fn translate(&self, tx: f64, ty: f64) -> Transform {
        Transform::new(
            self.matrix.translate_self(tx, ty), 
            self.generate_translation.clone_box()
        )
    }

    pub fn get_translation(&self) -> Vector2D<f32> {
        Vector2D::new(self.e() as f32, self.f() as f32)
    }

    pub fn set_translation(&self, translation: Vector2D<f32>) {
        self.matrix.set_e(translation.x as f64);
        self.matrix.set_f(translation.y as f64);
    }

    pub fn transform_point(&self, point: &mut Vector2D<f32>) {
        point.x = self.a() as f32 * point.x + self.e() as f32;
        point.y = self.d() as f32 * point.y + self.f() as f32;
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        self.matrix = self.matrix.multiply(&transform.matrix);
    }

    pub fn get_inverse(&self) -> Transform {
        Transform::new(
            self.matrix.inverse(),
            self.generate_translation.clone_box()
        )
    }

    pub fn lerp_towards(&self, transform: &Transform, factor: f32) {
        let a = self.a() + (transform.a() - self.a()) * factor as f64;
        let b = self.b() + (transform.b() - self.b()) * factor as f64;
        let c = self.c() + (transform.c() - self.c()) * factor as f64;
        let d = self.d() + (transform.d() - self.d()) * factor as f64;
        let e = self.e() + (transform.e() - self.e()) * factor as f64;
        let f = self.f() + (transform.f() - self.f()) * factor as f64;

        self.matrix.set_a(a);
        self.matrix.set_b(b);
        self.matrix.set_c(c);
        self.matrix.set_d(d);
        self.matrix.set_e(e);
        self.matrix.set_f(f);
    }
}

pub struct Canvas2d {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d
}

impl Canvas2d {
    pub fn new(id: &str) -> Canvas2d {
        let canvas = if id.is_empty() {
            document().create_element("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap()
        } else {
            get_element_by_id_and_cast!(id, HtmlCanvasElement)
        };

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let canvas = Canvas2d {
            canvas,
            ctx
        };

        canvas.resize();

        canvas
    }

    pub fn get_canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }

    pub fn set_cursor(&self, style: &str) {
        let _ = self.canvas.style().set_property("cursor", style);
    }

    pub fn get_width(&self) -> u32 {
        self.canvas.width()
    }
    
    pub fn get_height(&self) -> u32 {
        self.canvas.height()
    }

    pub fn get_dimensions(&self) -> Vector2D<f32> {
        Vector2D::new(self.get_width() as f32, self.get_height() as f32)
    }

    pub fn save(&self) {
        self.ctx.save();
    }

    pub fn restore(&self) {
        self.ctx.restore();
    }

    pub fn set_line_join(&self, value: &str) {
        self.ctx.set_line_join(value);
    }

    pub fn clear_rect(&self) {
        self.ctx.clear_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64);
    }

    pub fn global_composite_operation(&self, operation: &str) {
        let _ = self.ctx.set_global_composite_operation(operation);
    }

    pub fn resize(&self) {
        self.canvas.set_width((window().inner_width().unwrap().as_f64().unwrap() * window().device_pixel_ratio()) as u32);
        self.canvas.set_height((window().inner_height().unwrap().as_f64().unwrap() * window().device_pixel_ratio()) as u32);
    }

    pub fn fill(&self) {
        self.ctx.fill();
    }

    pub fn stroke(&self) {
        self.ctx.stroke();
    }

    pub fn begin_path(&self) {
        self.ctx.begin_path();
    }

    pub fn close_path(&self) {
        self.ctx.close_path();
    }

    pub fn move_to(&self, x: f32, y: f32) {
        self.ctx.move_to(x as f64, y as f64);
    }

    pub fn line_to(&self, x: f32, y: f32) {
        self.ctx.line_to(x as f64, y as f64);
    }

    pub fn fill_style(&self, color: Color) {
        self.ctx.set_fill_style_str(&color.css());
    }

    pub fn stroke_style(&self, color: Color) {
        self.ctx.set_stroke_style_str(&color.css());
    }

    pub fn set_stroke_size<T: Into<f64>>(&self, size: T) {
        self.ctx.set_line_width(size.into());
    }

    pub fn shadow_color(&self, color: Color) {
        self.ctx.set_shadow_color(&color.css());
    }

    pub fn shadow_blur(&self, blur: f64) {
        self.ctx.set_shadow_blur(blur);
    }

    pub fn global_alpha(&self, alpha: f64) {
        self.ctx.set_global_alpha(alpha);
    }

    pub fn set_font(&self, font: &str) {
        self.ctx.set_font(font);
    }

    pub fn set_text_align(&self, align: &str) {
        self.ctx.set_text_align(align);
    }

    pub fn stroke_text(&self, text: &str) {
        let _ = self.ctx.stroke_text(text, 0.0, 0.0);
    }

    pub fn fill_text(&self, text: &str) {
        let _ = self.ctx.fill_text(text, 0.0, 0.0);
    }

    pub fn measure_text(&self, text: &str) -> TextMetrics {
        self.ctx.measure_text(text).unwrap()
    }

    pub fn fill_rect<T: Into<f64>>(&self, x: T, y: T, w: T, h: T) {
        self.ctx.fill_rect(x.into(), y.into(), w.into(), h.into());
    }

    pub fn stroke_rect<T: Into<f64>>(&self, x: T, y: T, w: T, h: T) {
        self.ctx.stroke_rect(x.into(), y.into(), w.into(), h.into());
    }

    pub fn begin_check_mark(&self, position: Vector2D<f64>, dimensions: Vector2D<f64>) {
        self.ctx.begin_path();
        self.ctx.move_to(position.x, position.y + dimensions.y * 0.5);
        self.ctx.line_to(position.x + dimensions.x * 0.4, position.y + dimensions.y);
        self.ctx.line_to(position.x + dimensions.x * 0.4, position.y);
    }

    pub fn begin_arc<T: Into<f64>>(&self, x: T, y: T, r: T, radians: f64) {
        self.ctx.begin_path();
        let _ = self.ctx.arc(x.into(), y.into(), r.into(), 0.0, radians);
    }

    pub fn begin_rect<T: Into<f64>>(&self, x: T, y: T, w: T, h: T) {
        self.ctx.begin_path();
        self.ctx.rect(x.into(), y.into(), w.into(), h.into());
    }

    pub fn begin_round_rect<T: Into<f64>>(&self, x: T, y: T, w: T, h: T, r: T) {
        self.ctx.begin_path();
        let _ = self.ctx.round_rect_with_f64(x.into(), y.into(), w.into(), h.into(), r.into());
    }

    pub fn begin_triangle<T: Into<f64>>(&self, r: T) {
        let radius = r.into();

        self.ctx.begin_path();
        self.ctx.move_to(0.0, -radius * 1.3);
        self.ctx.line_to(radius * 1.3 * 0.8660254037844387, radius * 1.3 * 0.5);
        self.ctx.line_to(-radius * 1.3 * 0.8660254037844387, radius * 1.3 * 0.5);
        self.ctx.close_path();
    }

    pub fn begin_pentagon<T: Into<f64>>(&self, r: T) {
        let radius = r.into();

        self.ctx.begin_path();
        self.ctx.move_to(0.0, -radius);
        self.ctx.line_to(radius * 0.9510565162951535, -radius * 0.30901699437494745);
        self.ctx.line_to(radius * 0.5877852522924731, radius * 0.8090169943749473);
        self.ctx.line_to(-radius * 0.587785252292473, radius * 0.8090169943749475);
        self.ctx.line_to(-radius * 0.9510565162951536, -radius * 0.30901699437494734);
        self.ctx.close_path();
    }

    pub fn begin_star(&self, points: u32, outer_radius: f32, inner_radius: f32) {
        let step = std::f32::consts::PI / points as f32;
        let mut angle = std::f32::consts::PI / 2.0 * 3.0;

        self.begin_path();
        for _ in 0..points {
            let outer = Vector2D::from_polar(outer_radius, angle);
            self.line_to(outer.x, outer.y);
            angle += step;

            let inner = Vector2D::from_polar(inner_radius, angle);
            self.line_to(inner.x, inner.y);
            angle += step;
        }
        self.close_path();
    }

    pub fn translate<T: Into<f64>>(&self, tx: T, ty: T) {
        let _ = self.ctx.translate(tx.into(), ty.into());
    }
    
    pub fn scale<T: Into<f64>>(&self, tx: T, ty: T) {
        let _ = self.ctx.scale(tx.into(), ty.into());
    }

    pub fn rotate<T: Into<f64>>(&self, r: T) {
        let _ = self.ctx.rotate(r.into());
    }   

    pub fn set_image_smoothing(&self, smooth: bool) {
        self.ctx.set_image_smoothing_enabled(smooth);
    }

    pub fn set_miter_limit(&self, limit: f64) {
        self.ctx.set_miter_limit(limit);
    }

    pub fn get_transform(&self) -> Transform {
        Transform::new(self.ctx.get_transform().unwrap(), Box::default())
    }

    pub fn reset_transform(&self) {
        let _ = self.ctx.reset_transform();
    }

    pub fn set_transform(&self, transform: &Transform) {
       let _ = self.ctx.transform(transform.a(), transform.b(), transform.c(), transform.d(), transform.e(), transform.f());
    }

    pub fn set_line_cap(&self, cap: &str) {
        self.ctx.set_line_cap(cap);
    }
}