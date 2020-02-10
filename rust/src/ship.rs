use log::{info, trace};
use zerocopy::AsBytes;

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
}

#[repr(C)]
#[derive(Clone, Copy, zerocopy::FromBytes, zerocopy::AsBytes)]
pub struct RenderUniforms {
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
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Uint,
            usage: wgpu::TextureUsage::COPY_SRC
                | wgpu::TextureUsage::STORAGE
                | wgpu::TextureUsage::OUTPUT_ATTACHMENT
                | wgpu::TextureUsage::COPY_DST
                | wgpu::TextureUsage::SAMPLED,
        });
        let ship_texture_view = ship_texture.create_default_view();

        let compute_uniform_size = std::mem::size_of::<RenderUniforms>() as wgpu::BufferAddress;
        let compute_uniforms = RenderUniforms {
            position: [0, 0],
            angle: 0.0,
        };
        let uniform_buf = device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST)
            .fill_from_slice(&[compute_uniforms]);

        // Sets up the quad canvas.
        let vs = super::include_shader!("particle_system/ship.vert.spv");
        let vs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs[..])).unwrap());
        // Renders the data texture onto the canvas.
        let fs = super::include_shader!("particle_system/ship.frag.spv");
        let fs_module =
            device.create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs[..])).unwrap());

        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    // Uniform inputs
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                    },
                ],
            });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bind_group_layout,
            bindings: &[
                // Uniforms
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &uniform_buf,
                        range: 0..compute_uniform_size,
                    },
                },
            ],
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

        ShipRenderer {
            render_bind_group,
            render_pipeline,
            uniform_buf,
            ship_texture,
            ship_texture_view,
        }
    }

    pub fn render(
        &self,
        texture_view: &wgpu::TextureView,
        device: &wgpu::Device,
        ship: &ShipState,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Update the ship orientation uniforms.
        let values = RenderUniforms {
            position: ship.position,
            angle: ship.orientation,
        };
        let bytes: &[u8] = values.as_bytes();
        let uniform_buf_size = std::mem::size_of::<RenderUniforms>();
        let temp_buf = device
            .create_buffer_mapped(uniform_buf_size, wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(bytes);
        encoder.copy_buffer_to_buffer(
            &temp_buf,
            0,
            &self.uniform_buf,
            0,
            uniform_buf_size as wgpu::BufferAddress,
        );

        // Render the ship to a texture.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: texture_view,
                resolve_target: None,
                load_op: wgpu::LoadOp::Load,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
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
