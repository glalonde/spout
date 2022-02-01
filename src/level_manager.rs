use crate::buffer_util::{self, SizedBuffer};

pub struct RectangleLevel {
    width: u32,
    height: u32,
    data: Vec<i32>,
    max_dimension: u32,
    // Creation state:
    rng: fastrand::Rng,
    num_vacancies: u32,
    completed_vacancies: u32,
}

impl RectangleLevel {
    fn init(level_index: u32, level_width: u32, level_height: u32) -> Self {
        let level_num = level_index + 1;
        let max_dimension = (level_width / level_num as u32) / 2;
        let num_vacancies = (level_height as f64 * (level_num as f64).sqrt()).ceil() as u32;

        // Maximum dimension of any of the vacancies(should be a function of level_num).
        let max_dimension = std::cmp::min(max_dimension, std::cmp::min(level_width, level_height));

        // Start with a solid buffer
        let data: Vec<i32> = vec![1000; (level_width * level_height) as usize];
        RectangleLevel {
            width: level_width,
            height: level_height,
            data,
            max_dimension,
            rng: fastrand::Rng::with_seed(0),
            num_vacancies,
            completed_vacancies: 0,
        }
    }
    fn done(&self) -> bool {
        return self.completed_vacancies >= self.num_vacancies;
    }

    fn step(&mut self) {
        let width = self.rng.u32(0..self.max_dimension) as usize;
        let height = self.rng.u32(0..self.max_dimension) as usize;
        let left = self.rng.u32(0..(self.width - width as u32)) as usize;
        let bot = self.rng.u32(0..(self.height - height as u32)) as usize;
        for y in bot..(bot + height) {
            // Inner loop over rows...
            let offset = y * (self.width as usize) + left;
            self.data[offset..offset + width].fill(0);
        }
        self.completed_vacancies += 1;
    }

    fn finish(&mut self) {
        while !self.done() {
            self.step();
        }
    }

    fn work_until(&mut self, deadline: instant::Instant) -> bool {
        while !self.done() && instant::Instant::now() < deadline {
            self.step();
        }
        return self.done();
    }
}

pub struct LevelMaker {
    level_width: u32,
    level_height: u32,
    levels: Vec<Vec<i32>>,
    wip_levels: std::collections::BTreeMap<u32, RectangleLevel>,
}

impl LevelMaker {
    fn init(level_width: u32, level_height: u32) -> Self {
        LevelMaker {
            level_width,
            level_height,
            levels: vec![],
            wip_levels: std::collections::BTreeMap::new(),
        }
    }

    pub fn prefetch_up_to_level(&mut self, i: i32) {
        for level_index in self.levels.len() as u32..(i + 1) as u32 {
            if !self.wip_levels.contains_key(&level_index) {
                self.wip_levels.insert(
                    level_index,
                    RectangleLevel::init(level_index, self.level_width, self.level_height),
                );
            }
        }
    }

    pub fn work_until(&mut self, deadline: instant::Instant) {
        while instant::Instant::now() < deadline {
            let mut to_remove = Vec::new();
            for (key, value) in &mut self.wip_levels {
                if value.done() {
                    log::info!("Finished generating level: {}", key);
                    to_remove.push(*key);
                    self.levels.push(std::mem::take(&mut value.data));
                } else {
                    value.work_until(deadline);
                    break;
                }
            }
            for key in to_remove.iter() {
                self.wip_levels.remove(key);
            }
        }
    }

    pub fn use_level<F>(&mut self, i: i32, mut action: F)
    where
        F: FnMut(&Vec<i32>),
    {
        // Resolve all futures up to the requested one.
        self.prefetch_up_to_level(i);

        for level_index in self.levels.len() as u32..(i + 1) as u32 {
            if let Some(wip) = self.wip_levels.get_mut(&level_index) {
                wip.finish();
                self.levels.push(std::mem::take(&mut wip.data));
            }
            self.wip_levels.remove(&level_index);
        }
        action(&self.levels[i as usize]);
    }
}

pub struct LevelManager {
    // Static params
    pub level_width: u32,
    pub level_height: u32,
    pub viewport_height: u32,

    // Output index -> Buffer index
    pub buffer_configurations: Vec<[usize; 2]>,

    // State
    pub height_of_viewport: i32,
    // Output index -> Buffer height
    buffer_heights: Vec<i32>,

    buffer_config_index: usize,

    // Buffer index -> Buffer. (This doesn't change after init)
    terrain_buffers: Vec<SizedBuffer>,

    staging_belt: wgpu::util::StagingBelt,
    // staged_level: Vec<i32>,

    // Buffer index -> level number
    buffer_levels: Vec<i32>,

    pub level_maker: LevelMaker,

    pub terrain_renderer: TerrainRenderer,
}

