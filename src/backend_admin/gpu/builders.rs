use super::enums::*;
use wgpu::*;
/// Specificies BindGroupLayout configuration through chaining
/// build() consumes self 
pub struct BindGroupLayoutBuilder {
    label: Option<String>,
    entries: Vec<wgpu::BindGroupLayoutEntry>
}

impl BindGroupLayoutBuilder {
    pub fn new(label: String) ->  Self {
        BindGroupLayoutBuilder {
            label: Some(label),
            entries: Vec::new()
        }
    }
    pub fn with_storage_buffer(mut self, 
        visibility: ShaderStages, 
        offset_behaviour: OffsetBehaviour,
        access: Access ) -> Self {
            
            let (is_dynamic, read_only) = match (offset_behaviour, access) {
                (OffsetBehaviour::Dynamic, Access::ReadOnly) => {
                    (true, true)
                },
                (OffsetBehaviour::Dynamic, Access::ReadWrite) => {
                    (true, false)
                },
                (OffsetBehaviour::Static, Access::ReadOnly) => {
                    (false, true)
                },
                (OffsetBehaviour::Static, Access::ReadWrite) => {
                    (false, false)
                }
            };

            let e = BindGroupLayoutEntry {
                binding: self.entries.len() as u32,
                visibility: visibility,
                ty: wgpu::BindingType::Buffer { 
                    ty: BufferBindingType::Storage { 
                    read_only: read_only }, 
                    has_dynamic_offset: is_dynamic, 
                    min_binding_size: None},
                count: None
                };

            self.entries.push(e);
            self
    }

    pub fn with_uniform_buffer(mut self,
        visibility: ShaderStages,
        offset_behaviour: OffsetBehaviour) -> Self {

            let is_dynamic = match offset_behaviour {
                OffsetBehaviour::Dynamic => {true},
                OffsetBehaviour::Static => {false}
            };

            let e = BindGroupLayoutEntry { 
                    binding: self.entries.len() as u32,
                    visibility: visibility,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: is_dynamic,
                        min_binding_size: None,
                    },
                    count: None
            };
            self.entries.push(e);
            self
        }

    pub fn with_storage_texture(mut self,
        visibility: ShaderStages,
        format: TextureFormat,
        access: StorageTextureAccess,
        dimensions: TextureViewDimension) -> Self {

            let e = BindGroupLayoutEntry {
                binding: self.entries.len() as u32,
                visibility: visibility,
                ty: wgpu::BindingType::StorageTexture { 
                    access: access, 
                    format: format, 
                    view_dimension: dimensions },
                count: None
            };

            self.entries.push(e);
            self
        }

    pub fn build(self, device: &Device) -> BindGroupLayout {

        device.create_bind_group_layout(&BindGroupLayoutDescriptor{
            label: self.label.as_deref(),
            entries: &self.entries
        })
    }
}
