use std::{sync::Arc};
use winit::{
    dpi::PhysicalSize,
    window::Window};
use wgpu::{Adapter, Device, Instance, Queue, Surface, SurfaceConfiguration, SurfaceTexture, TextureView};
use anyhow::Result;
use std::error::Error;

pub struct GraphicsContext {
    pub window: Arc<Window>,
    pub size: PhysicalSize<u32>,
    instance: Instance,
    adapter: Adapter,

    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub surface_configured: bool,

    pub device: Device,
    pub queue: Queue,
    
}

impl GraphicsContext {
    pub async fn new(win: Arc<Window>) -> Result<Self, Box<dyn Error>> {
        // Instance == handle to GPU
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // Surface == handle to window (GPU output)
        let surface = instance.create_surface(win.clone())?; // clone here otherwise surface takes ownership of window. Clone on arc is very cheap.

        let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false
        }).await?;

        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor{
            label: None,
            required_features: wgpu::Features::default(), 
            required_limits: wgpu::Limits::defaults(),
            trace: wgpu::Trace::Off,
            memory_hints: Default::default(),
        }).await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = win.inner_size();
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        Ok (
            GraphicsContext {
                window: win,
                size: size,
                instance: instance,
                adapter: adapter,

                surface: surface,
                surface_config: surface_config,
                surface_configured: true,

                device: device,
                queue: queue,
            }
        )
    }

    pub fn update_surface_config(&mut self) -> Result<PhysicalSize<u32>, Box<dyn Error>> {
        let surface_caps = self.surface.get_capabilities(&self.adapter);

        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let size = self.window.inner_size();

        self.surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        self.surface.configure(&self.device, &self.surface_config);

        
        self.surface_configured = true;
        Ok(
            size
        )

    }

}