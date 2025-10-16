use std::sync::Arc;

use glam::Vec3;
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};

pub mod logic;
pub mod texture;
pub mod utils;

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    piece_vb: wgpu::Buffer,
    board_vb: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    surface_configured: bool,
    pieces_render_pipeline: wgpu::RenderPipeline,
    board_render_pipeline: wgpu::RenderPipeline,
    texture_bind_group: wgpu::BindGroup,
    game_info_bind_group: wgpu::BindGroup,
    game_info_buffer: wgpu::Buffer,
    game_info: GameInfo,
    last_time: std::time::Instant,
    board_state: logic::BoardState,
    mouse_down: bool,
    holding_piece: bool,
    window: Arc<Window>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            }],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GameInfo {
    time: f32,
    state: u32,
    legal_moves_low: u32,
    legal_moves_high: u32,
}

impl GameInfo {
    fn set_legal_moves(&mut self, moves: &[u8]) {
        self.clear_legal_moves();
        for &mv in moves.iter() {
            if mv < 32 {
                self.legal_moves_low |= 1 << mv;
            } else {
                self.legal_moves_high |= 1 << (mv - 32);
            }
        }
    }
    fn clear_legal_moves(&mut self) {
        self.legal_moves_low = 0;
        self.legal_moves_high = 0;
    }
    fn set_white_to_play(&mut self, white_to_play: bool) {
        self.state = (self.state & !0x1) | ((white_to_play as u32) & 0x1); // bit 0
    }
    fn set_hovered(&mut self, hovered: u32) {
        self.state = (self.state & !(0x7F << 1)) | ((hovered & 0x7F) << 1); // bits 1-7
    }
    fn hovered(&self) -> u32 {
        (self.state >> 1) & 0x7F
    }
    fn set_selected(&mut self, selected: u32) {
        self.state = (self.state & !(0x7F << 8)) | ((selected & 0x7F) << 8); // bits 8-14
    }
    fn selected(&self) -> u32 {
        (self.state >> 8) & 0x7F
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Instance {
    position: [f32; 3],
    data: u32, // bit 0 = white, bits 1-4 = piece, bits 5-10 = index
}

impl Instance {
    fn new(position: [f32; 3], piece: u32, white: u32, index: u32) -> Self {
        Self {
            position,
            data: (white & 0x1) | ((piece & 0xF) << 1) | ((index & 0x3F) << 5),
        }
    }
    fn _white(&self) -> u32 {
        self.data & 0x1
    }
    fn _piece(&self) -> u32 {
        (self.data >> 1) & 0xF
    }
    fn index(&self) -> u32 {
        (self.data >> 5) & 0x3F
    }
}

impl Instance {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Instance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

impl State {
    pub async fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoNoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let pieces_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string("assets/pieces.wgsl")?.into()),
        });

