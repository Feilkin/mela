use crate::components::{Player, Position, Sprite};
use crate::states::loading::GameAssets;
use crate::states::States;
use crate::systems::{PlayerMvmtSystem, SpriteSystem};
use crate::world::MyWorld;
use mela::asset::tilemap::{Orthogonal, Tilemap};
use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::world::World;
use mela::ecs::System;
use mela::game::IoState;
use mela::gfx::primitives::{Quad, Vertex, MVP};
use mela::gfx::RenderContext;
use mela::state::State;
use nalgebra::{Point2, Vector2, Vector3};
use std::time::Duration;
use mela::gfx::light::{Light, Lights};

pub struct Play {
    assets: GameAssets,
    tilemap: Tilemap<Orthogonal, MyWorld>,
    world: MyWorld,
    systems: Vec<Box<dyn System<MyWorld>>>,
    render_targets: (wgpu::Texture, wgpu::Texture),
    sampler: wgpu::Sampler,
}

impl Play {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Play {
        let tilemap = Tilemap::from_file("assets/tilemaps/ld46.json", render_ctx)
            .expect("failed to load tilemap");

        let world = MyWorld::new()
            .add_entity()
            .with_component(Sprite {
                quad: Quad::new(0., 480., 16., 32., 512., 512.),
            })
            .with_component(Position(Point2::new(100., 100.)))
            .with_component(Player(0))
            .build();

        let systems: Vec<Box<dyn System<MyWorld>>> = vec![
            Box::new(SpriteSystem::new(
                assets.textures.get("spritesheet").unwrap().clone(),
                assets.textures.get("material").unwrap().clone(),
            )),
            Box::new(PlayerMvmtSystem {}),
        ];

        let sampler = render_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: Default::default(),
            address_mode_v: Default::default(),
            address_mode_w: Default::default(),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: Default::default(),
            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Never,
        });

        let color_target = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 768,
                height: 576,
                depth: 1
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT
        });

        let material_target = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 768,
                height: 576,
                depth: 1
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT
        });


        Play {
            assets,
            tilemap,
            world,
            systems,
            sampler,
            render_targets: (color_target, material_target),
        }
    }
}

impl State for Play {
    type Wrapper = States;

    fn name(&self) -> &str {
        "Play"
    }

    fn update(
        self,
        delta: Duration,
        io_state: &IoState,
        render_ctx: &mut RenderContext,
        debug_ctx: &mut DebugContext,
    ) -> Self::Wrapper {
        let Play {
            mut tilemap,
            mut world,
            mut systems,
            ..
        } = self;

        for layer in tilemap.layers_mut() {
            layer.update(render_ctx);
        }

        for system in &mut systems {
            world = system.update(delta, world, io_state, render_ctx);
        }

        States::Play(Play {
            tilemap,
            world,
            systems,
            ..self
        })
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        let projection = nalgebra::Matrix4::new_nonuniform_scaling(&Vector3::new(1., 16. / 9., 1.))
            .append_translation(&Vector3::new(-1., -1., 0.));

        let render_views = (self.render_targets.0.create_default_view(), self.render_targets.1.create_default_view());

        // draw color
        {

            for layer in self.tilemap.layers() {
                layer.draw_to(&projection, &[&render_views.0, &render_views.1], render_ctx);
            }

            for system in &self.systems {
                system.draw_to(&[&render_views.0, &render_views.1], render_ctx);
            }
        }

        // finally, draw both to screen using raycast shader
        let lights = Lights::new(&[
            Light {
                position: [376., 216., 1.],
                _padding: 0.,
                color: [1.0, 0.80, 0.20],
                strength: 1.0,
            },
            Light {
                position: [100., 100., 1.],
                _padding: 0.,
                color: [1.0, 1.0, 1.0],
                strength: 1.0,
            },
        ]);

        let light_buffer = render_ctx.device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[lights]);

        let bind_group = render_ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &render_ctx.pipelines.raycast2d.1,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_views.0)
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&render_views.1)
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler)
                },
                wgpu::Binding {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &light_buffer,
                        range: 0 .. std::mem::size_of::<Lights>() as u64
                    }
                }
            ]
        });

        let quad = Quad::new(0., 0., 2., 2., 2., 2.);
        let (vertices, indices) = quad.vertices_and_indices2d([-1., -1.], [1., 1., 1., 1.]);

        let vertex_buf = render_ctx
            .device
            .create_buffer_mapped(vertices.len(), wgpu::BufferUsage::VERTEX)
            .fill_from_slice(&vertices);

        let index_buf = render_ctx
            .device
            .create_buffer_mapped(indices.len(), wgpu::BufferUsage::INDEX)
            .fill_from_slice(&indices);

        let mut rpass = render_ctx.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &render_ctx.frame,
                resolve_target: None,
                load_op: wgpu::LoadOp::Clear,
                store_op: wgpu::StoreOp::Store,
                clear_color: wgpu::Color::BLACK,
            }],
            depth_stencil_attachment: None
        });

        rpass.set_pipeline(&render_ctx.pipelines.raycast2d.0);
        rpass.set_bind_group(0, &bind_group, &[]);
        rpass.set_index_buffer(&index_buf, 0);
        rpass.set_vertex_buffers(0, &[(&vertex_buf, 0)]);
        rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
    }
}

impl DebugDrawable for Play {}
