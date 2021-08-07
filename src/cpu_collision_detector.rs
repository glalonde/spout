struct CpuCollisionDetector {
    staging_texture: wgpu::Buffer,
}

impl CpuCollisionDetector {
    // This needs to be in the rendering loop: Each frame it stages one of the terrain buffers into CPU visible memory.
    // This can then be used for CPU-local collision detection.
    fn check(&mut self, device: &wgpu::Device, level_manager: &super::level_manager::LevelManager) {
    }

    fn request_staging_buffer(
        &mut self,
        device: &wgpu::Device,
        ship_height: i32,
        level_manager: &super::level_manager::LevelManager,
    ) {
        // Get a buffer handle that corresponds to height
        
    }
}
