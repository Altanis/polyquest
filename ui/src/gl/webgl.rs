use gloo::utils::document;
use wasm_bindgen::JsCast;
use web_sys::{js_sys::Float32Array, HtmlCanvasElement, WebGlRenderingContext, Window};

use crate::canvas2d::Canvas2d;

use super::shaders::{NORMAL_FRAG_SHADER, NORMAL_VERT_SHADER};

pub struct WebGl {
    canvas: HtmlCanvasElement,
    ctx: WebGlRenderingContext
}

impl Default for WebGl {
    fn default() -> Self {
        Self::new()
    }
}

fn compile_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<web_sys::WebGlShader, String> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl.get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(&shader).unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    gl: &WebGlRenderingContext,
    vert_shader: &web_sys::WebGlShader,
    frag_shader: &web_sys::WebGlShader,
) -> Result<web_sys::WebGlProgram, String> {
    let program = gl.create_program().ok_or_else(|| String::from("Unable to create program"))?;
    gl.attach_shader(&program, vert_shader);
    gl.attach_shader(&program, frag_shader);
    gl.link_program(&program);

    if gl.get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(&program).unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

impl WebGl {
    pub fn new() -> WebGl {
        let canvas = document().get_element_by_id("webgl_canvas")
            .unwrap()
            .dyn_into::<HtmlCanvasElement>()
            .unwrap();

        let ctx = canvas
            .get_context("webgl")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGlRenderingContext>()
            .unwrap();

        WebGl {
            canvas,
            ctx
        }
    }

    pub fn set_cursor(&self, style: &str) {
        let _ = self.canvas.style().set_property("cursor", style);
    }
    
    pub fn resize(&self, window: &Window) {
        self.canvas.set_width((window.inner_width().unwrap().as_f64().unwrap() * window.device_pixel_ratio()) as u32);
        self.canvas.set_height((window.inner_height().unwrap().as_f64().unwrap() * window.device_pixel_ratio()) as u32);
    }

    fn draw_crc2d(&self, context: &Canvas2d) {
        self.canvas.set_width(context.get_width());
        self.canvas.set_height(context.get_height());

        self.ctx.bind_texture(
            WebGlRenderingContext::TEXTURE_2D, 
            Some(&self.ctx.create_texture().unwrap())
        );

        self.ctx.tex_image_2d_with_u32_and_u32_and_canvas(
            WebGlRenderingContext::TEXTURE_2D,
            0,
            WebGlRenderingContext::RGBA as i32,
            WebGlRenderingContext::RGBA,
            WebGlRenderingContext::UNSIGNED_BYTE,
            context.get_canvas()
        ).expect("should've been able to convert CRC2D to WebGL");

        self.ctx.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D, 
            WebGlRenderingContext::TEXTURE_WRAP_S,
            WebGlRenderingContext::CLAMP_TO_EDGE as i32
        );

        self.ctx.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D,
            WebGlRenderingContext::TEXTURE_WRAP_T, 
            WebGlRenderingContext::CLAMP_TO_EDGE as i32
        );

        self.ctx.tex_parameteri(
            WebGlRenderingContext::TEXTURE_2D, 
            WebGlRenderingContext::TEXTURE_MIN_FILTER, 
            WebGlRenderingContext::LINEAR as i32
        );    
    }

    pub fn render(&self, context: &Canvas2d) {
        let gl = &self.ctx;

        let vert_shader = compile_shader(gl, WebGlRenderingContext::VERTEX_SHADER, NORMAL_VERT_SHADER).expect("c");
        let frag_shader = compile_shader(gl, WebGlRenderingContext::FRAGMENT_SHADER, NORMAL_FRAG_SHADER).expect("d");
    
        let program = link_program(gl, &vert_shader, &frag_shader).expect("a");

        let position_loc = gl.get_attrib_location(&program, "a_position") as u32;
        let texcoord_loc = gl.get_attrib_location(&program, "a_texcoord") as u32;
        let texture_loc = gl.get_uniform_location(&program, "u_texture");
    
        let vertices: [f32; 20] = [
            -1.0, -1.0, 0.0, 0.0, 0.0,
            1.0, -1.0, 0.0, 1.0, 0.0,   
            -1.0,  1.0, 0.0, 0.0, 1.0,
            1.0,  1.0, 0.0, 1.0, 1.0,
        ];
    
        let buffer = gl.create_buffer().ok_or("failed to create buffer").expect("b");
        gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));
        unsafe {
            let vertices_array = Float32Array::view(&vertices);
            gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vertices_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }
    
        gl.vertex_attrib_pointer_with_i32(position_loc, 3, WebGlRenderingContext::FLOAT, false, 20, 0);
        gl.enable_vertex_attrib_array(position_loc);
        gl.vertex_attrib_pointer_with_i32(texcoord_loc, 2, WebGlRenderingContext::FLOAT, false, 20, 12);
        gl.enable_vertex_attrib_array(texcoord_loc);
    
        gl.use_program(Some(&program));

        self.draw_crc2d(context);
        gl.uniform1i(texture_loc.as_ref(), 0);
        gl.viewport(0, 0, context.get_width() as i32, context.get_height() as i32);
    
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
    
        gl.draw_arrays(WebGlRenderingContext::TRIANGLE_STRIP, 0, 4);
    } 
}
