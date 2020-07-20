//! entity component Systems

use std::time::Duration;

use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, ComponentStorage, Entity, Event, ReadAccess, WriteAccess};
use crate::game::IoState;
use crate::gfx::RenderContext;

pub mod physics;

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

pub trait SystemData<'access, W: World> {
    fn get(world: &'access W) -> Self;
}

// impl<'read, 'data: 'read, W, C, CS, R> SystemData<'data, W> for Read<'read, C>
// where
//     C: Component,
//     W: World + WorldStorage<C, Storage = CS>,
//     CS: ComponentStorage<C, Reader = R>,
//     R: ReadAccess<'read, C>,
// {
//     fn get(world: &W) -> Read<'read, C> {
//         let component_storage: CS = world.storage();
//         let reader: R = component_storage.read();
//         Read::new(reader)
//     }
// }

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
    ) -> ();

    fn draw(&self, render_ctx: &mut RenderContext) {}
    fn draw_to(&self, view: &[&wgpu::TextureView], render_ctx: &mut RenderContext) {}
}

pub trait SystemCaller<W: World> {
    fn dispatch<'a, 's>(
        &'s mut self,
        world: &'a W,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
    ) -> ();
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
    ) -> () {
        self.update(
            <<S as System<W>>::SystemData<'a> as SystemData<'a, W>>::get(world),
            delta,
            io_state,
            render_ctx,
        )
    }
}
