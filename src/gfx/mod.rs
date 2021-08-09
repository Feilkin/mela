//! Graphics stuff

use std::mem;
use std::rc::Rc;

//pub mod primitives;

/// Type alias over reference counted wgpu texture
pub type Texture = Rc<wgpu::Texture>;

pub trait MiddlewareRenderer {
    // TODO: wtf is "screen" size?
    fn new(
        device: &wgpu::Device,
        texture_format: &wgpu::TextureFormat,
        screen_size: [f32; 2],
    ) -> Self
    where
        Self: Sized;

    // Prepare for rendering, create all resources used during render, storing render data internally
    fn prepare(
        &mut self,
        world: &mut crate::ecs::World,
        device: &wgpu::Device,
        staging_belt: &mut wgpu::util::StagingBelt,
        command_encoder: &mut wgpu::CommandEncoder,
    );

    // Render using internal data and user provided render pass
    fn render<'rpass>(&'rpass mut self, render_pass: &mut wgpu::RenderPass<'rpass>);
}
