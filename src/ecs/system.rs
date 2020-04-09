//! entity component Systems

use std::time::Duration;

use crate::ecs::world::World;
use crate::profiler;
use crate::profiler::{OpenTagTree, Profiler};

pub trait System<W: World> {
    fn name(&self) -> &'static str;
    fn update<'f>(
        &mut self,
        delta: Duration,
        world: W,
        profiler_tag: profiler::OpenTagTree<'f>,
    ) -> (W, OpenTagTree<'f>);
}
