use crate::backend_admin::gpu::{gfx_context::GraphicsContext, resources::Resources};

pub trait Update<I, O, T> {
    fn on_resize(&mut self, a: &I, b: &O, c: &T);
}

impl Update<I, O, T> for GraphicsContext {
    fn on_resize(&mut self, a: &I, b: &O, c: &T) {
        
    }
}