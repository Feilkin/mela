use crate::components::{Enemy, Fire, Player, Position, Sprite};
use crate::states::loading::GameAssets;
use crate::states::winner::Loser;
use crate::states::{States, Winner};
use crate::systems::{
    FireSystem, LogBurner, MoveToPlayer, PlayerKiller, PlayerMvmtSystem, SpriteSystem,
};
use crate::world::MyWorld;
use mela::asset::tilemap::layers::ObjectLayer;
use mela::asset::tilemap::{Orthogonal, Tilemap};
use mela::debug::{DebugContext, DebugDrawable};
use mela::ecs::world::World;
use mela::ecs::{ComponentStorage, ReadAccess, System};
use mela::game::IoState;
use mela::gfx::light::{Light, Lights};
use mela::gfx::primitives::{Quad, Vertex, MVP};
use mela::gfx::{RenderContext, Spritebatch};
use mela::state::State;
use nalgebra::{Point2, Vector2, Vector3};
use std::time::Duration;

pub struct Play {
    assets: GameAssets,
    tilemap: Tilemap<Orthogonal, MyWorld>,
    world: MyWorld,
    systems: Vec<Box<dyn System<MyWorld>>>,
    render_targets: (wgpu::Texture, wgpu::Texture, wgpu::Texture),
    sampler: wgpu::Sampler,
    ui_batch: Spritebatch,
}

impl Play {
    pub fn new(assets: GameAssets, render_ctx: &mut RenderContext) -> Play {
        let tilemap = Tilemap::from_file("assets/tilemaps/ld46.json", render_ctx)
            .expect("failed to load tilemap");

        let mut world = MyWorld::new()
            .add_entity()
            .with_component(Sprite {
                quad: Quad::new(0., 480., 16., 32., 512., 512.),
            })
            .with_component(Position(Point2::new(360., 200.)))
            .with_component(Player {
                health: 3,
                direction: 0.0,
                invulnerable_timer: None,
            })
            .build();

        world = world
            .add_entity()
            .with_component(Sprite {
                quad: Quad::new(0., 448., 17., 27., 512., 512.),
            })
            .with_component(Position(Point2::new(360., 300.)))
            .with_component(Enemy { health: 3 })
            .build();

        world = world
            .add_entity()
            .with_component(Position(Point2::new(376., 217.)))
            .with_component(Fire {
                time_left: Duration::new(80, 0),
            })
            .build()
            .add_entity()
            .with_component(Position(Point2::new(374., 218.)))
            .with_component(Fire {
                time_left: Duration::new(60, 0),
            })
            .build()
            .add_entity()
            .with_component(Position(Point2::new(380., 220.)))
            .with_component(Fire {
                time_left: Duration::new(30, 0),
            })
            .build();

        let log_quad = Quad::new(0., 448., 17., 27., 512., 512.);

        for object in tilemap.layers()[1].objects() {
            if &object._type == "Log" {
                world = world
                    .add_entity()
                    .with_component(Sprite {
                        quad: log_quad.clone(),
                    })
                    .with_component(Position(Point2::new(object.x, object.y)))
                    .with_component(Enemy { health: 3 })
                    .build();
            }
        }

        let systems: Vec<Box<dyn System<MyWorld>>> = vec![
            Box::new(SpriteSystem::new(
                assets.textures.get("spritesheet").unwrap().clone(),
                assets.textures.get("material").unwrap().clone(),
            )),
            Box::new(PlayerMvmtSystem {}),
            Box::new(MoveToPlayer {}),
            Box::new(LogBurner {}),
            Box::new(FireSystem {}),
            Box::new(PlayerKiller {}),
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
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let material_target = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 768,
                height: 576,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let final_target = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width: 768,
                height: 576,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let ui_batch = Spritebatch::new(assets.textures.get("spritesheet").unwrap().clone());

        Play {
            assets,
            tilemap,
            world,
            systems,
            sampler,
            render_targets: (color_target, material_target, final_target),
            ui_batch,
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
            assets,
            mut ui_batch,
            ..
        } = self;

        for layer in tilemap.layers_mut() {
            layer.update(render_ctx);
        }

        for system in &mut systems {
            world = system.update(delta, world, io_state, render_ctx);
        }

        world = world.remove_dead();

        let enemies_left = world.components.enemies.read().iter().count();
        let players_left = world.components.players.read().iter().count();

        if enemies_left == 0 {
            States::Win(Winner::new(render_ctx))
        } else if players_left == 0 {
            States::Lose(Loser::new(assets, render_ctx))
        } else {
            // update health here because we lazy
            let health = world
                .components
                .players
                .read()
                .iter()
                .next()
                .unwrap()
                .1
                .health;

            if health > 0 {
                let heart_quad = Quad::new(0., 16., 16., 16., 512., 512.);

                ui_batch.clear();

                for i in 0..health as usize {
                    ui_batch.add_quad(&heart_quad, [8. + 20. * i as f32, 8.]);
                }

                ui_batch.update(render_ctx);
            }

            States::Play(Play {
                assets,
                tilemap,
                world,
                systems,
                ui_batch,
                ..self
            })
        }
    }

    fn redraw(&self, render_ctx: &mut RenderContext, debug_ctx: &mut DebugContext) {
        let projection = nalgebra::Matrix4::new_nonuniform_scaling(&Vector3::new(1., 16. / 9., 1.))
            .append_translation(&Vector3::new(-1., -1., 0.));

        let render_views = (
            self.render_targets.0.create_default_view(),
            self.render_targets.1.create_default_view(),
            self.render_targets.2.create_default_view(),
        );

        // draw color
        {
            for layer in self.tilemap.layers() {
                layer.draw_to(&projection, &[&render_views.0, &render_views.1], render_ctx);
            }

            for system in &self.systems {
                system.draw_to(&[&render_views.0, &render_views.1], render_ctx);
            }
        }

        // TODO: move to system
        let (player_entity, player) = self
            .world
            .components
            .players
            .read()
            .iter()
            .next()
            .and_then(|(entity, pc)| Some((entity, pc.clone())))
            .expect("no player??");

        let player_pos_component = self
            .world
            .components
            .positions
            .read()
            .fetch(player_entity)
            .expect("no player position??")
            .clone();

        let player_pos = [
            player_pos_component.0.coords.x + 14.,
            player_pos_component.0.coords.y + 16.,
            1.3,
        ];

        let num_fires = self.world.components.fires.read().iter().count() as f32;

        // finally, draw both to screen using raycast shader
        let lights = Lights::new(&[
            Light {
                position: [384. + 1. - rand::random::<f32>() * 2., 230., 0.3],
                radius: if num_fires > 0. { 50. } else { 0. },
                color: [1.0, 0.80, 0.20],
                strength: 1.2,
                angle: 0.,
                sector: std::f32::consts::PI,
                _padding: [0., 0.],
            },
            Light {
                position: [
                    384. + rand::random::<f32>(),
                    230. + rand::random::<f32>(),
                    1.3,
                ],
                radius: 100. * num_fires,
                color: [0.8, 0.3, 0.2],
                strength: 0.2,
                angle: 0.,
                sector: std::f32::consts::PI,
                _padding: [0., 0.],
            },
            Light {
                position: player_pos,
                radius: 100.,
                color: [0.7, 0.7, 1.0],
                strength: 1.0,
                angle: player.direction,
                sector: 0.13 * std::f32::consts::PI,
                _padding: [0., 0.],
            },
        ]);

        let light_buffer = render_ctx
            .device
            .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
            .fill_from_slice(&[lights]);

        let bind_group = render_ctx
            .device
            .create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &render_ctx.pipelines.raycast2d.1,
                bindings: &[
                    wgpu::Binding {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&render_views.0),
                    },
                    wgpu::Binding {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&render_views.1),
                    },
                    wgpu::Binding {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::Binding {
                        binding: 3,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &light_buffer,
                            range: 0..std::mem::size_of::<Lights>() as u64,
                        },
                    },
                ],
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

