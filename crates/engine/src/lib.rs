#![deny(warnings)]

use wgpu;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};
use postprocess::Postprocess;

mod util;
mod postprocess;
mod scene;

pub use util::buffer::{RgbaImage, IndexedImage, GrayscaleImage};
pub use scene::*;


const HAS_STENCIL: bool = false;
pub(crate) const DEBUG: bool = false;


#[derive(Debug, Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}


pub trait Spawner {
    fn spawn_local(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>);
}

impl<S> Spawner for Arc<S> where S: Spawner + ?Sized {
    #[inline]
    fn spawn_local(&self, future: Pin<Box<dyn Future<Output = ()> + 'static>>) {
        (**self).spawn_local(future);
    }
}


pub struct EngineSettings<Window> where Window: HasRawWindowHandle + HasRawDisplayHandle {
    pub window: Window,
    pub scene: Node,
    pub window_size: WindowSize,
    pub spawner: Arc<dyn Spawner>,
}


pub(crate) struct DepthBuffer {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    depth_view: wgpu::TextureView,
    stencil_view: Option<wgpu::TextureView>,
}

impl DepthBuffer {
    #[inline]
    pub(crate) fn format(&self) -> wgpu::TextureFormat {
        self.texture.format()
    }

    #[inline]
    pub(crate) fn has_stencil(&self) -> bool {
        self.stencil_view.is_some()
    }
}

impl Drop for DepthBuffer {
    fn drop(&mut self) {
        self.texture.destroy();
    }
}


pub(crate) struct EngineState {
    window_size: WindowSize,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_buffer: DepthBuffer,
    config: wgpu::SurfaceConfiguration,
}

impl EngineState {
    fn make_depth_buffer(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) -> DepthBuffer {
        let size = wgpu::Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };

        let format = if HAS_STENCIL {
            wgpu::TextureFormat::Depth24PlusStencil8

        } else {
            wgpu::TextureFormat::Depth32Float
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Buffer"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            // TODO change this based on whether postprocess is enabled or not
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Depth Buffer View"),
            ..wgpu::TextureViewDescriptor::default()
        });

        let depth_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Depth Buffer Depth View"),
            aspect: wgpu::TextureAspect::DepthOnly,
            ..wgpu::TextureViewDescriptor::default()
        });

        let stencil_view = if HAS_STENCIL {
            Some(texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Depth Buffer Stencil View"),
                aspect: wgpu::TextureAspect::StencilOnly,
                ..wgpu::TextureViewDescriptor::default()
            }))

        } else {
            None
        };

        DepthBuffer { texture, view, depth_view, stencil_view }
    }

    fn resize(&mut self, window_size: WindowSize) {
        self.window_size = window_size;
        self.config.width = window_size.width;
        self.config.height = window_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_buffer = EngineState::make_depth_buffer(&self.device, &self.config);
    }

    pub(crate) fn depth_stencil_state(&self, depth_write: bool, stencil: Option<wgpu::StencilState>) -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: self.depth_buffer.format(),
            depth_write_enabled: depth_write,
            depth_compare: wgpu::CompareFunction::Greater,
            stencil: if self.depth_buffer.has_stencil() {
                stencil.unwrap_or_else(|| wgpu::StencilState::default())
            } else {
                wgpu::StencilState::default()
            },
            bias: wgpu::DepthBiasState::default(),
        }
    }
}


pub struct Engine<Window> {
    // This must be owned by the Engine, for memory safety
    _window: Window,
    state: EngineState,
    postprocess: Option<Postprocess>,
    scene: Scene,
}

static_assertions::assert_not_impl_all!(EngineState: Send, Sync);
static_assertions::assert_not_impl_all!(Option<Postprocess>: Send, Sync);
static_assertions::assert_not_impl_all!(Scene: Send, Sync);

impl<Window> Engine<Window> where Window: HasRawWindowHandle + HasRawDisplayHandle {
    pub async fn new(settings: EngineSettings<Window>) -> Self {
        let window = settings.window;

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL,
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // Engine owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                // WebGL doesn't support all of wgpu's features, so if
                // we're building for the web we'll have to disable some.
                limits: wgpu::Limits {
                    max_texture_dimension_2d: 8192,
                    ..wgpu::Limits::downlevel_webgl2_defaults()
                },
                label: None,
            },
            None,
        ).await.unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        // Uses sRGB for rendering
        let surface_format = surface_caps.formats.iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: settings.window_size.width,
            height: settings.window_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let depth_buffer = EngineState::make_depth_buffer(&device, &config);

        let state = EngineState {
            window_size: settings.window_size,
            surface,
            device,
            queue,
            config,
            depth_buffer,
        };

        let scene = Scene::new(&state, settings.scene, settings.spawner);

        let postprocess = None;
        //let postprocess = Some(Postprocess::new(&state));

        Self {
            _window: window,
            state,
            postprocess,
            scene,
        }
    }

    pub fn resize(&mut self, window_size: WindowSize) {
        self.state.resize(window_size);

        if let Some(postprocess) = &mut self.postprocess {
            postprocess.resize(&self.state);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        if self.scene.should_render() {
            let mut scene_prerender = self.scene.prerender(&self.state);

            let output = self.state.surface.get_current_texture()?;

            let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder = self.state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: if let Some(postprocess) = &self.postprocess {
                            postprocess.view()
                        } else {
                            &view
                        },
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.state.depth_buffer.view,
                        depth_ops: Some(wgpu::Operations {
                            // TODO use reverse z-order
                            load: wgpu::LoadOp::Clear(0.0),
                            store: true,
                        }),
                        stencil_ops: if self.state.depth_buffer.has_stencil() {
                            Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(0),
                                store: true,
                            })

                        } else {
                            None
                        },
                    }),
                });

                scene_prerender.render(&mut render_pass);
            }

            if let Some(postprocess) = &mut self.postprocess {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Postprocessing Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

                postprocess.render(&mut render_pass);
            }

            self.state.queue.submit(std::iter::once(encoder.finish()));
            output.present();

            /*fn read_texture(encoder: , texture: &Texture, aspect: wgpu::TextureAspect) {
                texture.as_image_copy(),

                /*wgpu::ImageCopyTexture {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect,
                },*/
                wgpu::ImageCopyBuffer {
                    buffer: buffer,
                    layout: wgu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: None,
                        rows_per_image: None,
                    },
                },
                texture.size(),
            }

            read_texture(encoder, self.state.depth_buffer.texture, wgpu::TextureAspect::StencilOnly)*/
        }

        Ok(())
    }
}
