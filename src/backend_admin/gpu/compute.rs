use wgpu::{BindGroup, PipelineCompilationOptions, BindGroupEntry, BindGroupLayout, BufferBinding, ComputePipeline, PipelineLayout, ShaderModule, ShaderStages, TextureFormat};
use std::num::NonZero;
use crate::{world::voxel_grid::Dims3, backend_admin::gpu::{
    enums::{Access, OffsetBehaviour}, 
    builders::BindGroupLayoutBuilder,
    gfx_context::GraphicsContext,
    resources::{Uniforms, Resources}}};



/// Responsible for Compute pipeline, including
/// init, raymarch and laplacian
pub struct Compute{
    init_shader: ShaderModule,
    laplacian_shader: ShaderModule,
    raymarch_shader: ShaderModule,

    bg_layout: BindGroupLayout,
    pub bg: BindGroup,

    p_layout: PipelineLayout,
    pub init_p: ComputePipeline,
    pub laplacian_p: ComputePipeline,
    pub raymarch_p: ComputePipeline
    
}

impl Compute {
    pub fn new(dims: &Dims3, resources: &Resources, gfx_ctx: &GraphicsContext) -> Self {
        // Load shader module
        let init = gfx_ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Init"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/init.wgsl").into())
            });
        let laplacian = gfx_ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Laplacian"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/laplacian.wgsl").into())
            });
        let raymarch = gfx_ctx.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raymarch"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/raymarch.wgsl").into())
            });

         // CONFIGURE BIND GROUP LAYOUT USING BUILDER //
        let bind_group_layout = BindGroupLayoutBuilder::new("Compute Bind Group".to_string())
            .with_uniform_buffer(
                ShaderStages::COMPUTE, 
                OffsetBehaviour::Static)
            .with_storage_buffer(
                ShaderStages::COMPUTE, 
                OffsetBehaviour::Static, 
                Access::ReadWrite)
            .with_storage_buffer(
                ShaderStages::COMPUTE,
                OffsetBehaviour::Static,
                Access::ReadWrite)
            .with_storage_texture(
                ShaderStages::COMPUTE, 
                TextureFormat::Rgba8Unorm, 
                wgpu::StorageTextureAccess::WriteOnly,
            wgpu::TextureViewDimension::D2)
            .build(&gfx_ctx.device);

        let bind_group_descriptor = &wgpu::BindGroupDescriptor { //TODO: CONSIDER MAKING A BUILD GROUP DESCRIPTOR BUILDER
        label: Some("Bind group descriptor"),
        layout: &bind_group_layout,
        entries: &[
        BindGroupEntry {
            binding: 0,
            resource: wgpu::BindingResource::Buffer(BufferBinding { 
                buffer: &resources.uniforms, 
                offset: 0, 
                size: NonZero::new((std::mem::size_of::<Uniforms>()) as u64)
            }),
        },
        BindGroupEntry {
            binding: 1,
            resource: wgpu::BindingResource::Buffer(BufferBinding{ 
                buffer:  &resources.ping_voxel_buffer,
                offset: 0,
                size: NonZero::new((std::mem::size_of::<f32>() as u32 * dims[0] * dims[1] * dims[2]) as u64)
        })
        },
        BindGroupEntry {
            binding: 2,
            resource: wgpu::BindingResource::Buffer(BufferBinding{ 
                buffer:  &resources.pong_voxel_buffer,
                offset: 0,
                size: NonZero::new((std::mem::size_of::<f32>() as u32 * dims[0] * dims[1] * dims[2]) as u64)
        })},
        BindGroupEntry {
            binding: 3,
            resource: wgpu::BindingResource::TextureView(&resources.texture_view)}
        ]
        };

        let bind_group = gfx_ctx.device.create_bind_group(bind_group_descriptor); 

         // COMPUTE PIPELINE SETUP //
        let pipeline_layout = gfx_ctx.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Compute Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout
            ],
            push_constant_ranges: &[]
        });

        // Pipelines

        // Entry Points
        let init_pipeline = gfx_ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Init"),
            layout: Some(&pipeline_layout),
            module: &init,
            entry_point: Some("init"),
            cache: None,
            compilation_options: PipelineCompilationOptions{
                constants: &[],
                zero_initialize_workgroup_memory: true
            }
        });

        let laplacian_pipeline = gfx_ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Laplacian"),   
            layout: Some(&pipeline_layout),
            module: &init,
            entry_point: Some("laplacian"),
            cache: None,
            compilation_options: PipelineCompilationOptions{
                constants: &[],
                zero_initialize_workgroup_memory: true 
            }
        });
         
        let raymarch_pipeline = gfx_ctx.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Raymarch"),
            layout: Some(&pipeline_layout),
            module: &init,
            entry_point: Some("raymarch"),
            cache: None,
            compilation_options: PipelineCompilationOptions{
                constants: &[],
                zero_initialize_workgroup_memory: true 
            }
        });

            Compute {
                init_shader: init,
                laplacian_shader: laplacian,
                raymarch_shader: raymarch,

                bg_layout: bind_group_layout,
                bg: bind_group,

                p_layout: pipeline_layout,
                init_p: init_pipeline,
                laplacian_p: laplacian_pipeline,
                raymarch_p: raymarch_pipeline
            }

    }

    pub fn on_resize(&mut self, dims: &Dims3, gfx_ctx: &GraphicsContext, rsrcs: &Resources) {
        let bind_group_descriptor = &wgpu::BindGroupDescriptor {
            label: Some("Bind group descriptor"),
            layout: &self.bg_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(BufferBinding { 
                    buffer: &rsrcs.uniforms, 
                    offset: 0, 
                    size: NonZero::new((std::mem::size_of::<Uniforms>()) as u64)
                }),
            },
            BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  &rsrcs.ping_voxel_buffer,
                    offset: 0,
                    size: NonZero::new((std::mem::size_of::<f32>() as u32 * dims[0] * dims[1] * dims[2]) as u64)
            })
            },
            BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(BufferBinding{ // actual voxel grid storage buffer @ binding 1
                    buffer:  &rsrcs.pong_voxel_buffer,
                    offset: 0,
                    size: NonZero::new((std::mem::size_of::<f32>() as u32 * dims[0] * dims[1] * dims[2]) as u64)
            })
            },
            BindGroupEntry {
                binding: 3,
                resource: wgpu::BindingResource::TextureView(&rsrcs.texture_view)
            }
            ]
        };
        self.bg = gfx_ctx.device.create_bind_group(bind_group_descriptor);
    }

}

