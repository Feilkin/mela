//! entity component Systems

use std::time::Duration;

use crate::ecs::world::World;
use crate::game::IoState;
use crate::gfx::RenderContext;

pub trait System<W: World> {
    fn name(&self) -> &'static str;
    fn update<'f>(
        &mut self,
        delta: Duration,
        world: W,
        io_state: &IoState,
        render_ctx: &mut RenderContext, // TODO: fix profiler
                                        //        profiler_tag: profiler::OpenTagTree<'f>
    ) -> W;

    fn draw(&self, render_ctx: &mut RenderContext) {}
    fn draw_to(&self, view: &[&wgpu::TextureView], render_ctx: &mut RenderContext) {}
}
