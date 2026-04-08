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
use winit::event::Event;
use winit::window::Window;
use rust_pro::parser::descriptor::ParserDes;
use rust_pro::parser::parser::Parser;

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
						buffers: &[],
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
				primitive: Default::default(),
				depth_stencil: None,
				multisample: Default::default(),
				multiview: None,
		});
		//=================test===============
		test()?;
		event_loop.run(|event, window_target| {
				match event {
						Event::WindowEvent { event, .. } => match event {
								WindowEvent::CloseRequested => window_target.exit(),
								WindowEvent::Resized(size) => {
										config.width = size.width.max(1);
										config.height = size.height.max(1);
										surface.configure(&device, &config);
										let _ = &window.request_redraw();
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
																load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
																store: Default::default(),
														},
												})],
												depth_stencil_attachment: None,
												timestamp_writes: None,
												occlusion_query_set: None,
										});
										render_pass.set_pipeline(&pipeline);
										render_pass.draw(0..3, 0..1);
										drop(render_pass);
										queue.submit(Some(encoder.finish()));
										frame.present();
								}
								_ => {}
						},
						_ => {}
				}
		})?;
		Ok(())
}
fn test() -> Result<(), Box<dyn Error>> {
		println!("test");
		let module = Parser::new(ParserDes {
				path: "_module/cube.glb",
		})?;
		module.glb()?;
		Ok(())
}