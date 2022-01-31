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
        };

        lm.sync_height(device, height_of_viewport, init_encoder);
        lm
    }

    pub fn sync_height(
        &mut self,
        device: &wgpu::Device,
        height_of_viewport: i32,
        encoder: &mut wgpu::CommandEncoder,
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
}
