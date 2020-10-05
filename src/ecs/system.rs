//! entity component Systems

use std::time::Duration;

use crate::debug::DebugContext;
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, ComponentStorage, Entity, ReadAccess, RwAccess};
use crate::game::IoState;
use crate::gfx::RenderContext;

pub mod physics;
#[cfg(feature = "3d")]
pub mod scene;

pub struct Read<'a, C> {
    reader: Box<dyn ReadAccess<'a, C> + 'a>,
}

impl<'a, C: Component> Read<'a, C> {
    pub fn new(reader: Box<dyn ReadAccess<'a, C> + 'a>) -> Read<'a, C> {
        Read { reader }
    }
}

impl<'access, C> Read<'access, C>
where
    C: Component,
{
    pub fn iter<'borrow, 's: 'borrow>(&'s self) -> impl Iterator<Item = (Entity, &'borrow C)> {
        self.reader.iter()
    }

    pub fn fetch(&self, entity: Entity) -> Option<&C> {
        self.reader.fetch(entity)
    }
}

pub struct Write<'a, C> {
    writer: Box<dyn RwAccess<'a, C> + 'a>,
}

impl<'a, C: Component> Write<'a, C> {
    pub fn new(writer: Box<dyn RwAccess<'a, C> + 'a>) -> Write<'a, C> {
        Write { writer }
    }
}

impl<'access, C> Write<'access, C>
where
    C: Component,
{
    pub fn iter<'borrow, 's: 'borrow>(&'s self) -> impl Iterator<Item = (Entity, &'borrow C)> {
        self.writer.iter()
    }

    pub fn fetch(&self, entity: Entity) -> Option<&C> {
        self.writer.fetch(entity)
    }

    /// sets value of Component for Entity
    pub fn set(&mut self, entity: Entity, value: C) {
        self.writer.set(entity, value)
    }

    /// unsets value of Component for Entity
    pub fn unset(&mut self, entity: Entity) {
        self.writer.unset(entity)
    }

    /// mutable iterator over component storage
    pub fn iter_mut<'borrow, 's: 'borrow>(
        &'s mut self,
    ) -> Box<dyn Iterator<Item = (Entity, &'borrow mut C)> + 'borrow> {
        self.writer.iter_mut()
    }

    /// clears this Component storage, unsetting the value for each Entity
    fn clear(&mut self) {
        self.writer.clear()
    }
}

pub trait SystemData<'access, W: World> {
    fn get(world: &'access W) -> Self;
}

impl<'a, W: World> SystemData<'a, W> for () {
    fn get(world: &'a W) -> Self {
        ()
    }
}

impl<'access, W, C> SystemData<'access, W> for Write<'access, C>
where
    C: 'access + Component,
    W: World + WorldStorage<C>,
{
    fn get(world: &'access W) -> Self {
        Write::new(Box::new(world.storage().write()))
    }
}

impl<'access, W, C> SystemData<'access, W> for Read<'access, C>
where
    C: 'access + Component,
    W: World + WorldStorage<C>,
{
    fn get(world: &'access W) -> Self {
        Read::new(Box::new(world.storage().read()))
    }
}

impl<'a, A, W> SystemData<'a, W> for (A,)
where
    A: SystemData<'a, W>,
    W: World,
{
    fn get(world: &'a W) -> Self {
        (A::get(world),)
    }
}

impl<'a, A, B, W> SystemData<'a, W> for (A, B)
where
    A: SystemData<'a, W>,
    B: SystemData<'a, W>,
    W: World,
{
    fn get(world: &'a W) -> Self {
        (A::get(world), B::get(world))
    }
}

impl<'a, A, B, C, W> SystemData<'a, W> for (A, B, C)
where
    A: SystemData<'a, W>,
    B: SystemData<'a, W>,
    C: SystemData<'a, W>,
    W: World,
{
    fn get(world: &'a W) -> Self {
        (A::get(world), B::get(world), C::get(world))
    }
}

impl<'a, A, B, C, D, W> SystemData<'a, W> for (A, B, C, D)
where
    A: SystemData<'a, W>,
    B: SystemData<'a, W>,
    C: SystemData<'a, W>,
    D: SystemData<'a, W>,
    W: World,
{
    fn get(world: &'a W) -> Self {
        (A::get(world), B::get(world), C::get(world), D::get(world))
    }
}

pub trait System<W: World> {
    type SystemData<'a>: SystemData<'a, W>;

    fn name(&self) -> &'static str;
    fn update<'f>(
        &mut self,
        data: Self::SystemData<'f>,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext, // TODO: fix profiler
        //        profiler_tag: profiler::OpenTagTree<'f>
        debug_ctx: &mut DebugContext,
    ) -> ();

    fn draw(&self, _render_ctx: &mut RenderContext) {}
    fn draw_to(&self, _view: &[&wgpu::TextureView], _render_ctx: &mut RenderContext) {}
}

pub trait SystemCaller<W: World> {
    fn dispatch<'a, 's>(
        &'s mut self,
        world: &'a W,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> ();

    fn render<'a, 's>(&'s self, render_ctx: &mut RenderContext) -> ();
}

impl<W: World, S> SystemCaller<W> for S
where
    S: System<W>,
{
    fn dispatch<'a, 's>(
        &'s mut self,
        world: &'a W,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> () {
        self.update(
            <<S as System<W>>::SystemData<'a> as SystemData<'a, W>>::get(world),
            delta,
            io_state,
            render_ctx,
            debug_ctx,
        )
    }

    fn render<'a, 's>(&'s self, render_ctx: &mut RenderContext) -> () {
        self.draw(render_ctx)
    }
}
