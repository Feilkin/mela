//! I don't know :shrug:

use replace_with::replace_with_or_abort;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

use crate::debug::DebugContext;
use crate::game::Playable;
use crate::gfx::{default_render_pipelines, RenderContext};
use std::thread::sleep;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalSize;

pub struct Settings {
    pub window_size: [f32; 2],
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            window_size: [1280., 720.],
        }
    }
}

pub struct Application<G: 'static + Playable> {
    title: String,
    game: G,
    settings: Settings,
}

impl<G: 'static + Playable> Application<G> {
    pub fn new<T: Into<String>>(game: G, title: T) -> Application<G> {
        Application {
            game,
            title: title.into(),
            settings: Settings::default(),
        }
    }

    pub fn new_with_settings<T: Into<String>>(
        game: G,
        title: T,
        settings: Settings,
    ) -> Application<G> {
        Application {
            game,
            title: title.into(),
            settings,
        }
    }

    /// Runs the game, consuming it
    pub fn run(self) -> () {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(
                self.settings.window_size[0],
                self.settings.window_size[1],
            ))
            .with_resizable(false)
            .with_title(&self.title)
            .build(&event_loop)
            .expect("Failed to create window");

        // TODO: move this init stuff away from here
        let size = window.inner_size();

        let surface = wgpu::Surface::create(&window);

        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            backends: wgpu::BackendBit::PRIMARY,
        })
        .unwrap();

        let (device, mut queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let render_pipelines = default_render_pipelines(&device);

        let mut sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Vsync,
        };

        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

        let mut game = self.game;
        let mut last_update = Instant::now();
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::LoopDestroyed => return,
                Event::MainEventsCleared => window.request_redraw(),
                Event::RedrawRequested(_) => {
                    let frame = swap_chain.get_next_texture();
                    let update_encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
                    let draw_encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });

                    let delta = last_update.elapsed();
                    last_update = Instant::now();

                    let (frame, update_buffer, draw_buffer) = {
                        let mut render_ctx = RenderContext {
                            frame: &frame.view,
                            encoder: update_encoder,
                            device: &device,
                            pipelines: &render_pipelines,
                        };
                        let mut debug_ctx = DebugContext {};

                        replace_with_or_abort(&mut game, |game| {
                            game.update(delta, &mut render_ctx, &mut debug_ctx)
                        });

                        let RenderContext { encoder, .. } = render_ctx;

                        let update_buffer = encoder.finish();

                        let mut render_ctx = RenderContext {
                            encoder: draw_encoder,
                            ..render_ctx
                        };

                        game.redraw(&mut render_ctx, &mut debug_ctx);

                        let RenderContext { frame, encoder, .. } = render_ctx;

                        (frame, update_buffer, encoder.finish())
                    };

                    queue.submit(&[update_buffer, draw_buffer])
                }
                event @ _ => match game.push_event(&event) {
                    Some(flow) => *control_flow = flow,
                    None => (),
                },
            }
        });
    }
}
