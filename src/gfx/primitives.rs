//! Graphical primitives

use zerocopy::{AsBytes, FromBytes};

use crate::debug::DebugContext;
use crate::ecs::component::Transform;
use crate::ecs::system::Read;
use crate::ecs::world::{World, WorldStorage};
use crate::ecs::{Component, System};
use crate::game::IoState;
use crate::gfx::{RenderContext, Texture};
use lyon::lyon_algorithms::path::Path;
use lyon::lyon_tessellation::{
    BuffersBuilder, FillAttributes, FillOptions, FillTessellator, StrokeAttributes, StrokeOptions,
    StrokeTessellator, VertexBuffers,
};
use std::time::Duration;

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub texture_coords: [f32; 2],
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, AsBytes, FromBytes)]
pub struct Vertex2D {
    pub position: [f32; 2],
    pub texture_coords: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Copy)]
pub struct Quad {
    position: [f32; 2],
    size: [f32; 2],
    source_size: [f32; 2],
}

impl Quad {
    pub fn new(x: f32, y: f32, width: f32, height: f32, sw: f32, sh: f32) -> Quad {
        Quad {
            position: [x, y],
            size: [width, height],
            source_size: [sw, sh],
        }
    }

    pub fn vertices_and_indices(
        &self,
        translation: [f32; 3],
        color: [f32; 4],
    ) -> ([Vertex; 4], [u16; 6]) {
        let [sw, sh] = self.source_size;

        let (w, h) = (self.size[0] / sw, self.size[1] / sh);

        // left
        let x0 = self.position[0] / sw;
        // top
        let y0 = self.position[1] / sh;
        // right
        let x1 = x0 + w;
        // down
        let y1 = y0 + h;
        let z = translation[2];

        // make normal face Z axis because we lazy
        let normal = [0., 0., -1.];

        (
            [
                // top left
                Vertex {
                    position: [translation[0], translation[1], z],
                    normal,
                    color,
                    texture_coords: [x0, y0],
                },
                // top right
                Vertex {
                    position: [translation[0] + self.size[0], translation[1], z],
                    normal,
                    color,
                    texture_coords: [x1, y0],
                },
                // bottom left
                Vertex {
                    position: [translation[0], translation[1] + self.size[1], z],
                    normal,
                    color,
                    texture_coords: [x0, y1],
                },
                // bottom right
                Vertex {
                    position: [
                        translation[0] + self.size[0],
                        translation[1] + self.size[1],
                        z,
                    ],
                    normal,
                    color,
                    texture_coords: [x1, y1],
                },
            ],
            [0, 1, 3, 0, 3, 2],
        )
    }
    pub fn vertices_and_indices2d(
        &self,
        translation: [f32; 2],
        color: [f32; 4],
    ) -> ([Vertex2D; 4], [u16; 6]) {
        let [sw, sh] = self.source_size;
        let (w, h) = (self.size[0] / sw, self.size[1] / sh);

        // left
        let x0 = self.position[0] / sw;
        // top
        let y0 = self.position[1] / sh;
        // right
        let x1 = x0 + w;
        // down
        let y1 = y0 + h;

        (
            [
                // top left
                Vertex2D {
                    position: [translation[0], translation[1]],
                    color,
                    texture_coords: [x0, y0],
                },
                // top right
                Vertex2D {
                    position: [translation[0] + self.size[0], translation[1]],
                    color,
                    texture_coords: [x1, y0],
                },
                // bottom left
                Vertex2D {
                    position: [translation[0], translation[1] + self.size[1]],
                    color,
                    texture_coords: [x0, y1],
                },
                // bottom right
                Vertex2D {
                    position: [translation[0] + self.size[0], translation[1] + self.size[1]],
                    color,
                    texture_coords: [x1, y1],
                },
            ],
            [0, 1, 3, 0, 3, 2],
        )
    }
}

pub struct Mesh2D {
    vertices: Vec<Vertex2D>,
    indices: Vec<u16>,
    texture: Texture,
}

impl Mesh2D {
    pub fn new(vertices: Vec<Vertex2D>, indices: Vec<u16>, texture: Texture) -> Mesh2D {
        Mesh2D {
            vertices,
            indices,
            texture,
        }
    }

    pub fn draw(&self, _render_ctx: &mut RenderContext) {
        // FIXME implemtn this
    }
}

#[derive(Clone, Debug)]
pub struct PrimitiveComponent {
    pub color: [f32; 4],
    pub shape: PrimitiveShape,
}

#[derive(Clone, Debug)]
pub enum PrimitiveShape {
    Ball(f32, f32),
    Path(Path),
}

impl Component for PrimitiveComponent {}

