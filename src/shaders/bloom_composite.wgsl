struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
};

var<private> vertex_positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(-1.0, -1.0),
    vec2<f32>(-1.0,  1.0),
    vec2<f32>( 1.0, -1.0),
    vec2<f32>( 1.0,  1.0),
);
var<private> tex_coords: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
    vec2<f32>(0.0, 1.0),
    vec2<f32>(0.0, 0.0),
    vec2<f32>(1.0, 1.0),
    vec2<f32>(1.0, 0.0),
);

@vertex
fn vs_main(@builtin(vertex_index) i: u32) -> VertexOutput {
    var out: VertexOutput;
    out.position = vec4<f32>(vertex_positions[i], 0.0, 1.0);
    out.tex_coord = tex_coords[i];
    return out;
}

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;
@group(0) @binding(2) var bloom_tex: texture_2d<f32>;
@group(0) @binding(3) var bloom_sampler: sampler;

// Bloom intensity in the final composite.
override bloom_strength: f32 = 1.0;
// CRT post-process intensity. 0.0 = bypass, 1.0 = full effect.
override crt_strength: f32 = 0.0;

// Barrel distortion: k > 0 bows the image outward (CRT-style).
fn barrel(uv: vec2<f32>, k: f32) -> vec2<f32> {
    let c = uv * 2.0 - vec2<f32>(1.0);
    return (c * (1.0 + k * dot(c, c))) * 0.5 + vec2<f32>(0.5);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // ── Barrel distortion ──────────────────────────────────────────────────
    let buv = barrel(in.tex_coord, 0.10 * crt_strength);

    // Bezel: black outside the distorted screen boundary.
    if any(buv < vec2<f32>(0.0)) || any(buv > vec2<f32>(1.0)) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // ── Chromatic aberration (scales with distance from centre) ───────────
    let centre_off = buv - vec2<f32>(0.5);
    let ca_dist    = length(centre_off);
    let ca_dir     = select(vec2<f32>(1.0, 0.0), normalize(centre_off), ca_dist > 0.0001);
    let ca         = ca_dir * ca_dist * crt_strength * 0.018;

    // Sample R, G, B from slightly offset UVs; fold bloom into the same samples.
    let r = textureSample(hdr_tex,   hdr_sampler,   buv + ca).r
          + textureSample(bloom_tex, bloom_sampler, buv + ca).r * bloom_strength;
    let g = textureSample(hdr_tex,   hdr_sampler,   buv     ).g
          + textureSample(bloom_tex, bloom_sampler, buv     ).g * bloom_strength;
    let b = textureSample(hdr_tex,   hdr_sampler,   buv - ca).b
          + textureSample(bloom_tex, bloom_sampler, buv - ca).b * bloom_strength;

    var color = clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0));

    // ── Aperture-grille phosphor mask (vertical R/G/B stripes) ────────────
    let col   = u32(in.position.x) % 3u;
    var mask  = vec3<f32>(0.5, 0.5, 0.5);
    if      col == 0u { mask.r = 1.0; }
    else if col == 1u { mask.g = 1.0; }
    else              { mask.b = 1.0; }
    color *= mix(vec3<f32>(1.0), mask, crt_strength * 0.75);

    // ── Scanlines (one bright + one dark row per pixel pair) ──────────────
    let scan = sin(in.position.y * 3.14159265) * 0.5 + 0.5;
    color   *= mix(1.0, 0.3 + 0.7 * scan, crt_strength * 0.8);

    // ── Vignette (darken corners) ──────────────────────────────────────────
    let vig = buv * (1.0 - buv.yx);
    color  *= pow(clamp(vig.x * vig.y * 16.0, 0.0, 1.0), crt_strength * 0.4);

    return vec4<f32>(color, 1.0);
}
