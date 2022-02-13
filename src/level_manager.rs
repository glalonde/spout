use crate::buffer_util::{self, SizedBuffer};

pub struct WIPRectangleLevel {
    width: u32,
    height: u32,
    data: Vec<i32>,
    max_dimension: u32,
    // Creation state:
    rng: fastrand::Rng,
    num_vacancies: u32,
    completed_vacancies: u32,
}

impl WIPRectangleLevel {
    fn init(level_index: u32, level_width: u32, level_height: u32) -> Self {
        let level_num = level_index + 1;
        let max_dimension = (level_width / level_num as u32) / 2;
        let num_vacancies = (level_height as f64 * (level_num as f64).sqrt()).ceil() as u32;

        // Maximum dimension of any of the vacancies(should be a function of level_num).
        let max_dimension = std::cmp::min(max_dimension, std::cmp::min(level_width, level_height));

        // Start with a solid buffer
        let data: Vec<i32> = vec![1000; (level_width * level_height) as usize];
        WIPRectangleLevel {
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

    // Finished levels, indexed by level index.
    levels: Vec<Vec<i32>>,
    // WIP levels, indexed by level index.
    wip_levels: std::collections::BTreeMap<u32, WIPRectangleLevel>,
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
                    WIPRectangleLevel::init(level_index, self.level_width, self.level_height),
                );
            }
        }
    }

    pub fn work_until(&mut self, deadline: instant::Instant) {
        while instant::Instant::now() < deadline {
            let mut to_remove = Vec::new();
            // Generate levels in order of level index.
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

    pub fn finish_through_level(&mut self, i: i32) {
        // Resolve all futures up to the requested one.
        self.prefetch_up_to_level(i);

        for level_index in self.levels.len() as u32..(i + 1) as u32 {
            if let Some(wip) = self.wip_levels.get_mut(&level_index) {
                wip.finish();
                self.levels.push(std::mem::take(&mut wip.data));
            }
            self.wip_levels.remove(&level_index);
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Interval {
    pub start: i32,
    pub end: i32,
}

impl Interval {
    pub fn intersection(&self, other: &Interval) -> Interval {
        Interval {
            start: core::cmp::max(self.start, other.start),
            end: core::cmp::min(self.end, other.end),
        }
    }
    pub fn empty(&self) -> bool {
        self.end <= self.start
    }

    pub fn intersects(&self, other: &Interval) -> bool {
        self.intersection(other).empty()
    }

    pub fn size(&self) -> i32 {
        self.end - self.start
    }
}

pub struct TerrainTile {
    // Shape of this `tile`: a 1d interval in units of rows from the start of the game.
    pub shape: Interval,
    pub buffer: SizedBuffer,
}

impl TerrainTile {
    // Returns true if there is overlap.
    pub fn copy_to_tile(
        &self,
        other: &TerrainTile,
        bytes_per_row: u64,
        encoder: &mut wgpu::CommandEncoder,
    ) -> bool {
        let intersection = self.shape.intersection(&other.shape);
        if intersection.empty() {
            return false;
        }
        let src_row_offset = (intersection.start - self.shape.start) as u64;
        let src_byte_offset = src_row_offset * bytes_per_row;
        let dst_row_offset = (intersection.start - other.shape.start) as u64;
        let dst_byte_offset = dst_row_offset * bytes_per_row;
        let copy_row_size = intersection.size() as u64;
        let copy_byte_size = copy_row_size * bytes_per_row;
        encoder.copy_buffer_to_buffer(
            &self.buffer.buffer,
            src_byte_offset,
            &other.buffer.buffer,
            dst_byte_offset,
            copy_byte_size,
        );
        return true;
    }
}

// For N levels above the current height, have level tiles ready.
// For the current level height, compute the 'active' tiles. This is the set of tiles that can interact with the particles according to some limit above and below.
// Render the 'interactive tile' by copying buffers into it.
// Run particle system on the interactive tile
// Copy the results back into the respective tiles.

pub struct LevelManager {
    // Static params
    pub level_width: u32,
    pub level_height: u32,
    pub active_extent_below_viewport: u32,
    pub active_extent_above_viewport: u32,

    // State
    // pub visible_interval: Interval,
    pub active_interval: Interval,

    loaded_tiles: std::collections::BTreeMap<i32, TerrainTile>,

    // This tile is composed of the above tiles. Each iteration, it is composited, then used, and then the results are copied out.
    composite_tile: TerrainTile,

    unused_buffers: Vec<SizedBuffer>,

    staging_belt: wgpu::util::StagingBelt,

    pub level_maker: LevelMaker,

    pub terrain_renderer: TerrainRenderer,
}

impl LevelManager {
    fn active_tiles(&self) -> impl Iterator<Item = &TerrainTile> {
        let active_interval = self.active_interval;
        self.loaded_tiles
            .iter()
            .map(move |pair| pair.1)
            .filter(move |tile| tile.shape.intersects(&active_interval))
    }

    pub fn compose_tiles(&self, encoder: &mut wgpu::CommandEncoder) {
        let BYTES_PER_ELEMENT = std::mem::size_of::<u32>() as u64;
        let bytes_per_row = self.level_width as u64 * BYTES_PER_ELEMENT;
        self.active_tiles().for_each(|f| {
            f.copy_to_tile(&self.composite_tile, bytes_per_row, encoder);
        })
    }

    pub fn decompose_tiles(&self, encoder: &mut wgpu::CommandEncoder) {
        let BYTES_PER_ELEMENT = std::mem::size_of::<u32>() as u64;
        let bytes_per_row = self.level_width as u64 * BYTES_PER_ELEMENT;
        self.active_tiles().for_each(|f| {
            self.composite_tile.copy_to_tile(&f, bytes_per_row, encoder);
        })
    }

    pub fn terrain_buffer(&self) -> &SizedBuffer {
        &self.composite_tile.buffer
    }

    pub fn init(
        device: &wgpu::Device,
        game_params: &super::game_params::GameParams,
        height_of_viewport: i32,
        init_encoder: &mut wgpu::CommandEncoder,
    ) -> Self {
        let level_width = game_params.level_width;
        let level_height = game_params.level_height;

        let composite_tile_buffer = buffer_util::make_buffer(
            device,
            level_width as usize,
            level_height as usize,
            "CompositeTerrainBuffer",
        );

        let unused_buffers = vec![buffer_util::make_buffer(
            device,
            level_width as usize,
            level_height as usize,
            "Terrain",
        )];

        let staging_belt = wgpu::util::StagingBelt::new((unused_buffers[0].size / 2) as u64);

        let renderer = TerrainRenderer::init(device, game_params, &composite_tile_buffer);

        let active_extent_below = (game_params.level_height as f64 * 0.25) as u32;
        let active_extent_above = (game_params.level_height as f64 * 0.25) as u32;
        let active_interval = LevelManager::get_active_interval(
            0,
            game_params.viewport_height as i32,
            active_extent_below as i32,
            active_extent_above as i32,
        );

        let mut lm = LevelManager {
            level_width: game_params.level_width,
            level_height: game_params.level_height,
            active_extent_below_viewport: active_extent_below,
            active_extent_above_viewport: active_extent_above,

            active_interval: active_interval,
            loaded_tiles: std::collections::BTreeMap::new(),
            composite_tile: TerrainTile {
                shape: Interval {
                    start: 0,
                    end: level_height as i32,
                },
                buffer: composite_tile_buffer,
            },

            unused_buffers: unused_buffers,
            staging_belt,
            level_maker: LevelMaker::init(level_width, level_height),
            terrain_renderer: renderer,
        };

        lm.sync_height(device, height_of_viewport, init_encoder, game_params);
        lm
    }

    pub fn block_on_levels(&mut self, active_levels: Interval) {
        for check_level_index in active_levels.start..active_levels.end {
            if check_level_index >= self.level_maker.levels.len() as i32 {
                log::warn!("Level {} not finished!", check_level_index);
                // Block until finished.
                self.level_maker.finish_through_level(check_level_index);
            }
        }
    }

    pub fn get_unused_tile_buffer(&mut self, device: &wgpu::Device) -> SizedBuffer {
        if let Some(buffer) = self.unused_buffers.pop() {
            return buffer;
        }
        buffer_util::make_buffer(
            device,
            self.level_width as usize,
            self.level_height as usize,
            "Terrain",
        )
    }

    pub fn load_active_levels(
        &mut self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        active_levels: Interval,
    ) {
        for level_index in active_levels.start..active_levels.end {
            assert!(level_index < self.level_maker.levels.len() as i32);
            // Check if this level has a buffer, if not, find it one.
            // self.level_maker.use_level(check_level_index, action);
            if !self.loaded_tiles.contains_key(&level_index) {
                // Tile isn't loaded, find a buffer.
                let buffer = self.get_unused_tile_buffer(device);
                let level_data = &self.level_maker.levels[level_index as usize];

                // Request data copy.
                self.staging_belt
                    .write_buffer(
                        encoder,
                        &buffer.buffer,
                        0,
                        wgpu::BufferSize::new(buffer.size as _).unwrap(),
                        device,
                    )
                    .copy_from_slice(bytemuck::cast_slice(level_data));
                let level_start = level_index * self.level_height as i32;
                let unused_buffer = self.get_unused_tile_buffer(device);
                self.loaded_tiles.insert(
                    level_index,
                    TerrainTile {
                        shape: Interval {
                            start: level_start,
                            end: level_start + self.level_height as i32,
                        },
                        buffer: unused_buffer,
                    },
                );
            }
        }

        self.staging_belt.finish();
    }

    pub fn get_active_interval(
        viewport_offset: i32,
        viewport_height: i32,
        active_extent_below: i32,
        active_extent_above: i32,
    ) -> Interval {
        let viewport_bottom = viewport_offset;
        let viewport_top = viewport_bottom + viewport_height;
        Interval {
            start: viewport_bottom - active_extent_below,
            end: viewport_top + active_extent_above,
        }
    }

    pub fn update_active_interval(
        &mut self,
        viewport_offset: i32,
        game_params: &crate::game_params::GameParams,
    ) {
        self.active_interval = LevelManager::get_active_interval(
            viewport_offset,
            game_params.viewport_height as i32,
            self.active_extent_below_viewport as i32,
            self.active_extent_above_viewport as i32,
        );
    }

    pub fn sync_height(
        &mut self,
        device: &wgpu::Device,
        viewport_offset: i32,
        encoder: &mut wgpu::CommandEncoder,
        game_params: &crate::game_params::GameParams,
    ) {
        log::info!("Syncing to height: {}", viewport_offset);
        self.update_active_interval(viewport_offset, game_params);

        // Find all level indices corresponding to active interval.
        let active_levels = Interval {
            start: std::cmp::min(self.active_interval.start / self.level_height as i32, 0),
            end: std::cmp::min(self.active_interval.end / self.level_height as i32, 0),
        };
        let on_deck_levels = Interval {
            start: active_levels.start,
            end: active_levels.end + 3,
        };

        // Start making the upcoming levels.
        self.level_maker.prefetch_up_to_level(on_deck_levels.end);

        // Find levels we need, make sure they're done (blocking) and loaded into the gpu(blocking)
        self.load_active_levels(device, encoder, active_levels);

        // TODO when a level is no longer needed, recycle the buffer.

        /*
        self.terrain_renderer.update_render_state(
            device,
            &game_params,
            height_of_viewport,
            height_of_bottom_buffer,
            height_of_top_buffer,
            encoder,
        );
        */
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
    pub render_bind_group: wgpu::BindGroup,
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
        game_params: &super::game_params::GameParams,
        composite_terrain_buffer: &SizedBuffer,
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
                            min_binding_size: std::num::NonZeroU64::new(
                                composite_terrain_buffer.size,
                            ),
                        },
                        count: None,
                    },
                ],
                label: None,
            });

        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
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
                        composite_terrain_buffer.buffer.as_entire_buffer_binding(),
                    ),
                },
            ],
            label: None,
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&render_bind_group_layout],
                label: Some("Terrain render pipeline layout"),
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
            render_bind_group,
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
        rpass.set_bind_group(0, &self.render_bind_group, &[]);
        rpass.draw(0..4 as u32, 0..1);
    }

    pub fn after_queue_submission(&mut self, spawner: &crate::framework::Spawner) {
        let belt_future = self.staging_belt.recall();
        spawner.spawn_local(belt_future);
    }
}
