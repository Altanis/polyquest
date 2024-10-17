use shared::utils::vec2::Vector2D;
use web_sys::{wasm_bindgen::JsCast, CanvasRenderingContext2d, Document, HtmlCanvasElement, Window};

use crate::color::Color;

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

    pub fn get_width(&self) -> u32 {
        self.canvas.width()
    }
    
    pub fn get_height(&self) -> u32 {
        self.canvas.height()
    }

    pub fn get_dimensions(&self) -> Vector2D<u32> {
        Vector2D::new(self.get_width(), self.get_height())
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

    pub fn fill_style(&self, color: Color) {
        self.ctx.set_fill_style_str(&color.css());
    }

    pub fn stroke_style(&self, color: Color) {
        self.ctx.set_stroke_style_str(&color.css());
    }

    pub fn set_stroke_size(&self, size: f64) {
        self.ctx.set_line_width(size);
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

    pub fn translate<T: Into<f64>>(&self, tx: T, ty: T) {
        let tx = tx.into();
        let ty = ty.into();
        let _ = self.ctx.translate(tx, ty);
    }
    
    pub fn scale<T: Into<f64>>(&self, tx: T, ty: T) {
        let tx = tx.into();
        let ty = ty.into();
        let _ = self.ctx.scale(tx, ty);
    }    

    pub fn set_image_smoothing(&self, smooth: bool) {
        self.ctx.set_image_smoothing_enabled(smooth);
    }

    pub fn set_miter_limit(&self, limit: f64) {
        self.ctx.set_miter_limit(limit);
    }
}