use crate::{
    buffer_util::{self, SizedBuffer},
    game_params,
};

#[repr(i8)]
#[derive(Copy, Clone)]
pub enum RotationDirection {
    CW = -1,
    None = 0,
    CCW = 1,
}

#[derive(Debug, Copy, Clone)]
pub struct ShipState {
    // This is the state in a kinematics sense, will move to the GPU eventually.
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub orientation: f32,

    // The ship's control variables.
    pub acceleration: f32,
    pub rotation_rate: f32,
    pub max_speed: f32,
}
impl Default for ShipState {
    fn default() -> Self {
        ShipState {
            position: [0.0, 0.0],
            velocity: [0.0, 0.0],
            orientation: 0.0,
            acceleration: 10.0,
            rotation_rate: 15.0,
            max_speed: 10000.0,
        }
    }
}

impl ShipState {
    pub fn init(ship_params: &game_params::ShipParams, init_pos: [f32; 2]) -> Self {
        ShipState {
            position: init_pos,
            acceleration: ship_params.acceleration,
            rotation_rate: ship_params.rotation_rate,
            max_speed: ship_params.max_speed,
            ..Default::default()
        }
    }

    pub fn update(&mut self, dt: f32, accelerate: bool, rotation: RotationDirection) {
        self.position[0] += dt * self.velocity[0];
        self.position[1] += dt * self.velocity[1];

        if accelerate {
            self.velocity[0] += dt * self.acceleration * self.orientation.cos();
            self.velocity[1] += dt * self.acceleration * self.orientation.sin();
        }
        {
            let speed = (self.velocity[0] * self.velocity[0] + self.velocity[1] * self.velocity[1]).sqrt();
            let speed_ratio = speed / self.max_speed;
            if speed_ratio > 1.0 {
                self.velocity[0] /= speed_ratio;
                self.velocity[1] /= speed_ratio;
            }
        }

        let angle_delta = dt * (rotation as i8 as f32) * self.rotation_rate;
        self.orientation += angle_delta;
    }

    pub fn get_emitter_state(&self) -> ([f32; 2], f32) {
        let emitter_offset = 2.0;
        let emitter_orientation = self.orientation + std::f32::consts::PI;
        let (mut y, mut x) = emitter_orientation.sin_cos();
        x = x * emitter_offset + self.position[0];
        y = y * emitter_offset + self.position[1];
        ([x, y], emitter_orientation)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ShipRendererUniforms {
    pub position: [f32; 2],
    pub orientation: f32,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub viewport_offset: i32,
}

pub struct ShipRenderer {
    uniform_buffer: SizedBuffer,
    render_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    staging_belt: wgpu::util::StagingBelt,
}

impl ShipRenderer {
    pub fn init(device: &wgpu::Device) -> Self {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("ship.wgsl")),
        });

        let ship_renderer_uniforms = ShipRendererUniforms {
            position: [0.0, 0.0],
            orientation: 0.0,
            viewport_width: 0,
            viewport_height: 0,
            viewport_offset: 0,
        };
        let uniform_buffer = crate::buffer_util::make_uniform_buffer::<ShipRendererUniforms>(
            device,
            "Ship Renderer Uniform Buffer",
            &ship_renderer_uniforms,
        );

        let render_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ShipRendererBGL"),
            entries: &[
                // Uniform inputs
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(uniform_buffer.size as _),
                    },
                    count: None,
                },
            ],
        });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_bgl,
            label: Some("ShipRendererBG"),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(
                    uniform_buffer.buffer.as_entire_buffer_binding(),
                ),
            }],
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Ship render pipeline layout"),
                bind_group_layouts: &[&render_bgl],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Ship render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: "fs_main",
                targets: &[wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::all(),
                }],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let staging_belt = wgpu::util::StagingBelt::new(uniform_buffer.size);
        ShipRenderer {
            uniform_buffer,
            render_bind_group,
            render_pipeline,
            staging_belt,
        }
    }

    pub fn render(
        &mut self,
        state: &ShipState,
        game_params: &game_params::GameParams,
        viewport_offset: i32,
        device: &wgpu::Device,
        output_texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Update uniforms
        let uniform_values = ShipRendererUniforms {
            position: state.position,
            orientation: state.orientation,
            viewport_width: game_params.viewport_width,
            viewport_height: game_params.viewport_height,
            viewport_offset: viewport_offset,
        };
        self.staging_belt
            .write_buffer(
                encoder,
                &self.uniform_buffer.buffer,
                0,
                wgpu::BufferSize::new(self.uniform_buffer.size as _).unwrap(),
                device,
            )
            .copy_from_slice(bytemuck::bytes_of(&uniform_values));
        self.staging_belt.finish();

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: output_texture_view,
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
        rpass.draw(0..4 as u32, 0..1);
    }

    pub fn after_queue_submission(&mut self, spawner: &crate::framework::Spawner) {
        let belt_future = self.staging_belt.recall();
        spawner.spawn_local(belt_future);
    }
}
