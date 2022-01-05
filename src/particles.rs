

pub struct ParticleSystem {
    params: EmitterParams,

    // State
    time: f32,
    dt: f32,
    emit_progress: f32,
    write_index: u32,

    // This holds the state of the current update's emit, in between update and compute.
    emit_params: Option<EmitParams>,

    // GPU interface cruft
    compute_work_groups: u32,
    compute_bind_group: wgpu::BindGroup,
    uniform_buffer: SizedBuffer,
    pub particle_buffer: SizedBuffer,
    compute_pipeline: wgpu::ComputePipeline,
    staging_belt: wgpu::util::StagingBelt,
}