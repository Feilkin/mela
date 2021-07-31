//! I don't know :shrug:

use std::env;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};

use futures::executor::block_on;
use serde::{Deserialize, Serialize};
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::{
    event::Event,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::debug::DebugContext;
use crate::game::Playable;

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

pub struct ApplicationBuilder<G: 'static + Playable> {
    title: String,
    game: G,
    settings: Settings,
}

impl<G: 'static + Playable> ApplicationBuilder<G> {
    pub async fn setup(self) -> Application<G> {
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

        if let Ok(x) = env::var("MELA_WINDOW_X").map(|x| x.parse::<i32>().unwrap()) {
            if let Ok(y) = env::var("MELA_WINDOW_Y").map(|y| y.parse::<i32>().unwrap()) {
                window.set_outer_position(winit::dpi::LogicalPosition::<i32>::from((x, y)));
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            let canvas = window.canvas();

            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let body = document.body().unwrap();

            body.append_child(&canvas)
                .expect("Append canvas to HTML body");
        }
        //let instance = wgpu::Instance::new(wgpu::BackendBit::all());
        let instance = wgpu::Instance::new(wgpu::BackendBit::all());

        let surface = unsafe { instance.create_surface(&window) };

        instance
            .enumerate_adapters(wgpu::BackendBit::all())
            .for_each(|adapter| println!("{:?}", adapter));

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, mut queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("mela adapter"),
                    features: Default::default(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .expect("Failed to get rendering device");

        #[cfg(not(target_arch = "wasm32"))]
        let (imgui_ctx, platform, imgui_renderer) =
            Self::initialize_imgui(&window, &device, &mut queue);

        Application {
            event_loop,
            window,
            device,
            queue,
            surface,
            title: self.title,
            game: self.game,
            settings: self.settings,
            #[cfg(not(target_arch = "wasm32"))]
            platform,
            #[cfg(not(target_arch = "wasm32"))]
            imgui_ctx,
            #[cfg(not(target_arch = "wasm32"))]
            imgui_renderer,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn initialize_imgui(
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
    ) -> (
        imgui::Context,
        imgui_winit_support::WinitPlatform,
        imgui_wgpu::Renderer,
    ) {
        let mut imgui_ctx = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui_ctx);
        platform.attach_window(
            imgui_ctx.io_mut(),
            window,
            imgui_winit_support::HiDpiMode::Default,
        );
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

        let mut render_config = imgui_wgpu::RendererConfig::new();
        render_config.texture_format = wgpu::TextureFormat::Bgra8UnormSrgb;

        let mut imgui_renderer =
            imgui_wgpu::Renderer::new(&mut imgui_ctx, &device, queue, render_config);

        (imgui_ctx, platform, imgui_renderer)
    }
}

pub struct Application<G: 'static + Playable> {
    title: String,
    game: G,
    settings: Settings,
    event_loop: EventLoop<()>,
    window: winit::window::Window,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    #[cfg(not(target_arch = "wasm32"))]
    platform: imgui_winit_support::WinitPlatform,
    #[cfg(not(target_arch = "wasm32"))]
    imgui_ctx: imgui::Context,
    #[cfg(not(target_arch = "wasm32"))]
    imgui_renderer: imgui_wgpu::Renderer,
}

impl<G: 'static + Playable> Application<G> {
    pub fn new<T: Into<String>>(game: G, title: T) -> ApplicationBuilder<G> {
        ApplicationBuilder {
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
    ) -> ApplicationBuilder<G> {
        ApplicationBuilder {
            game,
            title: title.into(),
            settings,
        }
    }

    /// Runs the game, consuming it
    pub fn run(self) -> () {
        let Application {
            event_loop,
            mut game,
            settings,
            window,
            queue,
            device,
            surface,
            ..
        } = self;

        #[cfg(not(target_arch = "wasm32"))]
        let (mut platform, mut imgui_ctx, mut imgui_renderer) =
            (self.platform, self.imgui_ctx, self.imgui_renderer);

        let sample_count = 4;

        let size = window.inner_size();
        let sc_desc = wgpu::SwapChainDescriptor {
            usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: if settings.vsync {
                wgpu::PresentMode::Fifo
            } else {
                wgpu::PresentMode::Mailbox
            },
        };

        let mut swap_chain = device.create_swap_chain(&surface, &sc_desc);
        let ms_framebuffer = {
            let multisampled_texture_extent = wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth_or_array_layers: 1,
            };
            let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
                size: multisampled_texture_extent,
                mip_level_count: 1,
                sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: sc_desc.format,
                usage: wgpu::TextureUsage::RENDER_ATTACHMENT,
                label: None,
            };

            device
                .create_texture(multisampled_frame_descriptor)
                .create_view(&wgpu::TextureViewDescriptor::default())
        };

        let mut staging_belt = wgpu::util::StagingBelt::new(1024);
        let mut local_pool = futures::executor::LocalPool::new();
        let local_spawner = local_pool.spawner();

        let screen_size = (
            settings.window_size[0] as u32,
            settings.window_size[1] as u32,
        );
        let mut last_update = Instant::now();
        let update_interval = Duration::from_secs_f64(1. / settings.max_fps as f64);
        event_loop.run(move |event, _, control_flow| {
            //*control_flow = ControlFlow::WaitUntil(last_update + update_interval);
            *control_flow = ControlFlow::Poll;

            #[cfg(not(target_arch = "wasm32"))]
            platform.handle_event(imgui_ctx.io_mut(), &window, &event);

            match event {
                Event::LoopDestroyed => return,
                Event::MainEventsCleared => {
                    if last_update.elapsed() >= update_interval {
                        window.request_redraw()
                    }
                }
                Event::RedrawRequested(_) => {
                    if let Ok(frame) = swap_chain.get_current_frame() {
                        let update_encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });
                        let mut draw_encoder =
                            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                                label: None,
                            });

                        #[cfg(not(target_arch = "wasm32"))]
                        imgui_ctx.io_mut().update_delta_time(last_update.elapsed());

                        let delta = last_update.elapsed();
                        last_update = Instant::now();

                        let (_frame, update_buffer, draw_buffer) = {
                            #[cfg(not(target_arch = "wasm32"))]
                            let mut debug_ctx = {
                                platform
                                    .prepare_frame(imgui_ctx.io_mut(), &window)
                                    .expect("Failed to prepare imgui frame");
                                let ui = imgui_ctx.frame();

                                DebugContext {
                                    ui,
                                    ui_renderer: &mut imgui_renderer,
                                }
                            };
                            #[cfg(target_arch = "wasm32")]
                            let mut debug_ctx = DebugContext {};

                            game.update(delta, &mut debug_ctx);

                            let update_buffer = update_encoder.finish();

                            game.redraw(&mut debug_ctx);

                            #[cfg(not(target_arch = "wasm32"))]
                            let ui = debug_ctx.ui;

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                let mut imgui_rpass =
                                    draw_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                                        label: Some("imgui renderpass"),
                                        color_attachments: &[wgpu::RenderPassColorAttachment {
                                            view: &frame.output.view,
                                            resolve_target: None,
                                            ops: wgpu::Operations {
                                                load: wgpu::LoadOp::Load,
                                                store: true,
                                            },
                                        }],
                                        depth_stencil_attachment: None,
                                    });

                                debug_ctx
                                    .ui_renderer
                                    .render(ui.render(), &queue, &device, &mut imgui_rpass)
                                    .unwrap();
                            }

                            staging_belt.finish();
                            (frame, update_buffer, draw_encoder.finish())
                        };

                        queue.submit(vec![update_buffer, draw_buffer]);

                        //Recall unused staging buffers
                        use futures::task::SpawnExt;

                        local_spawner
                            .spawn(staging_belt.recall())
                            .expect("Recall staging belt");

                        local_pool.run_until_stalled();
                    }
                }
                event
                @
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { .. },
                    ..
                } => {
                    #[cfg(not(target_arch = "wasm32"))]
                    if !imgui_ctx.io().want_capture_mouse {
                        match game.push_event(&event) {
                            Some(flow) => *control_flow = flow,
                            None => (),
                        }
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        match game.push_event(&event) {
                            Some(flow) => *control_flow = flow,
                            None => (),
                        }
                    }
                }
                event => match game.push_event(&event) {
                    Some(flow) => *control_flow = flow,
                    None => (),
                },
            }
        });
    }
}
