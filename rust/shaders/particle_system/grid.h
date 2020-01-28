// Particle position units are packed strangely:
// There are two parts corrresponding to a hierarchical grid.
// The lower-resolution grid is the "pixels" on the screen output, and the resolution of the environment.
// The N most significant bits hold this higher level grid.
// The 32 - N least significant bits hold the lower level grid.
//
// So this means there is a tradeoff between the resolution of the inner grid and the outer grid.
//
// Additionally, there is a coordinate system defined so that the position data can be unsigned.
// The grid is centered at an anchor point which should be halfway into the outer grid range.
// 
// This is pretty much just enabling adjustable fixed-point math.

// Some of these should be specialization constants...
// but WebGPU doesn't support that yet, so until then, either just real constants or codegen could probably do it.
const int kInnerGridBits = 12;
const int kOuterGridBits = 32 - kInnerGridBits;
const int kOuterGridSize = 1 << kOuterGridBits;
const int kHalfOuterGridSize = kOuterGridSize / 2;
const int kGridAnchor = kHalfOuterGridSize;
const uint kHighResMask = (1 << kInnerGridBits) - 1;
const int kInnerGridSize = 1 << kInnerGridBits;
const int kHalfInnerGridSize = kInnerGridSize / 2;

// Get the "visible" position.
ivec2 GetOuterGrid(in uvec2 pos) {
  return ivec2(pos >> kInnerGridBits) - kGridAnchor;
}

// Get the high-res fractional position on the inner grid of the cell.
uvec2 GetInnerGrid(in uvec2 pos) {
  return pos & kHighResMask;
}

uvec2 SetPosition(in uvec2 outer, in uvec2 inner) {
  return (outer << kInnerGridBits) + inner;
}

uvec2 SetPositionRelative(in uvec2 outer, in uvec2 inner) {
  return SetPosition(outer + kGridAnchor, inner);
}