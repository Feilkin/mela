//! Signed Distance Fields

pub mod renderer;

pub enum SdfShape {
    Ball(f32),
}

pub struct SdfObject {
    pub shape: SdfShape,
}
