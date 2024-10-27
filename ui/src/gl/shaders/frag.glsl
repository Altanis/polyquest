precision mediump float;

uniform sampler2D u_texture;   // The texture to display
uniform vec2 u_resolution;     // The resolution of the output
uniform float u_time;          // A time uniform for animation effects

void main(void)
{
    vec2 uv = gl_FragCoord.xy / u_resolution;
    uv.y = 1.0 - uv.y;

    vec4 color = texture2D(u_texture, uv);
    gl_FragColor = color;
}