use std::{path, sync::Arc};
use winit::window::Window;
use wgpu::{Adapter, Device, Instance, Queue, ShaderModule, Surface, SurfaceConfiguration};
use anyhow::Result;

pub struct GraphicsContext {
    window: Arc<Window>,
    instance: Option<Instance>,
    adapter: Option<Adapter>,
    surface: Option<Surface<'static>>,
    pub device: Option<Device>,
    queue: Option<Queue>,
    surface_config: Option<SurfaceConfiguration>,
    surface_configured: bool
}

impl GraphicsContext {
    pub async fn new(win: Arc<Window>) -> Result<Self> {
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
                instance: Some(instance),
                adapter: Some(adapter),
                surface: Some(surface),
                device: Some(device),
                queue: Some(queue),
                surface_config: Some(surface_config),
                surface_configured: true
            }
        )
    }


}