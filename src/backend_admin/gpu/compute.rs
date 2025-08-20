use wgpu::ShaderModule;
use crate::backend_admin::gpu::gfx_context::GraphicsContext;
use anyhow::Result;

pub struct Compute{
    init: ShaderModule,
    laplacian: ShaderModule,
    raymarch: ShaderModule
    
}

impl Compute {
    pub fn new(ctx: &GraphicsContext) -> Option<Self> {
        let device = match & ctx.device {
            Some(d) => d,
            None => return None 
        };

        let init = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Init"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/init.wgsl").into())
            });
        let laplacian = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Laplacian"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/laplacian.wgsl").into())
            });
        let raymarch = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Raymarch"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../shaders/raymarch.wgsl").into())
            });
        
        Some(
            Compute {
                init: init,
                laplacian: laplacian,
                raymarch: raymarch
            }
        )
    }

}