impl LevelManager {
    pub fn buffer_config_index(&self) -> usize {
        self.buffer_config_index
    }
    pub fn terrain_buffer_size(&self) -> usize {
        self.terrain_buffers[0].size as usize
    }
    #[allow(dead_code)]
    pub fn current_configuration(&self) -> &[usize; 2] {
        &self.buffer_configurations[self.buffer_config_index]
    }
    pub fn buffer_configurations(&self) -> &std::vec::Vec<[usize; 2]> {
        &self.buffer_configurations
    }
    pub fn terrain_buffers(&self) -> &std::vec::Vec<SizedBuffer> {
        &self.terrain_buffers
    }
    #[allow(dead_code)]
    pub fn height_of_viewport(&self) -> i32 {
        self.height_of_viewport
    }
    #[allow(dead_code)]
    pub fn buffer_height(&self, position_index: usize) -> i32 {
        self.buffer_heights[self.current_configuration()[position_index]]
    }

    pub fn init(
        device: &wgpu::Device,
        game_params: &super::game_params::GameParams,
        height_of_viewport: i32,
        init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let level_width = game_params.level_width;
        let level_height = game_params.level_height;
        let mut buffer_configurations = vec![];
        buffer_configurations.push([0, 1]);
        buffer_configurations.push([1, 0]);

        let mut terrain_buffers = Vec::<SizedBuffer>::new();
        let mut buffer_levels = Vec::<i32>::new();
        let mut buffer_heights = Vec::<i32>::new();
        for _ in 0..2 {
            buffer_levels.push(-1);
            buffer_heights.push(0);
            terrain_buffers.push(buffer_util::make_buffer(
                device,
                level_width as usize,
                level_height as usize,
                "Terrain",
            ));
        }

        let staging_belt = wgpu::util::StagingBelt::new((terrain_buffers[0].size / 2) as u64);

        let renderer = TerrainRenderer::init(
            device,
            game_params,
            &buffer_configurations,
            &terrain_buffers,
        );

        let mut lm = LevelManager {
            level_width: game_params.level_width,
            level_height: game_params.level_height,
            viewport_height: game_params.viewport_height,
            buffer_configurations,
            height_of_viewport: -1,
            buffer_heights,
            buffer_config_index: 1,
            terrain_buffers,
            staging_belt,
            buffer_levels,
            level_maker: LevelMaker::init(level_width, level_height),
            terrain_renderer: renderer,
        };

        lm.sync_height(device, height_of_viewport, init_encoder, game_params);
        lm
    }

    pub fn sync_height(
        &mut self,
        device: &wgpu::Device,
        height_of_viewport: i32,
        encoder: &mut wgpu::CommandEncoder,
        game_params: &crate::game_params::GameParams,
    ) {
        let current_bottom_level = height_of_viewport / (self.level_height as i32);
        let current_top_level = current_bottom_level + 1;

        // self.make_levels_through(current_top_level);
        // Async request levels
        self.level_maker.prefetch_up_to_level(current_top_level + 3);

        // Update the assignment of levels to buffers.
        let new_buffer_config_index = (current_bottom_level % 2) as usize;
        if new_buffer_config_index != self.buffer_config_index {
            log::info!("New Buffer config: {}", new_buffer_config_index);
            // New configuration: We're rearranging the buffers. Need to update all of the state.
            let mut new_buffer_levels = self.buffer_levels.clone();

            // Update the buffer index to level mapping:
            for i in 0..self.buffer_levels.len() {
                let level_number = current_bottom_level
                    + ((i + new_buffer_config_index) % self.buffer_levels.len()) as i32;
                log::info!("Buffer index {} has level {}", i, level_number);
                new_buffer_levels[i] = level_number;
            }

            self.sync_buffers(device, &new_buffer_levels, encoder);
            self.buffer_config_index = new_buffer_config_index;
            self.buffer_levels = new_buffer_levels;

            for i in 0..self.buffer_levels.len() {
                self.buffer_heights[i] = self.buffer_levels[i] * self.level_height as i32;
            }
        }

        self.height_of_viewport = height_of_viewport;

        let height_of_bottom_buffer = self.buffer_height(0);
        let height_of_top_buffer = self.buffer_height(1);
        self.terrain_renderer.update_render_state(
            device,
            &game_params,
            height_of_viewport,
            height_of_bottom_buffer,
            height_of_top_buffer,
            encoder,
        );
    }

    fn sync_buffers(
        &mut self,
        device: &wgpu::Device,
        new_level_assignment: &std::vec::Vec<i32>,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Find which buffer(s) has a new level, and load it.
        assert_eq!(self.buffer_levels.len(), new_level_assignment.len());
        let it = self.buffer_levels.iter().zip(new_level_assignment.iter());

        for (buffer_index, (old, new)) in it.enumerate() {
            if old != new {
                // Drop the old level, and load in a new level
                LevelManager::copy_level_to_buffer(
                    &mut self.staging_belt,
                    &mut self.level_maker,
                    device,
                    *new,
                    &self.terrain_buffers[buffer_index],
                    encoder,
                );
            }
        }
    }

