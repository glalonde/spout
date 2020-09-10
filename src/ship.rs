use log::{info, trace};
use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

gflags::define! {
    --ship_acceleration: f32 = 100.0
}

gflags::define! {
    --ship_rotation_rate: f32 = 15.0
}

gflags::define! {
    --emit_velocity: f32 = 500.0
}

gflags::define! {
    --emit_velocity_spread: f32 = 0.5
}

#[cfg_attr(rustfmt, rustfmt_skip)]
#[allow(unused)]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

#[repr(i8)]
#[derive(Copy, Clone)]
pub enum RotationDirection {
    CW = -1,
    None = 0,
    CCW = 1,
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
        state.emit_params.speed_min = EMIT_VELOCITY.flag
            * (1.0 - EMIT_VELOCITY_SPREAD.flag)
            * super::int_grid::cell_size() as f32;
        state.emit_params.speed_max = EMIT_VELOCITY.flag
            * (1.0 + EMIT_VELOCITY_SPREAD.flag)
            * super::int_grid::cell_size() as f32;
        state
    }

    pub fn update(&mut self, dt: f32, accelerate: bool, rotation: RotationDirection) {
        // Update position.
        self.emit_params.position_start = [self.position[0], self.position[1]];
        // Apparently it is important to cast through i32 before going to u32.
        // So this goes, -5.1 -> -5 -> INT_MAX - 4
        self.position[0] =
            self.position[0].wrapping_add(((dt * self.velocity[0] as f32) as i32) as u32);
        self.position[1] =
            self.position[1].wrapping_add(((dt * self.velocity[1] as f32) as i32) as u32);
        self.emit_params.position_end = [self.position[0], self.position[1]];

        // Update velocity.
        self.emit_params.velocity = [self.velocity[0], self.velocity[1]];
        if accelerate {
            let delta_v = [
                (dt * self.acceleration
                    * self.orientation.cos()
                    * (super::int_grid::cell_size() as f32)) as i32,
                (dt * self.acceleration
                    * self.orientation.sin()
                    * (super::int_grid::cell_size() as f32)) as i32,
            ];

            trace!("delta_v: {:?}", delta_v);
            trace!("acceleration: {:?}", self.velocity);
            self.velocity[0] += (dt
                * self.acceleration
                * self.orientation.cos()
                * (super::int_grid::cell_size() as f32)) as i32;
            self.velocity[1] += (dt
                * self.acceleration
                * self.orientation.sin()
                * (super::int_grid::cell_size() as f32)) as i32;
            trace!("acceleration: {:?}", self.velocity);
        }

        // Update orientation.
        let angle_delta = dt * (rotation as i8 as f32) * self.rotation_rate;
        self.emit_params.angle_start = self.orientation;
        self.orientation += angle_delta;
        self.emit_params.angle_end = self.orientation;
    }
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::FromBytes, zerocopy::AsBytes)]
pub struct RenderUniforms {
    pub projection_matrix: [[f32; 4]; 4],
    pub position: [u32; 2],
    pub angle: f32,
}

pub struct ShipRenderer {
    pub render_bind_group: wgpu::BindGroup,
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_buf: wgpu::Buffer,
    pub ship_texture: wgpu::Texture,
    pub ship_texture_view: wgpu::TextureView,
}

impl ShipRenderer {
    pub fn init(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let texture_extent = wgpu::Extent3d {
            width,
            height,
            depth: 1,
        };
        let ship_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::STORAGE
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::SAMPLED,
            label: None,
        });
        let ship_texture_view = ship_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let compute_uniform_size = std::mem::size_of::<RenderUniforms>() as wgpu::BufferAddress;
        let compute_uniforms = RenderUniforms {
            projection_matrix: [[0.0; 4]; 4],
            position: [0, 0],
            angle: 0.0,
        };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Ship render uniforms"),
            contents: &compute_uniforms.as_bytes(),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        // Sets up the quad canvas.
        let vs = super::shader_utils::Shaders::get("particle_system/ship.vert.spv").unwrap();
        let vs_module = device.create_shader_module(wgpu::util::make_spirv(&vs));
        // Renders the data texture onto the canvas.
        let fs = super::shader_utils::Shaders::get("particle_system/ship.frag.spv").unwrap();
        let fs_module = device.create_shader_module(wgpu::util::make_spirv(&fs));

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            entries: &[
                // Uniforms
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
            ],
            label: None,
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ship render layout"),
                bind_group_layouts: &[&render_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Ship render pipeline"),
            layout: Some(&render_pipeline_layout),
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
                clamp_depth: false,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleStrip,
            color_states: &[wgpu::ColorStateDescriptor {
                format: wgpu::TextureFormat::Bgra8UnormSrgb,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[],
            },
            depth_stencil_state: None,
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        ShipRenderer {
            render_bind_group,
            render_pipeline,
            uniform_buf,
            ship_texture,
            ship_texture_view,
        }
    }

    fn generate_orthographic_matrix(
        level_manager: &super::level_manager::LevelManager,
    ) -> cgmath::Matrix4<f32> {
        let viewport_bottom = level_manager.height_of_viewport as f32;
        let mx_projection = cgmath::ortho(
            0.0,
            level_manager.level_width as f32,
            viewport_bottom,
            viewport_bottom + level_manager.viewport_height as f32,
            0.0,
            1.0,
        );
        let mx_correction = OPENGL_TO_WGPU_MATRIX;
        mx_correction * mx_projection
    }

    pub fn render(
        &self,
        texture_view: &wgpu::TextureView,
        device: &wgpu::Device,
        ship: &ShipState,
        level_manager: &super::level_manager::LevelManager,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let view_projection = Self::generate_orthographic_matrix(level_manager);
        // Update the ship orientation uniforms.
        let values = RenderUniforms {
            projection_matrix: cgmath::conv::array4x4(view_projection),
            position: ship.position,
            angle: ship.orientation,
        };
        let bytes: &[u8] = values.as_bytes();
        let uniform_buf_size = std::mem::size_of::<RenderUniforms>();
        let create_buffer_start = std::time::Instant::now();
        let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Staging buffer"),
            contents: bytes,
            usage: wgpu::BufferUsage::COPY_SRC,
        });
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            0,
            &self.uniform_buf,
            0,
            uniform_buf_size as wgpu::BufferAddress,
        );
        info!(
            "Ship render buffer time: {:?}",
            create_buffer_start.elapsed()
        );

        // Render the ship to a texture.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(0, &self.render_bind_group, &[]);
        // TODO don't hardcode this index count, it needs to be the number
        // of vertices in the ship.
        rpass.draw(0..4 as u32, 0..1);
    }
}
