// Faint diagonal line for the Triangle touch-control scheme.
//
// Renders a thin rectangle along the diagonal from screen-space (W/2, 0) to
// (W, H) — the line that splits the right-half rotate zones into CW (above)
// and CCW (below). The intent is a barely-visible hint while the player
// learns the scheme; it should not compete with gameplay visuals.
//
// The line is opaque near the bottom-right corner where the diagonal meets
// the screen edge, and fades to nothing as it climbs toward the top-center
// start — so only the bottom portion of the diagonal is drawn, enough to
// suggest the boundary without cutting across the whole screen.
//
// Drawn after the composite pass directly onto the surface. Uses alpha
// blending; alpha is small so the line reads as a faint glow rather than a
// hard divider.

struct Uniforms {
    surface_size: vec2<f32>, // pixels
    thickness_px: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    // Distance along the diagonal, 0.0 at the top-center start, 1.0 at the
    // bottom-right end. Used by the fragment shader to fade out.
    @location(0) along: f32,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    // Endpoints in screen-pixel space (origin top-left, y down).
    let start = vec2<f32>(u.surface_size.x * 0.5, 0.0);
    let end = vec2<f32>(u.surface_size.x, u.surface_size.y);

    let dir = normalize(end - start);
    let perp = vec2<f32>(-dir.y, dir.x);
    let half_thick = perp * (u.thickness_px * 0.5);

    // Triangle strip: 4 vertices forming a thin rectangle along the diagonal.
    // Vertices 0,1 sit at the start (along=0), 2,3 sit at the end (along=1).
    var pos: vec2<f32>;
    var along: f32;
    if vi == 0u {
        pos = start - half_thick;
        along = 0.0;
    } else if vi == 1u {
        pos = start + half_thick;
        along = 0.0;
    } else if vi == 2u {
        pos = end - half_thick;
        along = 1.0;
    } else {
        pos = end + half_thick;
        along = 1.0;
    }

    // Pixel space → NDC. Y-axis is flipped (screen y down → NDC y up).
    let ndc = vec2<f32>(
        pos.x / u.surface_size.x * 2.0 - 1.0,
        -(pos.y / u.surface_size.y * 2.0 - 1.0),
    );
    var out: VertexOutput;
    out.position = vec4<f32>(ndc, 0.0, 1.0);
    out.along = along;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Faintly visible at the bottom-right end of the line (along = 1.0);
    // eased to invisible by ~1/3 of the way up the diagonal (along ≈ 0.67).
    let peak_alpha: f32 = 0.4;
    let fade = smoothstep(0.67, 1.0, in.along);
    return vec4<f32>(1.0, 0.95, 0.85, peak_alpha * fade);
}
