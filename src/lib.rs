mod math;
mod widgets;

use log::debug;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use wgpu::*;
use wgpu_text::glyph_brush::{self, ab_glyph};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{Key, NamedKey::*},
    window::{Window, WindowBuilder},
};

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub fn run() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    debug!("dpi scale is {}", window.scale_factor());

    let mut render_context = smol::block_on(RenderContext::new(&window));

    let mut rects: Vec<widgets::Rect> = Vec::new();
    rects.push(widgets::Rect::new(
        0.0,
        0.0,
        10.0,
        10.0,
        &mut render_context,
    ));
    let mut cursor_location: Option<PhysicalPosition<f64>> = None;

    #[cfg(target_arch = "wasm32")]
    {
        use winit::dpi::PhysicalSize;
        window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas().unwrap());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    event_loop
        .run(|event, window_target| match event {
            //Event::Resumed => window.request_redraw(),
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CursorLeft { .. } => cursor_location = None,
                WindowEvent::CursorMoved { position, .. } => cursor_location = Some(*position),
                WindowEvent::MouseInput {
                    state: ElementState::Pressed,
                    button: MouseButton::Left,
                    ..
                } if cursor_location.is_some() => {
                    debug!("painting square at {:?}", cursor_location);
                    rects.push(widgets::Rect::new(
                        cursor_location.unwrap().x as f32,
                        cursor_location.unwrap().y as f32,
                        20.0,
                        20.0,
                        &mut render_context,
                    ));
                    window.request_redraw();
                }
                WindowEvent::RedrawRequested => match render_context.render(&rects) {
                    Ok(_) => (),
                    Err(SurfaceError::Lost) => render_context.resize(render_context.window_size),
                    Err(SurfaceError::OutOfMemory) => window_target.exit(),
                    Err(e) => eprintln!("{:?}", e),
                },
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            state: ElementState::Pressed,
                            logical_key: Key::Named(Escape),
                            ..
                        },
                    ..
                } => window_target.exit(),
                WindowEvent::Resized(new_size) => render_context.resize(*new_size),
                _ => {}
            },
            _ => {}
        })
        .expect("The event loop should exit gracefully");
}

pub struct WindowBindGroup {
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
    pub window_to_device_transform_buffer: Buffer,
    pub window_size_buffer: Buffer,
}

impl WindowBindGroup {
    pub fn new(device: &Device, window: &winit::window::Window) -> Self {
        use util::DeviceExt;
        let transform = crate::math::window_to_wgpu_transform(window);
        let window_to_device_transform_buffer =
            device.create_buffer_init(&util::BufferInitDescriptor {
                label: None,
                contents: &bytemuck::cast_slice(AsRef::<[f32; 16]>::as_ref(&transform)),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

        let window_size = window.inner_size().to_logical(window.scale_factor());
        let window_size_vector: [f32; 4] = [window_size.width, window_size.height, 0.0, 0.0];

        let window_size_buffer = device.create_buffer_init(&util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&window_size_vector),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(LAYOUT_DESCRIPTOR);

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("WindowBindGroup"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: window_to_device_transform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: window_size_buffer.as_entire_binding(),
                },
            ],
        });

        Self {
            bind_group,
            bind_group_layout,
            window_to_device_transform_buffer,
            window_size_buffer,
        }
    }
}

const LAYOUT_DESCRIPTOR: &'static BindGroupLayoutDescriptor<'static> = &BindGroupLayoutDescriptor {
    label: Some("WindowBindGroup"),
    entries: &[
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::VERTEX_FRAGMENT,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ],
};

struct RenderContext<'w> {
    window: &'w Window,
    surface: Surface<'w>,
    device: Device,
    queue: Queue,
    surface_config: SurfaceConfiguration,
    window_size: winit::dpi::PhysicalSize<u32>,
    text_brush: wgpu_text::TextBrush,
    render_pipeline: RenderPipeline,
    window_bind_group: WindowBindGroup,
}

impl<'window> RenderContext<'window> {
    async fn new(window: &'window Window) -> Self {
        let window_size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&RequestAdapterOptionsBase {
                power_preference: PowerPreference::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    required_features: Features::empty(),
                    required_limits: if cfg!(target_arch = "wasm32") {
                        Limits::downlevel_webgl2_defaults()
                    } else {
                        Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &surface_config);

        let font = ab_glyph::FontArc::new(
            ab_glyph::FontRef::try_from_slice(include_bytes!("fonts/Roboto-Regular.ttf")).unwrap(),
        );
        let text_brush = wgpu_text::BrushBuilder::using_font(font).build(
            &device,
            window_size.width,
            window_size.height,
            surface_config.format,
        );

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let window_bind_group = WindowBindGroup::new(&device, window);

        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render pipeline layout"),
            bind_group_layouts: &[&window_bind_group.bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[math::Vertex::desc()],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(ColorTargetState {
                    format: surface_config.format,
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Cw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            window,
            surface,
            device,
            queue,
            surface_config,
            window_size,
            text_brush,
            render_pipeline,
            window_bind_group,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.window_size = new_size;
            self.surface_config.width = new_size.width;
            self.surface_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surface_config);
            self.text_brush
                .resize_view(new_size.width as f32, new_size.height as f32, &self.queue);
        }
    }

    fn render(&mut self, rects: &Vec<widgets::Rect>) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let transform = math::window_to_wgpu_transform(self.window);

        self.queue.write_buffer(
            &self.window_bind_group.window_to_device_transform_buffer,
            0,
            bytemuck::cast_slice(AsRef::<[f32; 16]>::as_ref(&transform)),
        );

        let window_size = self
            .window
            .inner_size()
            .to_logical(self.window.scale_factor());
        let window_size_vector: [f32; 4] = [window_size.width, window_size.height, 0.0, 0.0];

        self.queue.write_buffer(
            &self.window_bind_group.window_size_buffer,
            0,
            bytemuck::cast_slice(&window_size_vector),
        );

        let mut sections = Vec::new();

        for rect in rects {
            sections.push(
                glyph_brush::Section::default()
                    .add_text(glyph_brush::Text::new("(;-;)"))
                    .with_screen_position((rect.x, rect.y))
                    .with_bounds((rect.width, rect.height)),
            );
        }
        self.text_brush.queue(&self.device, &self.queue, sections);

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Main render pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.window_bind_group.bind_group, &[]);
            for rect in rects {
                rect.render(&mut render_pass);
            }
            self.text_brush.draw(&mut render_pass);
        }

        self.queue.submit([encoder.finish()]);
        output.present();

        Ok(())
    }
}
