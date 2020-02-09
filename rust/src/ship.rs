use log::{info, trace};

gflags::define! {
    --ship_acceleration: f32 = 100.0
}

gflags::define! {
    --ship_rotation_rate: f32 = 15.0
}

gflags::define! {
    --ship_emit_velocity: f32 = 100.0
}

gflags::define! {
    --ship_emit_velocity_spread: f32 = 0.5
}

#[repr(i8)]
#[derive(Copy, Clone)]
pub enum RotationDirection {
    CW = -1,
    None = 0,
    CCW = 1,
}

pub struct Rendering {
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
}

#[derive(Debug)]
pub struct ShipState {
    // This is the state in a kinematics sense, will move to the GPU eventually.
    pub position: [u32; 2],
    pub velocity: [i32; 2],
    pub orientation: f32,

    // The ship's control variables.
    pub rotation_rate: f32,
    pub acceleration: f32,

    // The ships's particle emitter
    pub emit_params: super::emitter::EmitParams,
}

impl ShipState {
    pub fn init(position: [u32; 2]) -> Self {
        ShipState {
            position: position,
            velocity: [0, 0],
            orientation: 0.0,
            rotation_rate: 15.0,
            acceleration: 100.0,
            emit_params: super::emitter::EmitParams::default(),
        }
    }

    pub fn init_from_flags(position: [u32; 2]) -> Self {
        let mut state = ShipState {
            position: position,
            velocity: [0, 0],
            orientation: 0.0,
            rotation_rate: SHIP_ROTATION_RATE.flag,
            acceleration: SHIP_ACCELERATION.flag,
            emit_params: super::emitter::EmitParams::default(),
        };
        state.emit_params.speed_min = SHIP_EMIT_VELOCITY.flag
            * (1.0 - SHIP_EMIT_VELOCITY_SPREAD.flag)
            * super::int_grid::cell_size() as f32;
        state.emit_params.speed_max = SHIP_EMIT_VELOCITY.flag
            * (1.0 + SHIP_EMIT_VELOCITY_SPREAD.flag)
            * super::int_grid::cell_size() as f32;
        state
    }

    pub fn update(&mut self, dt: f32, accelerate: bool, rotation: RotationDirection) {
        // Update position.
        self.emit_params.position_start = [self.position[0], self.position[1]];
        self.position[0] = self.position[0].wrapping_add((dt * self.velocity[0] as f32) as u32);
        self.position[1] = self.position[1].wrapping_add((dt * self.velocity[1] as f32) as u32);
        self.emit_params.position_end = [self.position[0], self.position[1]];

        // Update velocity.
        self.emit_params.velocity = [self.velocity[0], self.velocity[1]];
        if accelerate {
            info!("acceleration: {:?}", self.velocity);
            self.velocity[0] += (dt
                * self.acceleration
                * self.orientation.cos()
                * (super::int_grid::cell_size() as f32)) as i32;
            self.velocity[1] += (dt
                * self.acceleration
                * self.orientation.sin()
                * (super::int_grid::cell_size() as f32)) as i32;
        }

        // Update orientation.
        let angle_delta = dt * (rotation as i8 as f32) * self.rotation_rate;
        self.emit_params.angle_start = self.orientation;
        self.orientation += angle_delta;
        self.emit_params.angle_end = self.orientation;
    }

    pub fn make_render_pipeline(_sc_desc: &wgpu::SwapChainDescriptor, device: &wgpu::Device) {
        // Sets up the quad canvas.
        let vs = super::include_shader!("particle_system/ship.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = super::include_shader!("particle_system/ship.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { bindings: &[] });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            bindings: &[],
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&render_bind_group_layout],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint16,
            vertex_buffers: &[],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });
    }
}
