pub struct LevelManager {
    // Static params
    pub level_width: u32,
    pub level_height: u32,
    pub viewport_height: u32,

    // State
    pub height_of_viewport: i32,
    pub height_of_bottom_buffer: i32,
    pub height_of_top_buffer: i32,

    pub buffer_config_a: bool,
    pub terrain_buffer_size: wgpu::BufferAddress,
    pub terrain_buffer_a: wgpu::Buffer,
    pub buffer_a_level: i32,
    pub terrain_buffer_b: wgpu::Buffer,
    pub buffer_b_level: i32,

    pub levels: std::vec::Vec<image::ImageBuffer<image::Luma<i32>, Vec<i32>>>,
}

impl LevelManager {
    pub fn init(
        device: &wgpu::Device,
        init_encoder: &mut wgpu::CommandEncoder,
        game_params: &super::game_params::GameParams,
        height_of_viewport: i32,
    ) -> Self {
        let level_width = game_params.level_width;
        let level_height = game_params.level_height;
        let (buffer_a, buffer_a_size) =
            LevelManager::make_terrain_buffer(device, level_width as usize, level_height as usize);
        let (buffer_b, buffer_b_size) =
            LevelManager::make_terrain_buffer(device, level_width as usize, level_height as usize);
        assert_eq!(buffer_a_size, buffer_b_size);

        let mut lm = LevelManager {
            level_width: game_params.level_width,
            level_height: game_params.level_height,
            viewport_height: game_params.viewport_height,
            height_of_viewport: -1,
            height_of_bottom_buffer: -1,
            height_of_top_buffer: -1,
            buffer_config_a: false,
            terrain_buffer_size: buffer_a_size,
            terrain_buffer_a: buffer_a,
            buffer_a_level: -1,
            terrain_buffer_b: buffer_b,
            buffer_b_level: -1,
            levels: vec![],
        };

        lm.sync_height(height_of_viewport);
        lm
    }

    pub fn top_buffer(&self) -> (&wgpu::Buffer, wgpu::BufferAddress) {
        (&self.terrain_buffer_b, self.terrain_buffer_size)
    }

    pub fn bottom_buffer(&self) -> (&wgpu::Buffer, wgpu::BufferAddress) {
        (&self.terrain_buffer_a, self.terrain_buffer_size)
    }

    pub fn sync_height(&mut self, height_of_viewport: i32) {
        let current_bottom_level = height_of_viewport / (self.level_height as i32);
        let current_top_level = current_bottom_level + 1;
        self.buffer_config_a = (current_bottom_level % 2) == 0;
        if self.buffer_config_a {
            self.buffer_a_level = current_bottom_level;
            self.buffer_b_level = current_top_level;
        } else {
            self.buffer_b_level = current_bottom_level;
            self.buffer_a_level = current_top_level;
        }
        self.height_of_viewport = height_of_viewport;
        self.make_levels_through(current_top_level);
    }

    fn make_level(&self, level_num: i32) -> image::ImageBuffer<image::Luma<i32>, Vec<i32>> {
        image::ImageBuffer::<image::Luma<i32>, Vec<i32>>::from_fn(
            self.level_width,
            self.level_height,
            |x, y| {
                let (index, _) = match level_num % 2 {
                    0 => (x, self.level_width),
                    1 => (y, self.level_height),
                    _ => panic!(),
                };
                match index % 5 {
                    0 => image::Luma::<i32>([1000]),
                    _ => image::Luma::<i32>([0]),
                }
            },
        )
    }

    fn make_levels_through(&mut self, level_num: i32) {
        for i in (self.levels.len() as i32)..level_num {
            self.levels.push(self.make_level(i))
        }
    }

    fn make_terrain_buffer(
        device: &wgpu::Device,
        width: usize,
        height: usize,
    ) -> (wgpu::Buffer, wgpu::BufferAddress) {
        let size = (std::mem::size_of::<i32>() * width * height) as wgpu::BufferAddress;
        (
            device.create_buffer(&wgpu::BufferDescriptor {
                size,
                usage: wgpu::BufferUsage::STORAGE
                    | wgpu::BufferUsage::COPY_DST
                    | wgpu::BufferUsage::COPY_SRC
                    | wgpu::BufferUsage::STORAGE_READ,
            }),
            size,
        )
    }

    fn sync_buffer(&mut self) {
        //
    }
}
