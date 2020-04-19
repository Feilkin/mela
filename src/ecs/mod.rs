//! my own entity component system

use std::collections::VecDeque;
use std::fmt::Debug;
use std::iter::{Enumerate, FilterMap};
use std::net::Shutdown::Write;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;

pub use entity::Entity;
pub use system::System;

use crate::ecs::entity::EntityBuilder;
use crate::ecs::world::World;

pub mod entity;
pub mod system;
pub mod world;

/// An interface for component storages. See `VecStorage` for example implementation
pub trait ComponentStorage<C: Component> {
    type Reader<'r>: ReadAccess<'r, C>;
    type Writer<'w>: WriteAccess<'w, C>;

    /// returns slice of components, indexed by entity
    fn read<'r, 'd: 'r>(&'d self) -> Self::Reader<'r>;

    /// writes new component value for entity
    fn write<'w, 'd: 'w>(&'d mut self) -> Self::Writer<'w>;
}

/// An interface for Component. Doesn't actually do anything yet, other than make sure our components are sized, and shareable across threads
pub trait Component: Sized + Send + Sync {}

/// An interface that describes read access to a Component
pub trait ReadAccess<'r, C: Component> {
    fn fetch(&self, entity: Entity) -> Option<&C>;
    fn iter<'a>(&'r self) -> Box<dyn Iterator<Item = (Entity, &'a C)> + 'a>
    where
        'r: 'a;
}

/// An interface that describes write access to a Component
pub trait WriteAccess<'w, C: Component> {
    /// sets value of Component for Entity
    fn set(&mut self, entity: Entity, value: C);

    /// unsets value of Component for Entity
    fn unset(&mut self, entity: Entity);

    /// mutable iterator over component storage
    fn iter_mut<'a>(&'w mut self) -> Box<dyn Iterator<Item = (Entity, &'a mut C)> + 'a>
    where
        'w: 'a;

    /// clears this Component storage, unsetting the value for each Entity
    fn clear(&mut self);
}

pub trait RwAccess<'a, C: Component>: ReadAccess<'a, C> + WriteAccess<'a, C> {}

// Storage types

/// Sparse vector storage. Possible the fastest type in terms of read access.
/// Write access can be horrible slow (might need to reallocate large vectors), and has bad
/// memory usage.
#[derive(Debug)]
pub struct VecStorage<C: Component + Debug> {
    data: Vec<Option<C>>,
}

impl<C: Component + Debug> Default for VecStorage<C> {
    fn default() -> Self {
        VecStorage { data: Vec::new() }
    }
}

impl<C: Component + Debug> VecStorage<C> {
    pub fn new() -> VecStorage<C> {
        VecStorage::default()
    }
}

/// Read accessor for VecStorage
pub struct VecReader<'r, C: 'r> {
    data: &'r Vec<Option<C>>,
}

impl<'r, C> VecReader<'r, C> {
    pub fn new(data: &'r Vec<Option<C>>) -> VecReader<'r, C> {
        VecReader { data }
    }
}

// ReadAccess requires the struct to implement IntoIterator
impl<'v: 'r, 'r, C: 'v + Component> ReadAccess<'r, C> for VecReader<'v, C> {
    fn fetch(&self, entity: Entity) -> Option<&C> {
        self.data.get(usize::from(entity)).unwrap_or(&None).as_ref()
    }

    fn iter<'a>(&'r self) -> Box<dyn Iterator<Item = (Entity, &'a C)> + 'a>
    where
        'r: 'a,
    {
        Box::new(
            self.data
                .iter()
                .enumerate()
                .filter_map(|(index, maybe_val)| match maybe_val {
                    Some(val) => Some((index.into(), val)),
                    None => None,
                }),
        )
    }
}

/// Write access to a VecStorage. Uses mutable borrow so there can only exists one writer at a time.
pub struct VecWriter<'v, C> {
    data: &'v mut Vec<Option<C>>,
}

impl<'v, C: Component> VecWriter<'v, C> {
    pub fn new(data: &'v mut Vec<Option<C>>) -> VecWriter<'v, C> {
        VecWriter { data }
    }
}

impl<'v: 'w, 'w, C: Component> WriteAccess<'w, C> for VecWriter<'v, C> {
    fn set(&mut self, entity: Entity, value: C) {
        let index = entity.into();

        if self.data.capacity() <= index {
            self.data.reserve(index - self.data.capacity() + 1);
        }

        if self.data.len() <= index {
            for _ in 0..index - self.data.len() {
                self.data.push(None);
            }
            self.data.push(Some(value));
        } else {
            self.data[index] = Some(value);
        }
    }

    fn unset(&mut self, entity: Entity) {
        let index = entity.into();

        // Can't be in this storage, so return early.
        if self.data.capacity() <= index {
            return;
        }

        self.data[index] = None;
    }

