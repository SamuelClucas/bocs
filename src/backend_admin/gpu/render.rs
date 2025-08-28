use wgpu::{BindGroup, BindGroupEntry, BindGroupLayout, PipelineLayout, PipelineLayoutDescriptor, RenderPipeline, ShaderModule, ShaderModuleDescriptor, ShaderStages};
use crate::{backend_admin::gpu::{
    builders::BindGroupLayoutBuilder,
    gfx_context::GraphicsContext,
    resources::Resources}
};

pub struct Render{
    frag_shader: ShaderModule,
    vert_shader: ShaderModule,

    bg_layout: BindGroupLayout,
    pub bg: BindGroup,

    p_layout: PipelineLayout,
    pub p: RenderPipeline,
}

impl Render {
    pub fn new(resources: &Resources, gfx_ctx: &GraphicsContext) -> Self {
        // Load shader modules //
        let f_module = gfx_ctx.device.create_shader_module(ShaderModuleDescriptor{
            label: Some("Fragment shader module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/fragment.wgsl").into())
            });

        let v_module = gfx_ctx.device.create_shader_module(
            ShaderModuleDescriptor { 
                label: Some("Vertex shader module"), 
                source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/vertex.wgsl").into()) 
            });
        

        let bind_group_layout = BindGroupLayoutBuilder::new("Render Bind Group".to_string())
                .with_sampler(ShaderStages::FRAGMENT)
                .with_sampled_texture(ShaderStages::FRAGMENT)
                .build(&gfx_ctx.device);
        
        let bind_group = gfx_ctx.device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Render Bind Group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&resources.sampler)},

                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&resources.texture_view)
                }
            ]
        });

        let pipeline_layout = gfx_ctx.device.create_pipeline_layout(&PipelineLayoutDescriptor{
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[]
        });

        let pipeline = gfx_ctx.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor { 
            label: Some("Render Pipeline"), 
            layout: Some(&pipeline_layout), 
            vertex: wgpu::VertexState{
                module: &v_module,
                entry_point: Some("main"), 
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[]
            }, 
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, 
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, 
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, 
            multisample: wgpu::MultisampleState {
                    count: 1, 
                    mask: !0, 
                    alpha_to_coverage_enabled: false, 
                }, 
            fragment: Some(wgpu::FragmentState { // needed to store colour data to the surface
               module: &f_module,
               entry_point: Some("main"),
               compilation_options: wgpu::PipelineCompilationOptions::default(),
               targets: &[Some(wgpu::ColorTargetState {
                    format: gfx_ctx.surface_config.format, // format of surface
                    blend: Some(wgpu::BlendState::REPLACE), // replace old colour with new colour
                    write_mask: wgpu::ColorWrites::ALL // write to all channels
               })]
            }), 
            multiview: None, 
            cache: None, 
        });

            Render {
                frag_shader: f_module,
                vert_shader: v_module,

                bg_layout: bind_group_layout,
                bg: bind_group,

                p_layout: pipeline_layout,
                p: pipeline
            }
    }

    pub fn on_resize(&mut self, gfx_ctx: &GraphicsContext, rsrcs: &Resources){
        self.bg = gfx_ctx.device.create_bind_group(&wgpu::BindGroupDescriptor{
            label: Some("Render Bind Group"),
            layout: &self.bg_layout,
            entries: &[BindGroupEntry{
                binding: 0,
                resource: wgpu::BindingResource::Sampler(&rsrcs.sampler)},

                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&rsrcs.texture_view)
                }]
            });
    }
}

