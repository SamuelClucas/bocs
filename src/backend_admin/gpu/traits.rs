use crate::backend_admin::gpu::{gfx_context::GraphicsContext, resources::Resources};

pub trait Pipeline {
    fn on_resize(&mut self, rscs: &Resources, ctx: &GraphicsContext);

    fn create_bind_group_layout(&mut self, rscs: &Resources, ctx: &GraphicsContext);

    fn create_bind_group(&mut self, rscs: &Resources, ctx: &GraphicsContext);

    fn add_shader(&mut self, rscs: &Resources, ctx: &GraphicsContext);

}