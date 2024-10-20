use shared::utils::vec2::Vector2D;
use web_sys::{wasm_bindgen::JsCast, CanvasRenderingContext2d, Document, DomMatrix, HtmlCanvasElement, TextMetrics, Window};

use crate::color::Color;

#[derive(Debug)]
pub struct Transform {
    matrix: DomMatrix
}

impl Default for Transform {
    fn default() -> Self {
        Transform::new(DomMatrix::new().unwrap())
    }
}

impl PartialEq for Transform {
    fn eq(&self, other: &Self) -> bool {
        self.a() == other.a() &&
            self.b() == other.b() &&
            self.c() == other.c() &&
            self.d() == other.d() &&
            self.e() == other.e() &&
            self.f() == other.f()
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
            matrix
        }
    }
}

impl Transform {
    pub fn new(matrix: DomMatrix) -> Transform {
        Transform {
            matrix
        }
    }

    pub fn a(&self) -> f64 { self.matrix.a() }
    pub fn b(&self) -> f64 { self.matrix.b() }
    pub fn c(&self) -> f64 { self.matrix.c() }
    pub fn d(&self) -> f64 { self.matrix.d() }
    pub fn e(&self) -> f64 { self.matrix.e() }
    pub fn f(&self) -> f64 { self.matrix.f() }

    pub fn scale(&self, sx: f64, sy: f64) -> Transform {
        Transform::new(self.matrix.scale_non_uniform_self_with_scale_y(sx, sy))
    }

    pub fn get_scale(&self) -> Vector2D<f32> {
        let scale_x = (self.matrix.a().powi(2) + self.matrix.b().powi(2)).sqrt();
        let scale_y = (self.matrix.c().powi(2) + self.matrix.d().powi(2)).sqrt();
        Vector2D::new(scale_x as f32, scale_y as f32)
    }

    pub fn rotate(&self, angle: f64) -> Transform {
        Transform::new(self.matrix.rotate_self(angle))
    }

    pub fn get_rotation(&self) -> f64 {
        self.a().atan2(self.b())
    }

    pub fn translate(&self, tx: f64, ty: f64) -> Transform {
        Transform::new(self.matrix.translate_self(tx, ty))
    }

    pub fn get_translation(&self) -> Vector2D<f32> {
        Vector2D::new(self.e() as f32, self.f() as f32)
    }

    pub fn transform_point(&self, point: &mut Vector2D<f32>) {
        point.x = self.a() as f32 * point.x + self.e() as f32;
        point.y = self.d() as f32 * point.y + self.f() as f32;
    }

    pub fn apply_transform(&mut self, transform: &Transform) {
        self.matrix = self.matrix.multiply(&transform.matrix);
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
    pub fn new(document: &Document) -> Canvas2d {
        let canvas = document.get_element_by_id("offscreen_canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let ctx = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        Canvas2d {
            canvas,
            ctx
        }
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

    pub fn clear_rect(&self) {
        self.ctx.clear_rect(0.0, 0.0, self.canvas.width() as f64, self.canvas.height() as f64);
    }

    pub fn resize(&self, window: &Window) {
        self.canvas.set_width((window.inner_width().unwrap().as_f64().unwrap() * window.device_pixel_ratio()) as u32);
        self.canvas.set_height((window.inner_height().unwrap().as_f64().unwrap() * window.device_pixel_ratio()) as u32);
    }

    pub fn fill(&self) {
        self.ctx.fill();
    }

    pub fn stroke(&self) {
        self.ctx.stroke();
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

    pub fn begin_ellipse<T: Into<f64>>(&self, x: T, y: T, r: T, radians: f64) {
        self.ctx.begin_path();
        let _ = self.ctx.arc(x.into(), y.into(), r.into(), 0.0, radians);
    }

    pub fn begin_round_rect<T: Into<f64>>(&self, x: T, y: T, w: T, h: T, r: T) {
        self.ctx.begin_path();
        let _ = self.ctx.round_rect_with_f64(x.into(), y.into(), w.into(), h.into(), r.into());
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
        Transform::new(self.ctx.get_transform().unwrap())
    }

    pub fn reset_transform(&self) {
        let _ = self.ctx.reset_transform();
    }

    pub fn set_transform(&self, transform: &Transform) {
       let _ = self.ctx.transform(transform.a(), transform.b(), transform.c(), transform.d(), transform.e(), transform.f());
    }
}