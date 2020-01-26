

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
//   
struct Particle {
  ivec2 position;
  ivec2 velocity;
  float ttl;
  int padding;
};

/*
struct GridParams {
  int inner_grid_bits;
  int anchor;
};

ivec2 GetCell(in GridParams grid, in uvec2 pos) {
  return ivec2(pos >> grid.inner_grid_bits) - grid.anchor;
}

uvec2 SetPosition(in GridParams grid, in uvec2 low_res, in uvec2 high_res) {
  return (low_res << grid.inner_grid_bits) + high_res;
}

uvec2 GetRemainder(in GridParams grid, in uvec2 pos) {
  const uint kHighResMask = (1 << grid.inner_grid_bits) - 1;
  return pos & kHighResMask;
}
*/