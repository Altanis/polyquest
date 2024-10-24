pub const NORMAL_VERT_SHADER: &str = r#"
    attribute vec4 a_position;
    attribute vec2 a_texcoord;
    varying vec2 v_texcoord;
    void main() {
        gl_Position = a_position;
        v_texcoord = a_texcoord;
    }
"#;

pub const NORMAL_FRAG_SHADER: &str = r#"
    precision mediump float;
    uniform sampler2D u_texture;
    varying vec2 v_texcoord;
    void main() {
        vec2 flipped_texcoord = vec2(v_texcoord.x, 1.0 - v_texcoord.y);
        gl_FragColor = texture2D(u_texture, flipped_texcoord);
    }
"#;