use crate::{backend_admin::gpu::gfx_context::GraphicsContext, 
    world::{
        voxel_grid::VoxelDims,
        camera::OrbitalCamera
    }};
use wgpu::BufferUsages;
use wgpu::util::DeviceExt;
use wgpu::Extent3d;
use winit::dpi::PhysicalSize;


pub struct Resources {
    sampler:,
    ping_voxel_buffer:,
    pong_voxel_buffer:,
    storage_texture:,
    texture_view:,
    uniforms: ,
    voxel_dims: VoxelDims,
    camera: OrbitalCamera
}

impl Resources {
    pub fn new(dims: VoxelDims, gfx_context: GraphicsContext) -> Result<Self> {
        let (surface_conf,device) = match & ( gfx_context.surface_config, gfx_context.device) {
            (Some(s),Some(d)) => (s, d),
            (Some(s), None) => return None,
            (None, Some(s)) => return None,
            (None, None) => return None
        };

        let size = if gfx_context.surface_configured {
             PhysicalSize::new(surface_conf.width, surface_conf.height) }
             else { gfx_context.update_surface_config()? };

        let ping_voxels = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute store a"),
            size:  (std::mem::size_of::<f32>() as u32 * dims.i * dims.j * dims.k) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false 
        });

        let pong_voxels = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Compute store b"),
            size:  (std::mem::size_of::<f32>() as u32 * dims.i * dims.j * dims.k) as u64,
            usage: BufferUsages::STORAGE,
            mapped_at_creation: false
        }); 

        let camera = OrbitalCamera::new((dims.i as f32) * 2, 0.0, 0.0, &size);

        let mut rng = rand::rng();

        let uniforms = Uniforms {
            window_dims: [size.width/2, size.height/2, 0, 0],
            dims: [dims.i as u32, dims.j as u32, dims.k as u32, (dims.i * dims.j) as u32],
            bounding_box: [0, 0, 0, 0], // set in render() 
            cam_pos: [camera.c[0], camera.c[1], camera.c[2], 0.0 as f32],
            forward: [camera.f[0], camera.f[1], camera.f[2], 0.0 as f32],
            centre: [camera.centre[0], camera.centre[1], camera.centre[2], 0.0 as f32],
            up: [camera.u[0], camera.u[1], camera.u[2], 0.0 as f32],
            right: [camera.r[0], camera.r[1], camera.r[2], 0.0 as f32],
            timestep: [0.0 as f32, 0.0 as f32, 0.0 as f32, 0.0 as f32],
            seed: [rng.random::<f32>(), 0.0 as f32, 0.0 as f32, 0.0 as f32],
            flags: [1, 0, 0, 0]
        };
        
        let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform buffer"),
            contents: uniforms.flatten_u8(),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let storage_texture = device.create_texture(&TextureDescriptor{
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



    }

}

#[repr(C)]
#[derive(Clone, Copy)]
struct Uniforms {
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
    seed: [f32; 4], // only [0]
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