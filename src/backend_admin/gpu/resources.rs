use crate::{backend_admin::{
    bridge::Bridge, gpu::gfx_context::GraphicsContext},
    world::{voxel_grid::Dims3, world::{BoundingBox, World}
    }};
use wgpu::{Buffer, BufferUsages, Extent3d, Sampler, Texture, TextureDescriptor, TextureUsages, TextureView, TextureViewDescriptor};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;


pub struct Resources {
    pub sampler: Sampler,
    pub ping_voxel_buffer: Buffer,
    pub pong_voxel_buffer: Buffer,
    storage_texture: Texture,
    pub texture_view: TextureView,
    pub uniforms: Buffer
}

impl Resources {
    pub fn new(dims: &Dims3, world: &World, bridge: &Bridge, gfx_ctx: &mut GraphicsContext) -> Self {

        let size = if gfx_ctx.surface_configured {
             PhysicalSize::new(gfx_ctx.surface_config.width, gfx_ctx.surface_config.height) }
             else { gfx_ctx.update_surface_config() };

        let ping_voxels = gfx_ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute store a"),
            size:  (std::mem::size_of::<f32>() as u32 * dims[0] * dims[1] * dims[2]) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false 
        });

        let pong_voxels = gfx_ctx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute store b"),
            size:  (std::mem::size_of::<f32>() as u32 * dims[0] * dims[1] * dims[2]) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false
        }); 


        let uniforms = Uniforms {
            window_dims: [size.width/2, size.height/2, 0, 0],
            dims: [dims[0], dims[1], dims[2], dims[0] * dims[1]],
            bounding_box: [0, 0, 0, 0], // set in render() 
            cam_pos: [world.camera.c[0], world.camera.c[1], world.camera.c[2], 0.0 as f32],
            forward: [world.camera.f[0], world.camera.f[1], world.camera.f[2], 0.0 as f32],
            centre: [world.camera.centre[0], world.camera.centre[1], world.camera.centre[2], 0.0 as f32],
            up: [world.camera.u[0], world.camera.u[1], world.camera.u[2], 0.0 as f32],
            right: [world.camera.r[0], world.camera.r[1], world.camera.r[2], 0.0 as f32],
            timestep: [0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32],
            seed: [bridge.rand_seed, 0, 0, 0],
            flags: [1, 0, 0, 0]
        };
        
        let uniforms = gfx_ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: uniforms.flatten_u8(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let storage_texture = gfx_ctx.device.create_texture(&TextureDescriptor{
            label: Some("Storage Texture"),
            size: Extent3d {
                width: size.width, 
                height: size.height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm]
        });

        let texture_view = storage_texture.create_view(&TextureViewDescriptor{
            label: Some("Texture View"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count:None
        });

        let sampler = gfx_ctx.device.create_sampler(&wgpu::SamplerDescriptor{
            label: Some("Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None
        });


        Resources {
            sampler: sampler,
            ping_voxel_buffer: ping_voxels,
            pong_voxel_buffer: pong_voxels,
            storage_texture: storage_texture,
            texture_view: texture_view,
            uniforms: uniforms
        }

    }

    pub fn on_resize(&mut self, dims: &Dims3, width: u32, height: u32, gfx_ctx: &GraphicsContext, world: &World, bridge: &Bridge){

        self.storage_texture = gfx_ctx.device.create_texture(&TextureDescriptor{
            label: Some("Storage Texture"),
            size: Extent3d {
                width: width, 
                height: height,
                depth_or_array_layers: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            view_formats: &[wgpu::TextureFormat::Rgba8Unorm]
        });

        self.texture_view = self.storage_texture.create_view(&TextureViewDescriptor{
            label: Some("Texture View"),
            format: Some(wgpu::TextureFormat::Rgba8Unorm),
            dimension: Some(wgpu::TextureViewDimension::D2),
            usage: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count:None
        });

        let unis = Uniforms {
            window_dims: [width/2, height/2, 0, 0], // could update these via command encoder
            dims: [dims[0], dims[1], dims[2], dims[0] * dims[1]],
            bounding_box: [0, 0, 0, 0], // set in render() 
            cam_pos: [world.camera.c[0], world.camera.c[1], world.camera.c[2], 0.0 as f32],
            forward: [world.camera.f[0], world.camera.f[1], world.camera.f[2], 0.0 as f32],
            centre: [world.camera.centre[0], world.camera.centre[1], world.camera.centre[2], 0.0 as f32],
            up: [world.camera.u[0], world.camera.u[1], world.camera.u[2], 0.0 as f32],
            right: [world.camera.r[0], world.camera.r[1], world.camera.r[2], 0.0 as f32],
            timestep: [0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32],
            seed: [bridge.rand_seed, 0, 0, 0],
            flags: [1, 0, 0, 0]
        };

        self.uniforms = gfx_ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: unis.flatten_u8(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        

    }

    pub fn uniforms_refresh(&mut self, 
        gfx_ctx: &GraphicsContext, read_ping: &bool, 
        duration: f32, bbox: BoundingBox, dims: &Dims3, 
        world: &World) {
        if gfx_ctx.surface_configured == true {

            let uniforms = Uniforms {
                window_dims: [gfx_ctx.surface_config.width/2, gfx_ctx.surface_config.height/2, 0, 0],
                dims: [dims[0], dims[1], dims[2], dims[0] * dims[1]],
                bounding_box: [bbox[0][0], bbox[0][1], bbox[1][0], bbox[1][1]],
                cam_pos: [world.camera.c[0], world.camera.c[1], world.camera.c[2], 0.0],
                forward: [world.camera.f[0], world.camera.f[1], world.camera.f[2], 0.0],
                centre: [world.camera.centre[0], world.camera.centre[1], world.camera.centre[2], 0.0],
                up: [world.camera.u[0], world.camera.u[1], world.camera.u[2], 0.0 as f32],
                right: [world.camera.r[0], world.camera.r[1], world.camera.r[2], world.right_sf],
                timestep: [duration, 0.0, 0.0, 0.0],
                seed: [0, 0, 0, 0 ], // could later reintroduce seed here for hot sim resizing 
                flags: [*read_ping as u32, 0, 0, 0]
            };

            let data = uniforms.flatten_u8();

            self.uniforms = gfx_ctx.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: data,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            gfx_ctx.queue.write_buffer(&self.uniforms, 0, data);
        }
        else { panic!("Tried to update uniforms with outdated graphics context\n") }

    }

    }



#[repr(C)]
#[derive(Clone, Copy)]
pub struct Uniforms {
    /// World -> Camera basis vectors, timestep, and random seed for voxel grid init
    /// Wgsl expects Vec4<f32> (16 byte alignment
    window_dims: [u32; 4],
    dims: [u32; 4], // i, j, k, ij plane stride for k
    bounding_box: [i32; 4],
    cam_pos: [f32; 4], // [2]< padding
    forward: [f32; 4], // [2]< padding
    centre: [f32; 4],
    up: [f32; 4], // [2]< padding
    right: [f32; 4], // [2]< padding
    timestep: [f32; 4], // only [0]
    seed: [u32; 4], // only [0]
    flags: [u32; 4]

}

impl Uniforms {
    pub fn flatten_u8(&self) -> &[u8] {
        let ptr = self as *const _ as *const u8;

        let len = std::mem::size_of::<Uniforms>(); 
        // Each f32/u32 padded rhs with 12 bytes to 16 byte alignment
        
        unsafe {
            std::slice::from_raw_parts(ptr, len)
        }
    }
}