pub struct PrimitiveRenderer {
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    primitives: Vec<(u32, u32)>,
}

impl PrimitiveRenderer {
    pub fn new() -> PrimitiveRenderer {
        PrimitiveRenderer {
            vertex_buffer: None,
            index_buffer: None,
            primitives: Vec::new(),
        }
    }
}

impl<W> System<W> for PrimitiveRenderer
where
    W: World + WorldStorage<Transform<f64>> + WorldStorage<PrimitiveComponent>,
{
    type SystemData<'a> = (Read<'a, Transform<f64>>, Read<'a, PrimitiveComponent>);

    fn name(&self) -> &'static str {
        "PrimitiveRenderer"
    }

    fn update<'f>(
        &mut self,
        (transforms, primitive_components): Self::SystemData<'f>,
        _delta: Duration,
        _io_state: &IoState,
        render_ctx: &mut RenderContext,
        _debug_ctx: &mut DebugContext,
    ) -> () {
        let mut primitives = Vec::new();
        let mut last_primitive_index = 0;
        let mut geometry_buffer: VertexBuffers<Vertex2D, u16> = VertexBuffers::new();
        let mut tesselator = StrokeTessellator::new();

        for (entity, prim) in primitive_components.iter() {
            if let Some(transform) = transforms.fetch(entity) {
                let color = prim.color;
                let mut buffer_builder = BuffersBuilder::new(
                    &mut geometry_buffer,
                    move |pos: lyon::math::Point, _: StrokeAttributes| Vertex2D {
                        position: pos.to_array(),
                        texture_coords: [0., 0.],
                        color,
                    },
                );

                let count = match &prim.shape {
                    PrimitiveShape::Ball(radiusX, radiusY) => {
                        lyon::tessellation::basic_shapes::stroke_ellipse(
                            lyon::math::point(
                                transform.0.translation.vector.x as f32,
                                transform.0.translation.vector.y as f32,
                            ),
                            lyon::tessellation::math::Vector::new(*radiusX, *radiusY),
                            lyon::tessellation::math::Angle::radians(
                                transform.rotation.angle() as f32
                            ),
                            &StrokeOptions::default()
                                .with_line_width(1.3)
                                .with_tolerance(0.5),
                            &mut buffer_builder,
                        )
                        .unwrap()
                    }
                    PrimitiveShape::Path(path) => tesselator
                        .tessellate_path(
                            path,
                            &StrokeOptions::default().with_line_width(2.),
                            &mut buffer_builder,
                        )
                        .unwrap(),
                };

                primitives.push((last_primitive_index, last_primitive_index + count.indices));
                last_primitive_index = last_primitive_index + count.indices;
            }
        }

        if last_primitive_index == 0 {
            return;
        }

        self.primitives = primitives;

        self.vertex_buffer = Some(render_ctx.device.create_buffer_with_data(
            geometry_buffer.vertices.as_bytes(),
            wgpu::BufferUsage::VERTEX,
        ));

        self.index_buffer =
            Some(render_ctx.device.create_buffer_with_data(
                geometry_buffer.indices.as_bytes(),
                wgpu::BufferUsage::INDEX,
            ));
    }

    fn draw(&self, render_ctx: &mut RenderContext) {
        if self.index_buffer.is_none() {
            return;
        }

        let proj = nalgebra::Matrix4::new_orthographic(0., 1280., 720., 0., -10., 10.);
        let view = nalgebra::Matrix4::identity();

        let vp = MVP {
            view: view.into(),
            proj: proj.into(),
            camera_pos: [0., 0., 0.],
            _padding: 0.0,
        };

        let global_buffer = render_ctx
            .device
            .create_buffer_with_data(vp.as_bytes(), wgpu::BufferUsage::UNIFORM);

        let global_bind_group = render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_ctx.pipelines.primitives.1,
                bindings: &[wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &global_buffer,
                        range: Default::default(),
                    },
                }],
                label: None,
            });

        {
            let mut pass = render_ctx
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &render_ctx.frame,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: Default::default(),
                    }],
                    depth_stencil_attachment: None,
                });

            pass.set_pipeline(&render_ctx.pipelines.primitives.0);
            pass.set_bind_group(0, &global_bind_group, &[]);
            pass.set_index_buffer(self.index_buffer.as_ref().unwrap(), 0, 0);
            pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap(), 0, 0);

            for (start, end) in &self.primitives {
                pass.draw_indexed(*start..*end, 0, 0..1);
            }
        }
    }
}

// TODO: wtf is this??
#[derive(Debug, Clone, Copy, AsBytes, FromBytes)]
#[repr(C)]
pub struct MVP {
    pub view: [[f32; 4]; 4],
    pub proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 3],
    pub _padding: f32,
}
