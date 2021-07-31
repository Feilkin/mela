//! Graphics stuff

use std::mem;
use std::rc::Rc;

//pub mod primitives;

/// Type alias over reference counted wgpu texture
pub type Texture = Rc<wgpu::Texture>;
