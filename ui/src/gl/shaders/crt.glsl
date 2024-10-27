precision mediump float;

uniform sampler2D u_texture;
uniform vec2 u_resolution;
uniform float u_time;

float warp = 0.25; // simulate curvature of CRT monitor
float scan = 1.0; // simulate darkness between scanlines
float aberrationAmount = 0.001; // strength of chromatic aberration
float pixelSize = 768.0;

void main(void)
{
    vec2 fragCoord = gl_FragCoord.xy;
    vec2 uv = fragCoord / u_resolution;
    uv.y = 1.0 - uv.y;

    // squared distance from center
    vec2 dc = abs(0.5 - uv);
    dc *= dc;

    // warp the fragment coordinates
    uv.x -= 0.5;
    uv.x *= 1.0 + (dc.y * (0.3 * warp));
    uv.x += 0.5;
    uv.y -= 0.5;
    uv.y *= 1.0 + (dc.x * (0.4 * warp));
    uv.y += 0.5;

    // Apply pixelation effect
    uv = floor(uv * pixelSize) / pixelSize;

    vec4 fragColor;

    // sample inside boundaries, otherwise set to black
    if (uv.y > 1.0 || uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0) {
        fragColor = vec4(0.0, 0.0, 0.0, 1.0);
    } else {
        // Moving scan lines upwards with u_time
        float apply = abs(sin(fragCoord.y + u_time * 2.0) * 0.5 * scan);

        // Dynamic aberration based on scan lines
        float dynamicShift = sin(fragCoord.y * 0.01 + u_time * 5.0) * aberrationAmount;

        // Apply chromatic aberration offsets for R, G, B channels with dynamic shifts
        vec2 redOffset = uv + vec2(dynamicShift, 0.0);
        vec2 greenOffset = uv;
        vec2 blueOffset = uv - vec2(dynamicShift, 0.0);

        // Sample the texture with different offsets for each color
        float r = texture2D(u_texture, redOffset).r;
        float g = texture2D(u_texture, greenOffset).g;
        float b = texture2D(u_texture, blueOffset).b;
        
        vec3 texColor = vec3(r, g, b);

        // Mix the color with black based on scanline effect
        fragColor = vec4(mix(texColor, vec3(0.0), apply), 1.0);
    }

    gl_FragColor = fragColor;
}