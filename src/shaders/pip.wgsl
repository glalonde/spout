// Picture-in-Picture overlay: dark panel + glowing corner brackets + mini ship + thruster flame.
// All coordinates are in game-view pixel space (origin bottom-left, Y up).
// The panel is placed at a fixed Y near the bottom; its X tracks the ship's world X.

const PIP_HW: f32 = 15.0;      // panel half-width  (total 30 px)
const PIP_HH: f32 = 11.0;      // panel half-height (total 22 px)
const BRACKET: f32 = 5.0;      // length of each corner tick
const SHIP_SCALE: f32 = 0.52;  // mini-ship scale factor

struct Uniforms {
    pip_center: vec2<f32>,
    ship_orientation: f32,
    viewport_width: u32,
    viewport_height: u32,
    _pad: u32,
}
@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VO { @builtin(position) pos: vec4<f32> }

// Game-view pixel → NDC.
// The game view texture is displayed via a textured quad that maps texel row 0
// to the bottom of the screen (UV flip).  In the render target NDC Y=+1 therefore
// ends up at the bottom of the final display, so we must invert Y here to match
// the rest of the game's shaders (ship, terrain, etc.).
fn px_ndc(p: vec2<f32>) -> vec2<f32> {
    return vec2<f32>(
         2.0 * p.x / f32(u.viewport_width) - 1.0,
        1.0 - 2.0 * p.y / f32(u.viewport_height),
    );
}

fn rot2(a: f32) -> mat2x2<f32> {
    return mat2x2<f32>(vec2(cos(a), sin(a)), vec2(-sin(a), cos(a)));
}

// ── Background panel ───────────────────────────────────────────────────────────

var<private> bg: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2(-PIP_HW, -PIP_HH), vec2( PIP_HW, -PIP_HH), vec2( PIP_HW,  PIP_HH),
    vec2(-PIP_HW, -PIP_HH), vec2( PIP_HW,  PIP_HH), vec2(-PIP_HW,  PIP_HH),
);

@vertex fn vs_bg(@builtin(vertex_index) i: u32) -> VO {
    return VO(vec4(px_ndc(bg[i] + u.pip_center), 0.0, 1.0));
}
@fragment fn fs_bg() -> @location(0) vec4<f32> {
    return vec4<f32>(0.02, 0.06, 0.20, 0.82);  // dark navy, semi-transparent
}

// ── Corner brackets (LineList, 16 verts = 8 segments) ─────────────────────────
// Each corner: two L-shaped ticks. Order: TL, TR, BR, BL.

var<private> bk: array<vec2<f32>, 16> = array<vec2<f32>, 16>(
    // Top-left
    vec2(-PIP_HW, PIP_HH - BRACKET), vec2(-PIP_HW,            PIP_HH),
    vec2(-PIP_HW,            PIP_HH), vec2(-PIP_HW + BRACKET,  PIP_HH),
    // Top-right
    vec2( PIP_HW - BRACKET,  PIP_HH), vec2( PIP_HW,            PIP_HH),
    vec2( PIP_HW,            PIP_HH), vec2( PIP_HW, PIP_HH - BRACKET),
    // Bottom-right
    vec2( PIP_HW, -PIP_HH + BRACKET), vec2( PIP_HW,           -PIP_HH),
    vec2( PIP_HW,           -PIP_HH), vec2( PIP_HW - BRACKET, -PIP_HH),
    // Bottom-left
    vec2(-PIP_HW + BRACKET, -PIP_HH), vec2(-PIP_HW,           -PIP_HH),
    vec2(-PIP_HW,           -PIP_HH), vec2(-PIP_HW, -PIP_HH + BRACKET),
);

@vertex fn vs_brackets(@builtin(vertex_index) i: u32) -> VO {
    return VO(vec4(px_ndc(bk[i] + u.pip_center), 0.0, 1.0));
}
@fragment fn fs_brackets() -> @location(0) vec4<f32> {
    return vec4<f32>(0.15, 1.1, 2.6, 1.0);  // electric cyan — HDR → bloom
}

// ── Mini ship body (TriangleList, 6 verts) ─────────────────────────────────────
// Shifted slightly upward inside the panel to leave room for the distance label.
const SHIP_LIFT: f32 = 1.5;

var<private> sv: array<vec2<f32>, 6> = array<vec2<f32>, 6>(
    vec2( 12.0,  0.0), vec2( -8.0,  9.0), vec2( -5.0,  0.0),
    vec2( 12.0,  0.0), vec2( -5.0,  0.0), vec2( -8.0, -9.0),
);

@vertex fn vs_ship(@builtin(vertex_index) i: u32) -> VO {
    let local = SHIP_SCALE * (rot2(u.ship_orientation) * sv[i]) + vec2(0.0, SHIP_LIFT);
    return VO(vec4(px_ndc(local + u.pip_center), 0.0, 1.0));
}
@fragment fn fs_ship() -> @location(0) vec4<f32> {
    return vec4<f32>(0.20, 0.35, 0.65, 0.92);  // steel-blue fill
}

// ── Mini ship outline (LineStrip, 5 verts) ─────────────────────────────────────

var<private> ov: array<vec2<f32>, 5> = array<vec2<f32>, 5>(
    vec2( 12.0,  0.0), vec2( -8.0,  9.0), vec2( -5.0,  0.0),
    vec2( -8.0, -9.0), vec2( 12.0,  0.0),
);

@vertex fn vs_outline(@builtin(vertex_index) i: u32) -> VO {
    let local = SHIP_SCALE * (rot2(u.ship_orientation) * ov[i]) + vec2(0.0, SHIP_LIFT);
    return VO(vec4(px_ndc(local + u.pip_center), 0.0, 1.0));
}
@fragment fn fs_outline() -> @location(0) vec4<f32> {
    return vec4<f32>(0.3, 0.7, 2.2, 1.0);  // bright electric blue — HDR → bloom
}

// ── Thruster flame (TriangleList, 3 verts) ─────────────────────────────────────
// Drawn only when thrusting (rust side passes draw count 3 or 0).

var<private> fv: array<vec2<f32>, 3> = array<vec2<f32>, 3>(
    vec2( -5.0,  3.2),   // left base, at tail notch
    vec2( -5.0, -3.2),   // right base
    vec2(-17.0,  0.0),   // flame tip — extends well behind the ship
);

@vertex fn vs_flame(@builtin(vertex_index) i: u32) -> VO {
    let local = SHIP_SCALE * (rot2(u.ship_orientation) * fv[i]) + vec2(0.0, SHIP_LIFT);
    return VO(vec4(px_ndc(local + u.pip_center), 0.0, 1.0));
}
@fragment fn fs_flame() -> @location(0) vec4<f32> {
    return vec4<f32>(2.8, 1.0, 0.05, 0.88);  // hot orange — HDR → bloom
}