    fn copy_level_to_buffer(
        staging_belt: &mut wgpu::util::StagingBelt,
        level_maker: &mut LevelMaker,
        device: &wgpu::Device,
        level_num: i32,
        terrain_buffer: &SizedBuffer,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if level_num < 0 {
            panic!("Need a positive level num. Requested: {}", level_num);
        }

        let copy_func = |level_data: &Vec<i32>| {
            staging_belt
                .write_buffer(
                    encoder,
                    &terrain_buffer.buffer,
                    0,
                    wgpu::BufferSize::new(terrain_buffer.size as _).unwrap(),
                    device,
                )
                .copy_from_slice(bytemuck::cast_slice(level_data));
            staging_belt.finish();
        };
        level_maker.use_level(level_num, copy_func)
    }

    pub fn after_queue_submission(&mut self, spawner: &crate::framework::Spawner) {
        self.terrain_renderer.after_queue_submission(spawner);
        let belt_future = self.staging_belt.recall();
        spawner.spawn_local(belt_future);
    }
}

/*
*
* TERRAIN RENDERER
*
*
*/

// Keep track of the rendering members and logic to turn the integer particle
// density texture into a colormapped texture ready to be visualized.
pub struct TerrainRenderer {
    pub render_bind_groups: std::vec::Vec<wgpu::BindGroup>,
    pub render_pipeline: wgpu::RenderPipeline,
    pub uniform_buf: SizedBuffer,
    staging_belt: wgpu::util::StagingBelt,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct FragmentUniforms {
    pub viewport_width: u32,
    pub viewport_height: u32,

    pub height_of_viewport: i32,
    pub height_of_bottom_buffer: i32,
    pub height_of_top_buffer: i32,
}

impl TerrainRenderer {
    pub fn update_render_state(
        &mut self,
        device: &wgpu::Device,
        game_params: &super::game_params::GameParams,
        height_of_viewport: i32,
        height_of_bottom_buffer: i32,
        height_of_top_buffer: i32,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        let uniforms = FragmentUniforms {
            viewport_width: game_params.level_width,
            viewport_height: game_params.viewport_height,
            height_of_viewport,
            height_of_bottom_buffer,
            height_of_top_buffer,
        };

        // Update uniforms
        self.staging_belt
            .write_buffer(
                encoder,
                &self.uniform_buf.buffer,
                0,
                wgpu::BufferSize::new(self.uniform_buf.size as _).unwrap(),
                device,
            )
            .copy_from_slice(bytemuck::bytes_of(&uniforms));
        self.staging_belt.finish();
    }

    pub fn init(
        device: &wgpu::Device,
        // compute_locals: &super::particle_system::ComputeLocals,
        game_params: &super::game_params::GameParams,
        buffer_configurations: &Vec<[usize; 2]>,
        terrain_buffers: &Vec<SizedBuffer>,
    ) -> Self {
        let shader_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(crate::include_shader!("terrain.wgsl")),
        });

        let fragment_uniforms = FragmentUniforms {
            viewport_width: game_params.viewport_width,
            viewport_height: game_params.viewport_height,
            height_of_viewport: 0,
            height_of_bottom_buffer: 0,
            height_of_top_buffer: 0,
        };
        let uniform_buf =
            crate::buffer_util::make_uniform_buffer(device, "Uniform buffer", &fragment_uniforms);

        // Create pipeline layout
        let render_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    // Uniform inputs
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(uniform_buf.size as _),
                        },
                        count: None,
                    },
                    // Bottom terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            // TODO fill out min binding size
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    // Top terrain buffer
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            // TODO fill out min binding size
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let mut render_bind_groups = vec![];
        for config in buffer_configurations {
            render_bind_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            uniform_buf.buffer.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            terrain_buffers[config[0]].buffer.as_entire_buffer_binding(),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(
                            terrain_buffers[config[1]].buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
                label: None,
            }));
        }

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&render_bind_group_layout],
                label: None,
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain render pipeline"),
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
                    blend: None,
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

        let staging_belt = wgpu::util::StagingBelt::new(uniform_buf.size);

        TerrainRenderer {
            render_bind_groups,
            render_pipeline,
            uniform_buf,
            staging_belt,
        }
    }

    pub fn render(
        &self,
        level_manager: &super::level_manager::LevelManager,
        output_texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // Render the density texture.
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: output_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_bind_group(
            0,
            &self.render_bind_groups[level_manager.buffer_config_index()],
            &[],
        );
        rpass.draw(0..4 as u32, 0..1);
    }

    pub fn after_queue_submission(&mut self, spawner: &crate::framework::Spawner) {
        let belt_future = self.staging_belt.recall();
        spawner.spawn_local(belt_future);
    }
}
