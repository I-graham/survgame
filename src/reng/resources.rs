use crate::utils;

pub struct RenderResources2D {
	pub win_size     : winit::dpi::PhysicalSize<u32>,
	pub sample_count : u32,

	pub surface      : wgpu::Surface,
	pub adapter      : wgpu::Adapter,
	pub device       : wgpu::Device,
	pub queue        : wgpu::Queue,
	pub sc_desc      : wgpu::SwapChainDescriptor,
	pub swap         : wgpu::SwapChain,
	pub pipeline     : wgpu::RenderPipeline,
	pub uniform_bg   : wgpu::BindGroup,
	pub uniform_bgl  : wgpu::BindGroupLayout,
	pub uniform_buf  : wgpu::Buffer,
	pub instance_bgl : wgpu::BindGroupLayout,

	pub depth_buffer : (wgpu::Texture, wgpu::TextureView),
	pub msaa_texture : (wgpu::Texture, wgpu::TextureView),

}

impl RenderResources2D {
	const DEPTH_FORMAT : wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	pub async fn new<T>(win : &winit::window::Window, vertex_desc : wgpu::VertexBufferDescriptor<'_>, vertex_uniform : T, sample_count : u32) -> Self{

		let win_size = win.inner_size();

		let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

		let surface = unsafe { instance.create_surface(win) };

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference : wgpu::PowerPreference::Default,
				compatible_surface : Some(&surface),
			}
		).await.unwrap();

		let adapter_features = adapter.features();

		let mut a = adapter_features;

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features : adapter_features,
				limits : Default::default(),
				shader_validation : false,
			},
			None
		).await.unwrap();

		let sc_desc = wgpu::SwapChainDescriptor {

			usage        : wgpu::TextureUsage::OUTPUT_ATTACHMENT,
			format       : wgpu::TextureFormat::Bgra8UnormSrgb,
			width        : win_size.width,
			height       : win_size.height,
			present_mode : wgpu::PresentMode::Mailbox,

		};

		let swap = device.create_swap_chain(&surface, &sc_desc);

		use wgpu::util::{BufferInitDescriptor, DeviceExt};
		let uniform_buf = device.create_buffer_init(
			&BufferInitDescriptor {
				label : None,
				contents : utils::to_char_slice(&[vertex_uniform]),
				usage : wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
			}
		);

		let uniforms_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("uniform_layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding : 0,
					visibility : wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
					ty: wgpu::BindingType::UniformBuffer {
						dynamic : false,
						min_binding_size : Some(core::num::NonZeroU64::new(std::mem::size_of::<T>() as u64).unwrap()),
					},
					count : None,
				},
			],
		});

		let uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
			layout: &uniforms_layout,
			label: Some("uniform_bg"),
			entries : &[
				wgpu::BindGroupEntry {
					binding : 0,
					resource : wgpu::BindingResource::Buffer(uniform_buf.slice(..))
				},
			],
		});

		let instance_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries : &[
				wgpu::BindGroupLayoutEntry {
					binding : 0,
					count : None,
					visibility : wgpu::ShaderStage::VERTEX,
					ty : wgpu::BindingType::StorageBuffer {
						dynamic : false,
						readonly : true,
						min_binding_size : None,
					},
				},
			],
			label : Some("instances")
		});

		let depth_buffer = utils::create_depth_texture(&device, &sc_desc, sample_count, Self::DEPTH_FORMAT);

		let pipeline = {

			let vertshader = device.create_shader_module(wgpu::include_spirv!("../../assets/shaders/default.vert.spv"));
			let fragshader = device.create_shader_module(wgpu::include_spirv!("../../assets/shaders/default.frag.spv"));

			let layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label : None,
				bind_group_layouts: &[&uniforms_layout, &instance_bgl],
				push_constant_ranges: &[],
			});

			device.create_render_pipeline(
				&wgpu::RenderPipelineDescriptor {
					label : None,
					layout : Some(layout),
					vertex_stage: wgpu::ProgrammableStageDescriptor {
						module : &vertshader,
						entry_point: "main",
					},
					fragment_stage: Some (wgpu::ProgrammableStageDescriptor {
						module: &fragshader,
						entry_point: "main",
					}),
					rasterization_state: Some(wgpu::RasterizationStateDescriptor{
						front_face : wgpu::FrontFace::default(),
						cull_mode : wgpu::CullMode::Back,
						clamp_depth : true,
						depth_bias : 0,
						depth_bias_slope_scale : 0.0,
						depth_bias_clamp : 1.0,
					}),
					color_states : &[
						wgpu::ColorStateDescriptor {
							format : sc_desc.format,
							color_blend: wgpu::BlendDescriptor {
								src_factor: wgpu::BlendFactor::One,
								dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
								operation: wgpu::BlendOperation::Add,
							},
							alpha_blend: wgpu::BlendDescriptor {
								src_factor: wgpu::BlendFactor::One,
								dst_factor: wgpu::BlendFactor::One,
								operation: wgpu::BlendOperation::Add,
							},
							write_mask : wgpu::ColorWrite::ALL,
						}
					],
					primitive_topology : wgpu::PrimitiveTopology::TriangleList,
					depth_stencil_state : Some( wgpu::DepthStencilStateDescriptor {
						format : Self::DEPTH_FORMAT,
						depth_write_enabled : true,
						depth_compare : wgpu::CompareFunction::Less,
						stencil : wgpu::StencilStateDescriptor {
							front : wgpu::StencilStateFaceDescriptor::IGNORE,
							back : wgpu::StencilStateFaceDescriptor::IGNORE,
							read_mask : 0,
							write_mask : 0,
						}
					}),
					vertex_state : wgpu::VertexStateDescriptor {
						index_format: wgpu::IndexFormat::Uint32,
						vertex_buffers: &[vertex_desc],
					},
					sample_count,
					sample_mask: !0,
					alpha_to_coverage_enabled: false,

				}
			)

		};

		let msaa_texture = utils::create_multisampled_framebuffer(&device, &sc_desc, sample_count);

		Self {
			win_size,
			sample_count,
			surface,
			adapter,
			device,
			queue,
			sc_desc,
			swap,
			pipeline,
			uniform_bg,
			uniform_bgl : uniforms_layout,
			uniform_buf,
			depth_buffer,
			instance_bgl,
			msaa_texture
		}

	}

	pub fn resize(&mut self, size : winit::dpi::PhysicalSize<u32>) {
		self.win_size = size;
		self.sc_desc.width = size.width;
		self.sc_desc.height = size.height;
		self.swap = self.device.create_swap_chain(&self.surface, &self.sc_desc);
		self.depth_buffer = utils::create_depth_texture(&self.device, &self.sc_desc, self.sample_count, Self::DEPTH_FORMAT);
		self.msaa_texture = utils::create_multisampled_framebuffer(&self.device, &self.sc_desc, self.sample_count);
	}

}

