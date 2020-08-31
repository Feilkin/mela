//! I don't know :shrug:

use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

use futures::executor::block_on;
use replace_with::replace_with_or_abort;
use serde::{Deserialize, Serialize};
use winit::dpi::PhysicalSize;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::debug::DebugContext;
use crate::game::Playable;
use crate::gfx::{default_render_pipelines, RenderContext};

fn default_max_fps() -> u32 {
    300
}

#[derive(Serialize, Deserialize)]
pub struct Settings {
    pub window_size: [f32; 2],
    #[serde(default)]
    pub vsync: bool,
    #[serde(default = "default_max_fps")]
    pub max_fps: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            window_size: [1280., 720.],
            vsync: true,
            max_fps: 300,
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
            settings: Application::<G>::load_settings(),
        }
    }

    fn load_settings() -> Settings {
        if let Ok(file) = File::open("settings.json") {
            let reader = BufReader::new(file);
            serde_json::from_reader(reader).expect("failed to load settings")
        } else {
            Settings::default()
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

        let adapter = block_on(wgpu::Adapter::request(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
            },
            wgpu::BackendBit::PRIMARY,
        ))
        .unwrap();

        let (device, mut queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        }));

        let render_pipelines = default_render_pipelines(&device);

        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: if self.settings.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Mailbox
            },
        };

        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);

        // initialize imgui
        let mut imgui_ctx = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_ctx);
        platform.attach_window(
            imgui_ctx.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        let imgui_font_size = 13.0;
        imgui_ctx
            .fonts()
            .add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: 13.0,
                    oversample_h: 1,
                    pixel_snap_h: true,
                    ..Default::default()
                }),
            }]);

        let mut imgui_renderer = imgui_wgpu::Renderer::new(
            &mut imgui_ctx,
            &device,
            &mut queue,
            sc_desc.format,
            Some(wgpu::Color::BLACK),
        );

        let screen_size = (
            self.settings.window_size[0] as u32,
            self.settings.window_size[1] as u32,
        );
        let mut game = self.game;
        let mut last_update = Instant::now();
        let update_interval = Duration::from_secs_f64(1. / self.settings.max_fps as f64);
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::WaitUntil(last_update + update_interval);

            platform.handle_event(imgui_ctx.io_mut(), &window, &event);

            match event {
                Event::LoopDestroyed => return,
                Event::MainEventsCleared => {
                    if last_update.elapsed() >= update_interval {
                        window.request_redraw()
                    }
                }
                Event::RedrawRequested(_) => {
                    if let Ok(frame) = swap_chain.get_next_texture() {
                        let update_encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        let draw_encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        imgui_ctx.io_mut().update_delta_time(last_update);

                        let delta = last_update.elapsed();
                        last_update = Instant::now();

                        let (_, update_buffer, draw_buffer) = {
                            let mut render_ctx = RenderContext {
                                screen_size,
                                frame: &frame.view,
                                encoder: update_encoder,
                                device: &device,
                                pipelines: &render_pipelines,
                                window: &window,
                            };

                            platform
                                .prepare_frame(imgui_ctx.io_mut(), &window)
                                .expect("Failed to prepare imgui frame");
                            let ui = imgui_ctx.frame();

                            let mut debug_ctx = DebugContext {
                                ui,
                                ui_renderer: &mut imgui_renderer,
                            };

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

                            let DebugContext { ui, ui_renderer } = debug_ctx;

                            imgui_renderer
                                .render(
                                    ui.render(),
                                    &mut render_ctx.device,
                                    &mut render_ctx.encoder,
                                    &render_ctx.frame,
                                )
                                .unwrap();

                            let RenderContext { frame, encoder, .. } = render_ctx;

                            (frame, update_buffer, encoder.finish())
                        };

                        queue.submit(&[update_buffer, draw_buffer])
                    }
                }
                event @ _ => match game.push_event(&event) {
                    Some(flow) => *control_flow = flow,
                    None => (),
                },
            }
        });
    }
}