        {
            let mut rpass = render_ctx
                .encoder
                .begin_render_pass(&wgpu::RenderPassDescriptor {
                    color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                        attachment: &render_views.2,
                        resolve_target: None,
                        load_op: wgpu::LoadOp::Clear,
                        store_op: wgpu::StoreOp::Store,
                        clear_color: wgpu::Color::BLACK,
                    }],
                    depth_stencil_attachment: None,
                });

            rpass.set_pipeline(&render_ctx.pipelines.raycast2d.0);
            rpass.set_bind_group(0, &bind_group, &[]);
            rpass.set_index_buffer(&index_buf, 0);
            rpass.set_vertex_buffers(0, &[(&vertex_buf, 0)]);
            rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
        }

        {
            let transformations: [[f32; 4]; 4] = nalgebra::Matrix4::identity().into();

            let transforms_buffer = render_ctx
                .device
                .create_buffer_mapped(1, wgpu::BufferUsage::UNIFORM)
                .fill_from_slice(&[transformations]);

            let bind_group = render_ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &render_ctx.pipelines.pixel.1,
                    bindings: &[
                        wgpu::Binding {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&render_views.2),
                        },
                        wgpu::Binding {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&self.sampler),
                        },
                        wgpu::Binding {
                            binding: 2,
                            resource: wgpu::BindingResource::Buffer {
                                buffer: &transforms_buffer,
                                range: 0..std::mem::size_of::<[[f32; 4]; 4]>() as u64,
                            },
                        },
                    ],
                });

            {
                let mut rpass = render_ctx
                    .encoder
                    .begin_render_pass(&wgpu::RenderPassDescriptor {
                        color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                            attachment: &render_ctx.frame,
                            resolve_target: None,
                            load_op: wgpu::LoadOp::Clear,
                            store_op: wgpu::StoreOp::Store,
                            clear_color: wgpu::Color::BLACK,
                        }],
                        depth_stencil_attachment: None,
                    });
                rpass.set_pipeline(&render_ctx.pipelines.pixel.0);
                rpass.set_bind_group(0, &bind_group, &[]);
                rpass.set_index_buffer(&index_buf, 0);
                rpass.set_vertex_buffers(0, &[(&vertex_buf, 0)]);
                rpass.draw_indexed(0..indices.len() as u32, 0, 0..1);
            }

            self.ui_batch
                .draw(&nalgebra::Matrix4::identity(), render_ctx);
        }
    }
}

impl DebugDrawable for Play {}