        let board_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(std::fs::read_to_string("assets/board.wgsl")?.into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let tex = texture::Texture::atlas_from(
            &device,
            &queue,
            vec![
                "pieces/wr.png",
                "pieces/wn.png",
                "pieces/wb.png",
                "pieces/wq.png",
                "pieces/wk.png",
                "pieces/wp.png",
                "pieces/br.png",
                "pieces/bn.png",
                "pieces/bb.png",
                "pieces/bq.png",
                "pieces/bk.png",
                "pieces/bp.png",
            ],
            6,
            false,
        )?;

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&tex.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&tex.sampler),
                },
            ],
            label: None,
        });

        let game_info = GameInfo {
            time: 0.0,
            state: 1,
            legal_moves_low: 0,
            legal_moves_high: 0,
        };

        let game_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Game Info Buffer"),
            contents: bytemuck::cast_slice(&[game_info]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let game_info_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("game_info_bind_group_layout"),
            });

        let game_info_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &game_info_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: game_info_buffer.as_entire_binding(),
            }],
            label: Some("game_info_bind_group"),
        });

        let pieces_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &game_info_bind_group_layout],
                push_constant_ranges: &[],
            });

        let pieces_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&pieces_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &pieces_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc(), Instance::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &pieces_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),

                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },

                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let board_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&game_info_bind_group_layout],
                push_constant_ranges: &[],
            });

        let board_render_pipeline =
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&board_render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &board_shader,
                    entry_point: Some("vs_main"),
                    buffers: &[Vertex::desc()],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &board_shader,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),

                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },

                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            });

        let args = std::env::args().collect::<Vec<_>>();
        let fen = if args.len() > 1 {
            &args[1..].join(" ")
        } else {
            logic::STARTING_FEN
        };
        let board_state = logic::BoardState::from_fen(fen)?;

        let instances = board_state
            .pieces
            .iter()
            .enumerate()
            .filter_map(|(index, piece)| {
                let piece = match piece {
                    Some(p) => p,
                    None => return None,
                };

                let instance_position = [
                    ((index as i32 % 8 - 4) as f32) * 0.125,
                    ((index as i32 / 8 - 4) as f32) * 0.125,
                    0.0,
                ];

                Some(Instance::new(
                    instance_position,
                    piece.to_idx(),
                    piece.white as u32,
                    index as u32,
                ))
            })
            .collect::<Vec<_>>();

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(&instances),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let piece_vertices =
            utils::Quad::from(Vec3::ZERO, Vec3::ONE / 8.2).map(|pos| Vertex { position: pos });

        let piece_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pieces Vertex Buffer"),
            contents: bytemuck::cast_slice(&piece_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let board_vertices = utils::Quad::from(Vec3::new(-0.5, -0.5, 0.0), Vec3::ONE)
            .map(|pos| Vertex { position: pos });

        let board_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&board_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&utils::Quad::generate_indices(4)),
            usage: wgpu::BufferUsages::INDEX,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            piece_vb,
            board_vb,
            index_buffer,
            instances,
            instance_buffer,
            surface_configured: false,
            pieces_render_pipeline,
            board_render_pipeline,
            texture_bind_group,
            game_info_bind_group,
            game_info_buffer,
            game_info,
            last_time: std::time::Instant::now(),
            board_state,
            mouse_down: false,
            holding_piece: false,
            window,
        })
    }

    pub fn update_instances(&mut self) {
        self.instances = self
            .board_state
            .pieces
            .iter()
            .enumerate()
            .filter_map(|(index, piece)| {
                let piece = match piece {
                    Some(p) => p,
                    None => return None,
                };

                let instance_position = [
                    ((index as i32 % 8 - 4) as f32) * 0.125,
                    ((index as i32 / 8 - 4) as f32) * 0.125,
                    0.0,
                ];

                Some(Instance::new(
                    instance_position,
                    piece.to_idx(),
                    piece.white as u32,
                    index as u32,
                ))
            })
            .collect::<Vec<_>>();

        self.queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&self.instances),
        );
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
            self.surface_configured = true;
        }
    }

    pub fn update(&mut self) {
        let now = std::time::Instant::now();
        self.game_info.time += now.duration_since(self.last_time).as_secs_f32();
        self.game_info
            .set_white_to_play(self.board_state.white_to_play);
        self.last_time = now;
        self.queue.write_buffer(
            &self.game_info_buffer,
            0,
            bytemuck::cast_slice(&[self.game_info]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.window.request_redraw();

        if !self.surface_configured {
            return Ok(());
        }

        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // board
            render_pass.set_pipeline(&self.board_render_pipeline);
            render_pass.set_bind_group(0, &self.game_info_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.board_vb.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..1);

            // pieces
            render_pass.set_pipeline(&self.pieces_render_pipeline);
            render_pass.set_bind_group(0, &self.texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.game_info_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.piece_vb.slice(..));
            render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.draw_indexed(0..6, 0, 0..self.instances.len() as u32);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[derive(Default)]
pub struct App {
    state: Option<State>,
}
impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title("GPU Chess")
            .with_resizable(false)
            .with_inner_size(PhysicalSize::new(800, 800));

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        self.state = Some(pollster::block_on(State::new(window)).unwrap());
    }

    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: State) {
        self.state = Some(event);
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let state = match &mut self.state {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => state.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}

                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        let size = state.window.inner_size();
                        state.resize(size.width, size.height);
                    }
                    Err(e) => {
                        log::error!("Unable to rendear {}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(code),
                        state: key_state,
                        ..
                    },
                ..
            } => match (code, key_state.is_pressed()) {
                (KeyCode::Escape, true) => event_loop.exit(),
                (KeyCode::Space, true) => {
                    println!("{}", state.board_state.to_fen());
                }
                _ => {}
            },
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                state.holding_piece = state.mouse_down && state.game_info.selected() != 0;
                let size = state.window.inner_size();
                let x = ((position.x / size.width as f64 - 0.25) * 16.0).floor() as i32;
                let y = ((1.0 - position.y / size.height as f64 - 0.25) * 16.0).floor() as i32;
                let hovered = if !(0..8).contains(&x) || !(0..8).contains(&y) {
                    0
                } else {
                    (y * 8 + x + 1) as u32
                };
                if hovered != state.game_info.hovered() {
                    state.game_info.set_hovered(hovered);
                }
                if state.holding_piece {
                    let holding_piece = state
                        .instances
                        .iter_mut()
                        .find(|x| x.index() == state.game_info.selected() - 1);
                    if let Some(holding_piece) = holding_piece {
                        holding_piece.position = [
                            (position.x as f32 / size.width as f32) * 2.0 - 1.05,
                            (1.0 - position.y as f32 / size.height as f32) * 2.0 - 1.05,
                            0.0,
                        ];
                    }
                    state.queue.write_buffer(
                        &state.instance_buffer,
                        0,
                        bytemuck::cast_slice(&state.instances),
                    );
                }
            }
            WindowEvent::MouseInput {
                device_id: _,
                state: button_state,
                button,
            } => {
                if button == MouseButton::Left {
                    if button_state == ElementState::Pressed {
                        state.mouse_down = true;
                        if state.game_info.selected() == state.game_info.hovered() {
                            state.game_info.set_selected(0);
                            state.game_info.clear_legal_moves();
                        } else if state.game_info.selected() != 0 && state.game_info.hovered() != 0
                        {
                            let from = state.game_info.selected() as u8 - 1;
                            let to = state.game_info.hovered() as u8 - 1;
                            let legal_moves = state.board_state.legal_moves(from);
                            state.game_info.set_legal_moves(&legal_moves);
                            if legal_moves.contains(&to) {
                                state.board_state.make_move(from, to);
                                state.update_instances();

                                state.game_info.set_selected(0);
                                state.game_info.clear_legal_moves();
                            } else if let Some(piece) =
                                &state.board_state.pieces[state.game_info.hovered() as usize - 1]
                                && piece.white == state.board_state.white_to_play
                            {
                                state.game_info.set_selected(state.game_info.hovered());

                                let legal_moves = state
                                    .board_state
                                    .legal_moves(state.game_info.hovered() as u8 - 1);
                                state.game_info.set_legal_moves(&legal_moves);
                            } else {
                                state.game_info.set_selected(0);
                                state.game_info.clear_legal_moves();
                            }
                        } else if state.game_info.hovered() != 0
                            && let Some(piece) =
                                &state.board_state.pieces[state.game_info.hovered() as usize - 1]
                            && piece.white == state.board_state.white_to_play
                        {
                            state.game_info.set_selected(state.game_info.hovered());

                            let legal_moves = state
                                .board_state
                                .legal_moves(state.game_info.hovered() as u8 - 1);
                            state.game_info.set_legal_moves(&legal_moves);
                        }
                    } else {
                        if state.holding_piece && state.game_info.selected() != 0 {
                            let from = state.game_info.selected() as u8 - 1;
                            if state.game_info.hovered() != 0 {
                                let to = state.game_info.hovered() as u8 - 1;
                                if state.board_state.legal_moves(from).contains(&to) {
                                    state.board_state.make_move(from, to);
                                }
                            }
                            state.update_instances();
                            state.game_info.set_selected(0);
                            state.game_info.clear_legal_moves();
                        }

                        state.mouse_down = false;
                        state.holding_piece = false;
                    }
                }
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}
