use winit::event::WindowEvent;
use pollster::block_on;
use wgpu::*;
use wgpu::PresentMode::Fifo;
use wgpu::ShaderSource::Wgsl;
use winit::{
		event_loop::{EventLoop},
		window::WindowBuilder,
};
use std::error::Error;
use wgpu::PolygonMode::Fill;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use winit::event::Event;
use winit::window::Window;
use rust_pro::parser::descriptor::ParserDes;
use rust_pro::parser::parser::{Package, Parser, Vertex};

async fn init<'a>(window: &'a Window) -> Result<(Surface<'a>, Device, Queue, SurfaceConfiguration), Box<dyn Error>> {
		let instance = Instance::new(InstanceDescriptor::default());
		let surface = instance.create_surface(window)?;
		let adapter = instance.request_adapter(&RequestAdapterOptions {
				power_preference: PowerPreference::HighPerformance,
				force_fallback_adapter: false,
				compatible_surface: Some(&surface),
		}).await.unwrap();
		let (device, queue) = adapter.request_device(&DeviceDescriptor {
				label: None,
				required_features: Default::default(),
				required_limits: Default::default(),
		}, None).await?;
		let surface_configuration = SurfaceConfiguration {
				usage: TextureUsages::RENDER_ATTACHMENT,
				format: surface.get_capabilities(&adapter).formats[0],
				width: window.inner_size().width.max(1),
				height: window.inner_size().height.max(1),
				present_mode: Fifo,
				desired_maximum_frame_latency: 2,
				alpha_mode: CompositeAlphaMode::Auto,
				view_formats: vec![],
		};
		Ok((surface, device, queue, surface_configuration))
}
fn glb_loading() -> Result<Package, Box<dyn Error>> {
		println!("start loading glb");
		let module = Parser::new(ParserDes {
				path: "_module/cube.glb",
		})?;
		Ok(module.glb()?)
}
fn main() -> Result<(), Box<dyn Error>> {
		let event_loop = EventLoop::new()?;
		let window = WindowBuilder::new().with_title("graph-test").build(&event_loop)?;
		let (surface, device, queue, mut config) = block_on(init(&window))?;
		let (vert_shader, frag_shader) = (device.create_shader_module(ShaderModuleDescriptor {
				label: Some("vert_shader"),
				source: Wgsl(include_str!("shaders/tran_1.vert.wgsl").into()),
		}), device.create_shader_module(ShaderModuleDescriptor {
				label: Some("frag_shader"),
				source: Wgsl(include_str!("shaders/tran_2.frag.wgsl").into()),
		}));
		//================================
		let glb = glb_loading()?;
		let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
				label: Some("vertices buffer"),
				contents: bytemuck::cast_slice(&glb.vertices),
				usage: BufferUsages::VERTEX,
		});
		let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
				label: Some("index buffer"),
				contents: bytemuck::cast_slice(&glb.indices),
				usage: BufferUsages::INDEX,
		});
		let vertex_layout = VertexBufferLayout {
				array_stride: size_of::<Vertex>() as BufferAddress,
				step_mode: VertexStepMode::Vertex,
				attributes: &[
						VertexAttribute {
								offset: 0,
								shader_location: 0,
								format: VertexFormat::Float32x3,
						},
				],
		};
		// 创建深度纹理
		let depth_texture = device.create_texture(&TextureDescriptor {
				label: Some("depth texture"),
				size: Extent3d {
						width: config.width,
						height: config.height,
						depth_or_array_layers: 1,
				},
				mip_level_count: 1,
				sample_count: 1,
				dimension: TextureDimension::D2,
				format: TextureFormat::Depth32Float,
				usage: TextureUsages::RENDER_ATTACHMENT,
				view_formats: &[],
		});
		let depth_view = depth_texture.create_view(&Default::default());
		//================================
		let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
				label: Some("render_pipeline"),
				layout: Option::from(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
						label: Some("rander_pipeline_layout"),
						bind_group_layouts: &[],
						push_constant_ranges: &[],
				})),
				vertex: VertexState {
						module: &vert_shader,
						entry_point: "main",
						buffers: &[vertex_layout],
				},
				fragment: FragmentState {
						module: &frag_shader,
						entry_point: "main",
						targets: &[Some(ColorTargetState {
								format: config.format,
								blend: Some(BlendState::REPLACE),
								write_mask: Default::default(),
						})],
				}.into(),
				primitive: PrimitiveState {
						topology: PrimitiveTopology::TriangleList,
						strip_index_format: None,
						front_face: Default::default(),
						cull_mode: None,
						unclipped_depth: false,
						polygon_mode: Fill,
						conservative: false,
				},
				depth_stencil: Some(DepthStencilState {
						format: TextureFormat::Depth32Float,
						depth_write_enabled: true,
						depth_compare: CompareFunction::Less,
						stencil: Default::default(),
						bias: Default::default(),
				}),
				multisample: Default::default(),
				multiview: None,
		});
		event_loop.run(|event, window_target| {
				if let Event::WindowEvent { event, .. } = event {
						match event {
								WindowEvent::CloseRequested => {
										window_target.exit();
								}
								WindowEvent::Resized(size) => {
										config.width = size.width.max(1);
										config.height = size.height.max(1);
										surface.configure(&device, &config);
										window.request_redraw();
								}
								WindowEvent::RedrawRequested => {
										let frame = match surface.get_current_texture() {
												Ok(frame) => frame,
												Err(_) => return,
										};
										let view = frame.texture.create_view(&Default::default());
										let mut encoder = device.create_command_encoder(&Default::default());
										
										let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
												label: Some("render_pass"),
												color_attachments: &[Some(RenderPassColorAttachment {
														view: &view,
														resolve_target: None,
														ops: Operations {
																load: LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
																store: StoreOp::Store,
														},
												})],
												depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
														view: &depth_view,
														depth_ops: Some(Operations {
																load: LoadOp::Clear(1.0),
																store: StoreOp::Store,
														}),
														stencil_ops: None,
												}),
												timestamp_writes: None,
												occlusion_query_set: None,
										});
										render_pass.set_pipeline(&pipeline);
										render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
										render_pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
										render_pass.draw_indexed(0..glb.indices.len() as u32, 0, 0..1);
										drop(render_pass);
										
										queue.submit(Some(encoder.finish()));
										frame.present();
								}
								_ => {}
						}
				}
		})?;
		Ok(())
}