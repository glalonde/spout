use std::{collections::VecDeque, pin::Pin};

use futures::{task::SpawnExt, Future};
use log::info;
use wgpu::util::DeviceExt;
use zerocopy::AsBytes;

pub struct LevelMaker {
    level_width: u32,
    level_height: u32,
    levels: Vec<Vec<i32>>,
    pool: futures::executor::ThreadPool,
    future_levels: std::collections::HashMap<i32, Pin<Box<dyn Future<Output = Vec<i32>>>>>,
}

impl LevelMaker {
    fn init(level_width: u32, level_height: u32) -> Self {
        LevelMaker {
            level_width,
            level_height,
            levels: vec![],
            pool: futures::executor::ThreadPool::new().unwrap(),
            future_levels: std::collections::HashMap::new(),
        }
    }
    fn make_level(level_num: i32, level_height: u32, level_width: u32) -> Vec<i32> {
        info!("Making level: {}", level_num);
        image::ImageBuffer::<image::Luma<i32>, Vec<i32>>::from_fn(
            level_width,
            level_height,
            |x, y| {
                let (index, _) = match level_num % 2 {
                    0 => (x, level_width),
                    1 => (y, level_height),
                    _ => panic!(),
                };
                match index % 5 {
                    0 => image::Luma::<i32>([1000]),
                    _ => image::Luma::<i32>([0]),
                }
            },
        )
        .into_raw()
    }
    pub fn prefetch_up_to_level(&mut self, i: i32) {
        for level_num in self.levels.len() as i32..(i + 1) {
            if !self.future_levels.contains_key(&level_num) {
                let width = self.level_width;
                let height = self.level_height;
                let future_level = async move {
                    log::info!("Making level: {}", i);
                    LevelMaker::make_level(level_num as i32, height, width)
                };
                // Resolve ASAP on threadpool.
                let handle = self.pool.spawn_with_handle(future_level).unwrap();
                self.future_levels.insert(level_num, Box::pin(handle));
            }
        }
    }

    pub fn use_level<F>(&mut self, i: i32, mut action: F)
    where
        F: FnMut(&Vec<i32>),
    {
        // Resolve all futures up to the requested one.
        for level_num in self.levels.len() as i32..(i + 1) {
            // If the unwrap fails, then prefetch probably wasn't called.
            self.levels.push(futures::executor::block_on(
                self.future_levels.get_mut(&level_num).unwrap(),
            ));
            log::info!("Resolved level: {}", level_num);
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
    terrain_buffer_size: usize,

    // Buffer index -> Buffer. (This doesn't change after init)
    terrain_buffers: Vec<wgpu::Buffer>,
    // Buffer index -> level number
    buffer_levels: Vec<i32>,

    level_maker: LevelMaker,
}

impl LevelManager {
    pub fn buffer_config_index(&self) -> usize {
        self.buffer_config_index
    }
    pub fn terrain_buffer_size(&self) -> usize {
        self.terrain_buffer_size
    }
    pub fn current_configuration(&self) -> &[usize; 2] {
        &self.buffer_configurations[self.buffer_config_index]
    }
    pub fn buffer_configurations(&self) -> &std::vec::Vec<[usize; 2]> {
        &self.buffer_configurations
    }
    pub fn terrain_buffers(&self) -> &std::vec::Vec<wgpu::Buffer> {
        &self.terrain_buffers
    }
    pub fn height_of_viewport(&self) -> i32 {
        self.height_of_viewport
    }
    pub fn buffer_height(&self, position_index: usize) -> i32 {
        self.buffer_heights[self.current_configuration()[position_index]]
    }
    pub fn print_state(&self) {}

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

        let mut terrain_buffers = Vec::<wgpu::Buffer>::new();
        let mut buffer_levels = Vec::<i32>::new();
        let mut buffer_heights = Vec::<i32>::new();
        let terrain_buffer_size =
            std::mem::size_of::<i32>() * (level_width * level_height) as usize;
        for _ in 0..2 {
            buffer_levels.push(-1);
            buffer_heights.push(0);
            terrain_buffers.push(LevelManager::make_terrain_buffer(
                device,
                terrain_buffer_size,
            ));
        }

        let mut lm = LevelManager {
            level_width: game_params.level_width,
            level_height: game_params.level_height,
            viewport_height: game_params.viewport_height,
            buffer_configurations,
            height_of_viewport: -1,
            buffer_heights,
            buffer_config_index: 1,
            terrain_buffer_size,
            terrain_buffers,
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
        self.level_maker
            .prefetch_up_to_level(current_top_level + 10);

        // Update the assignment of levels to buffers.
        let new_buffer_config_index = (current_bottom_level % 2) as usize;
        if new_buffer_config_index != self.buffer_config_index {
            info!("New Buffer config: {}", new_buffer_config_index);
            // New configuration: We're rearranging the buffers. Need to update all of the state.
            let mut new_buffer_levels = self.buffer_levels.clone();

            // Update the buffer index to level mapping:
            for i in 0..self.buffer_levels.len() {
                let level_number = current_bottom_level
                    + ((i + new_buffer_config_index) % self.buffer_levels.len()) as i32;
                info!("Buffer index {} has level {}", i, level_number);
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

    fn make_terrain_buffer(device: &wgpu::Device, size: usize) -> wgpu::Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            size: size as wgpu::BufferAddress,
            usage: wgpu::BufferUsage::STORAGE
                | wgpu::BufferUsage::COPY_DST
                | wgpu::BufferUsage::COPY_SRC,
            label: Some("Terrain buffer"),
            mapped_at_creation: false,
        })
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
                    &mut self.level_maker,
                    device,
                    *new,
                    &self.terrain_buffers[buffer_index],
                    self.terrain_buffer_size,
                    encoder,
                );
            }
        }
    }

    fn copy_level_to_buffer(
        level_maker: &mut LevelMaker,
        device: &wgpu::Device,
        level_num: i32,
        terrain_buffer: &wgpu::Buffer,
        terrain_buffer_size: usize,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if level_num < 0 {
            panic!("Need a positive level num. Requested: {}", level_num);
        }
        let copy_func = |level_data: &Vec<i32>| {
            let temp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Staging Buffer"),
                contents: level_data.as_bytes(),
                usage: wgpu::BufferUsage::COPY_SRC | wgpu::BufferUsage::MAP_WRITE,
            });
            encoder.copy_buffer_to_buffer(
                &temp_buf,
                0,
                terrain_buffer,
                0,
                terrain_buffer_size as u64,
            );
        };
        level_maker.use_level(level_num, copy_func)
    }
}