    fn iter_mut<'a>(&'w mut self) -> Box<dyn Iterator<Item = (Entity, &'a mut C)> + 'a>
    where
        'w: 'a,
    {
        Box::new(self.data.iter_mut().enumerate().filter_map(
            |(index, maybe_val)| match maybe_val {
                Some(val) => Some((index.into(), val)),
                None => None,
            },
        ))
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

impl<'v: 'w, 'w, C: Component> ReadAccess<'w, C> for VecWriter<'v, C> {
    fn fetch(&self, entity: Entity) -> Option<&C> {
        self.data.get(usize::from(entity)).unwrap_or(&None).as_ref()
    }

    fn iter<'a>(&'w self) -> Box<dyn Iterator<Item = (Entity, &'a C)> + 'a>
    where
        'w: 'a,
    {
        Box::new(
            self.data
                .iter()
                .enumerate()
                .filter_map(|(index, maybe_val)| match maybe_val {
                    Some(val) => Some((index.into(), val)),
                    None => None,
                }),
        )
    }
}

impl<'v: 'w, 'w, C: Component> RwAccess<'w, C> for VecWriter<'v, C> {}

// finally, we can implement CompontentStorage for VecStorage using the reader and writer we
// implemented above
impl<C: 'static + Component + Debug> ComponentStorage<C> for VecStorage<C> {
    type Reader<'r> = VecReader<'r, C>;
    type Writer<'w> = VecWriter<'w, C>;

    fn read<'r, 'd: 'r>(&'d self) -> Self::Reader<'r>
    where
        C: 'r,
    {
        VecReader::new(&self.data)
    }

    fn write<'w, 'd: 'w>(&'d mut self) -> Self::Writer<'w> {
        VecWriter::new(&mut self.data)
    }
}

type DequeData<C> = VecDeque<(Entity, C)>;

/// Deque-based storage for items that get added and cleared of (event-like)
#[derive(Debug)]
pub struct DequeStorage<C: Component + Debug> {
    data: DequeData<C>,
}

impl<C: Component + Debug> Default for DequeStorage<C> {
    fn default() -> Self {
        DequeStorage {
            data: VecDeque::new(),
        }
    }
}

impl<C: Component + Debug> DequeStorage<C> {
    pub fn new() -> DequeStorage<C> {
        DequeStorage::default()
    }
}

pub struct DequeReader<'d, C> {
    data: &'d DequeData<C>,
}

impl<'d, C> DequeReader<'d, C> {
    pub fn new(data: &'d DequeData<C>) -> DequeReader<'d, C> {
        DequeReader { data }
    }
}

impl<'d: 'r, 'r, C: 'd + Component> ReadAccess<'r, C> for DequeReader<'d, C> {
    fn fetch(&self, entity: Entity) -> Option<&C> {
        for (other, component) in self.data {
            if entity == *other {
                return Some(component);
            }
        }

        None
    }

    fn iter<'a>(&'r self) -> Box<dyn Iterator<Item = (Entity, &'a C)> + 'a>
    where
        'r: 'a,
    {
        Box::new(self.data.iter().map(|(e, c)| (e.clone(), c)))
    }
}

pub struct DequeWriter<'d, C> {
    data: &'d mut DequeData<C>,
}

impl<'d, C> DequeWriter<'d, C> {
    pub fn new(data: &'d mut DequeData<C>) -> DequeWriter<'d, C> {
        DequeWriter { data }
    }
}

impl<'d: 'w, 'w, C: 'd + Component> WriteAccess<'w, C> for DequeWriter<'d, C> {
    fn set(&mut self, entity: Entity, value: C) {
        match self.data.iter().position(|(e, _)| *e == entity) {
            Some(index) => self.data[index] = (entity, value),
            None => self.data.push_back((entity, value)),
        }
    }

    fn unset(&mut self, entity: Entity) {
        unimplemented!()
    }

    fn iter_mut<'a>(&'w mut self) -> Box<dyn Iterator<Item = (Entity, &'a mut C)> + 'a>
    where
        'w: 'a,
    {
        Box::new(self.data.iter_mut().map(|(e, c)| (e.clone(), c)))
    }

    fn clear(&mut self) {
        self.data.clear();
    }
}

impl<C: 'static + Component + Debug> ComponentStorage<C> for DequeStorage<C> {
    type Reader<'r> = DequeReader<'r, C>;
    type Writer<'w> = DequeWriter<'w, C>;

    fn read<'r, 'd: 'r>(&'d self) -> Self::Reader<'r>
    where
        C: 'r,
    {
        DequeReader::new(&self.data)
    }

    fn write<'w, 'd: 'w>(&'d mut self) -> Self::Writer<'w> {
        DequeWriter::new(&mut self.data)
    }
}
