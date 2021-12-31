mod camera;
#[path = "../examples/framework.rs"]
mod framework;
mod game_params;
mod textured_quad;

use log::error;
use std::{borrow::Cow, mem};
use wgpu::util::DeviceExt;


/* 
gflags::define! {
    --config: &str = "game_config.toml"
}
*/

/*
TODO Render into the preloaded texture.
 */

struct Example {
    camera: camera::Camera,
    camera_bind_group: wgpu::BindGroup,
    camera_uniform_buf: wgpu::Buffer,

    draw_pipeline: wgpu::RenderPipeline,

    model: textured_quad::TexturedQuad,

    frame_num: i64,
    staging_belt: wgpu::util::StagingBelt,
}

impl Example {
    fn read_config_from_file(path: &str) -> anyhow::Result<game_params::GameParams> {
        let params = std::fs::read_to_string(path)?.parse()?;
        Ok(params)
    }

    /*
    fn get_game_config() -> game_params::GameParams {
        match Example::read_config_from_file(CONFIG.flag) {
            Ok(params) => params,
            Err(e) => {
                error!("Failed to parse config file({}): {:?}", CONFIG.flag, e);
                game_params::GameParams::default()
            }
        }
    }
    */
}

impl framework::Example for Example {
    fn optional_features() -> wgpu::Features {
        wgpu::Features::TIMESTAMP_QUERY | wgpu::Features::PIPELINE_STATISTICS_QUERY
    }

    fn init(
        config: &wgpu::SurfaceConfiguration,
        _adapter: &wgpu::Adapter,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Self {
        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // let game_params = Example::get_game_config();

        let camera = camera::Camera {
            screen_size: (config.width, config.height),
            radius: 5.0,
            phi: 0.0,
            height: 3.0,
        };
        let raw_uniforms = camera.to_uniform_data();
        let camera_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&raw_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<textured_quad::Vertex>();
        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // Vertex position.
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                // Texture position.
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 4 * 4,
                    shader_location: 1,
                },
            ],
        }];

        // Create the render pipeline
        let shader = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("textured_model.wgsl"))),
        });

        let draw_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("draw"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[config.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                // cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // Create bind group
        let camera_bind_group_layout = draw_pipeline.get_bind_group_layout(0);
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_uniform_buf.as_entire_binding(),
            }],
            label: None,
        });

        let textured_quad = textured_quad::TexturedQuad::init(
            device,
            draw_pipeline.get_bind_group_layout(1),
            &mut init_encoder,
        );

        queue.submit(Some(init_encoder.finish()));

        Example {
            camera: camera::Camera {
                screen_size: (config.width, config.height),
                radius: 5.0,
                phi: 0.0,
                height: 3.0,
            },
            camera_bind_group,
            camera_uniform_buf,
            draw_pipeline,
            model: textured_quad,

            frame_num: 0,
            staging_belt: wgpu::util::StagingBelt::new(0x100),
        }
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {}

    fn resize(
        &mut self,
        config: &wgpu::SurfaceConfiguration,
        _device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) {
        self.camera.screen_size = (config.width, config.height);
    }

    fn render(
        &mut self,
        view: &wgpu::TextureView,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _spawner: &framework::Spawner,
    ) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            // Update camera position by rotating in cylindrical coordinates.
            let p = self.frame_num as f32 / 120.0;
            self.camera.phi = p;

            let raw_uniforms = self.camera.to_uniform_data();
            self.staging_belt
                .write_buffer(
                    &mut encoder,
                    &self.camera_uniform_buf,
                    0,
                    wgpu::BufferSize::new((raw_uniforms.len() * 4) as wgpu::BufferAddress).unwrap(),
                    device,
                )
                .copy_from_slice(bytemuck::cast_slice(&raw_uniforms));

            self.staging_belt.finish();
        }

        {
            let clear_color = wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            };
            {
                let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(clear_color),
                            store: true,
                        },
                    }],
                    depth_stencil_attachment: None,
                });
                rpass.set_pipeline(&self.draw_pipeline);

                // Bind camera data.
                rpass.set_bind_group(0, &self.camera_bind_group, &[]);

                self.model.render(&mut rpass);
            }
        }

        self.frame_num += 1;

        queue.submit(Some(encoder.finish()));
    }
}

fn main() {
    framework::run::<Example>("mipmap");
}
