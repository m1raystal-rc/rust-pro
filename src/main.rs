use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use winit::{
	event::*,
	event_loop::{ControlFlow, EventLoop},
	window::WindowBuilder,
};

const VERTEX_SHADER: &str = r#"
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    // 三个顶点位置（NDC坐标系）
    var positions = array<vec2<f32>, 3>(
        vec2( 0.0,  0.5),  // 顶部
        vec2(-0.5, -0.5),  // 左下
        vec2( 0.5, -0.5),  // 右下
    );
    
    let pos = positions[vertex_index];
    return vec4(pos, 0.0, 1.0);
}
"#;

const FRAGMENT_SHADER: &str = r#"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4(0.0, 0.6, 1.0, 1.0);  // 浅蓝色三角形
}
"#;

fn main() {
	let event_loop = EventLoop::new().unwrap();
	let window = Rc::new(WindowBuilder::new().with_title("wgpu 三角形").build(&event_loop).unwrap());
	
	// 将 Rc 转换为原始指针并泄漏，获得 'static 生命周期
	let window_ptr: &'static winit::window::Window = unsafe { &*(Rc::into_raw(window.clone()) as *const _) };
	
	let (surface, device, queue, mut config) = pollster::block_on(init_wgpu(window_ptr));
	
	let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("三角形着色器"),
		source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
	});
	let fragment_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
		label: Some("片段着色器"),
		source: wgpu::ShaderSource::Wgsl(FRAGMENT_SHADER.into()),
	});
	
	let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
		label: Some("管线布局"),
		bind_group_layouts: &[],
		push_constant_ranges: &[],
	});
	
	let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
		label: Some("渲染管线"),
		layout: Some(&pipeline_layout),
		vertex: wgpu::VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[],
		},
		fragment: Some(wgpu::FragmentState {
			module: &fragment_shader,
			entry_point: "fs_main",
			targets: &[Some(wgpu::ColorTargetState {
				format: config.format,
				blend: Some(wgpu::BlendState::REPLACE),
				write_mask: wgpu::ColorWrites::ALL,
			})],
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
	});
	
	// 保存 window 的另一个引用，以便在闭包中使用
	let window_for_closure = window.clone();
	
	event_loop.run(move |event, elwt| {
		match event {
			Event::WindowEvent { event, .. } => match event {
				WindowEvent::CloseRequested => elwt.exit(),
				WindowEvent::Resized(size) => {
					config.width = size.width.max(1);
					config.height = size.height.max(1);
					surface.configure(&device, &config);
					window_for_closure.request_redraw();
				}
				WindowEvent::RedrawRequested => {
					let frame = match surface.get_current_texture() {
						Ok(frame) => frame,
						Err(_) => return,
					};
					let view = frame.texture.create_view(&Default::default());
					let mut encoder = device.create_command_encoder(&Default::default());
					{
						let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
							label: Some("渲染通道"),
							color_attachments: &[Some(wgpu::RenderPassColorAttachment {
								view: &view,
								resolve_target: None,
								ops: wgpu::Operations {
									load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.2, b: 0.3, a: 1.0 }),
									store: wgpu::StoreOp::Store,
								},
							})],
							depth_stencil_attachment: None,
							occlusion_query_set: None,
							timestamp_writes: None,
						});
						render_pass.set_pipeline(&render_pipeline);
						render_pass.draw(0..3, 0..1);
					}
					queue.submit(Some(encoder.finish()));
					frame.present();
				}
				_ => {}
			},
			Event::AboutToWait => {
				window_for_closure.request_redraw();
			}
			_ => {}
		}
	}).unwrap();
}

async fn init_wgpu(window: &'static winit::window::Window) -> (wgpu::Surface<'static>, wgpu::Device, wgpu::Queue, wgpu::SurfaceConfiguration) {
	let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
	let surface = instance.create_surface(window).unwrap();
	let adapter = instance.request_adapter(&wgpu::RequestAdapterOptions {
		power_preference: wgpu::PowerPreference::HighPerformance,
		compatible_surface: Some(&surface),
		force_fallback_adapter: false,
	}).await.unwrap();
	
	let (device, queue) = adapter.request_device(
		&wgpu::DeviceDescriptor {
			label: None,
			required_features: wgpu::Features::empty(),
			required_limits: wgpu::Limits::default(),
		},
		None,
	).await.unwrap();
	
	let size = window.inner_size();
	let config = wgpu::SurfaceConfiguration {
		usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
		format: surface.get_capabilities(&adapter).formats[0],
		width: size.width.max(1),
		height: size.height.max(1),
		present_mode: wgpu::PresentMode::Fifo,
		desired_maximum_frame_latency: 2,
		alpha_mode: wgpu::CompositeAlphaMode::Auto,
		view_formats: vec![],
	};
	surface.configure(&device, &config);
	
	(surface, device, queue, config)
}

fn test() {
	let num = 0;
	println!(
		"{}",
		(|mut a: i32| -> i32 {
			a += 1;
			a
		})(num)
	); //1
	
	println!("{:?}", std::env::args());
	
	let a = 10;
	let ref b = a;
	let vec = vec![1, 2, 3];
	println!("{}", *b); //10
	
	let em = [1, 2, 3, 4, 5];
	
	let c = 100;
	println!("{}", c); //100
	
	let d = move || c;
	println!("{}", d()); //100
	
	let c = 200;
	println!("{}", c);
	//200
	let handle = thread::spawn(|| {
		for i in 0..5 {
			println!("spawned thread print {}", i);
			thread::sleep(Duration::from_millis(1));
		}
	});
	
	let (tx, rx) = mpsc::channel();
	
	thread::spawn(move || {
		let val = String::from("hi");
		tx.send(val).unwrap();
	});
	
	let received = rx.recv().unwrap();
	println!("Got: {}", received);
	
	apply_to_num(
		1,
		|_| {
			return 5;
		},
		|_| {
			return 7;
		},
	);
	
	macro_rules! test_print {
		($a:ty,$b:tt,$c:expr,$d:expr) => {
			let c:$a=$c;println!("{:?}", $c);
			println!("{}", $d $b c);
		};
	}
	
	test_print!(i32,-,1+1,7);
}

fn apply_to_num<F, G>(num: i32, f: F, f2: G)
where
	F: Fn(i32) -> i32,
	G: Fn(i32) -> i32,
{
	println!("{}", num + f(1) + f2(2));
	// let mut input = String::new();
	//
	// io::stdin().read_line(&mut input).expect("");
}
