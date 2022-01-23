struct DensityBuffer {
    data: [[stride(4)]] array<u32>;
};
[[group(0), binding(0)]]
var<storage, read_write> density_buffer: DensityBuffer;

[[stage(compute), workgroup_size(256)]]
fn main([[builtin(global_invocation_id)]] global_id: vec3<u32>,  [[builtin(num_workgroups)]] num_workgroups: vec3<u32>) {
  let gid = global_id[0];
  density_buffer.data[gid] = 0u;
}