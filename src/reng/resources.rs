use super::utils;

pub struct RenderResources2D<UniformType, InstanceType> {
	pub win_size     : winit::dpi::PhysicalSize<u32>,
	pub sample_count : u32,
	pub surface      : wgpu::Surface,
	pub adapter      : wgpu::Adapter,
	pub device       : wgpu::Device,
	pub queue        : wgpu::Queue,
	pub sc_desc      : wgpu::SwapChainDescriptor,
	pub swap         : wgpu::SwapChain,
	pub pipeline     : wgpu::RenderPipeline,
	pub uniform_bgl  : wgpu::BindGroupLayout,
	pub instance_bgl : wgpu::BindGroupLayout,
	pub texture_bgl  : wgpu::BindGroupLayout,
	pub depth_buffer : (wgpu::Texture, wgpu::TextureView),
	pub msaa_texture : (wgpu::Texture, wgpu::TextureView),
	_unif_marker     : std::marker::PhantomData<UniformType>,
	_inst_marker     : std::marker::PhantomData<InstanceType>,

}

impl<UniformType, InstanceType> RenderResources2D<UniformType, InstanceType> {
	const DEPTH_FORMAT : wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

	const VERTEX_DESC : wgpu::VertexBufferDescriptor<'static> = wgpu::VertexBufferDescriptor {
		stride : 8,
		step_mode : wgpu::InputStepMode::Vertex,
		attributes : &wgpu::vertex_attr_array![0 => Float2],
	};

	pub async fn new(win : &winit::window::Window, sample_count : u32, vert_shader_path : Option<&std::path::Path>, frag_shader_path : Option<&std::path::Path>) -> Self{

		let win_size = win.inner_size();

		let instance = wgpu::Instance::new(wgpu::BackendBit::PRIMARY);

		let surface = unsafe { instance.create_surface(win) };

		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				power_preference : wgpu::PowerPreference::LowPower,
				compatible_surface : Some(&surface),
			}
		).await.unwrap();

		let adapter_features = adapter.features();

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

		let uniform_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: Some("uniform_layout"),
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding : 0,
					count : None,
					visibility : wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
					ty: wgpu::BindingType::UniformBuffer {
						dynamic : false,
						min_binding_size : Some(std::num::NonZeroU64::new(std::mem::size_of::<UniformType>() as u64).unwrap()),
					},
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

		let texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			entries : &[
				wgpu::BindGroupLayoutEntry {
					binding : 0,
					count : None,
					visibility : wgpu::ShaderStage::FRAGMENT,
					ty : wgpu::BindingType::SampledTexture {
						multisampled : false,
						dimension : wgpu::TextureViewDimension::D2,
						component_type : wgpu::TextureComponentType::Uint,
					}
				},
				wgpu::BindGroupLayoutEntry {
					binding : 1,
					count : None,
					visibility : wgpu::ShaderStage::FRAGMENT,
					ty : wgpu::BindingType::Sampler {
						comparison : false,
					}
				}
			],
			label : None,
		});

		let depth_buffer = utils::create_depth_texture(&device, &sc_desc, sample_count, Self::DEPTH_FORMAT);

		let pipeline = {

			let vert_shader;
			let frag_shader;

			if let Some(path) = vert_shader_path {
				let full = path.canonicalize().unwrap_or_else(|err| panic!("unable to canonicalize path due to the following error : '{}'", err));
				let source = std::fs::read(full.clone()).unwrap_or_else(|err| panic!("Unable to load fragment source from '{}' due to the following error : '{}'.", full.clone().display(), err));
				let module = wgpu::util::make_spirv(source.as_slice());
				vert_shader = device.create_shader_module(module);
			} else {
				vert_shader = device.create_shader_module(wgpu::include_spirv!("./shaders/default.vert.spv"));
			};

			if let Some(path) = frag_shader_path {
				let full = path.canonicalize().unwrap_or_else(|err| panic!("unable to canonicalize path due to the following error : '{}'", err));
				let source = std::fs::read(full.clone()).unwrap_or_else(|err| panic!("Unable to load fragment source from '{}' due to the following error : '{}'.", full.clone().display(), err));
				let module = wgpu::util::make_spirv(source.as_slice());
				frag_shader = device.create_shader_module(module);
			} else {
				frag_shader = device.create_shader_module(wgpu::include_spirv!("./shaders/default.frag.spv"));
			};


			let layout = &device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label : None,
				bind_group_layouts: &[&uniform_bgl, &instance_bgl, &texture_bgl],
				push_constant_ranges: &[],
			});

			device.create_render_pipeline(
				&wgpu::RenderPipelineDescriptor {
					label : None,
					layout : Some(layout),
					vertex_stage: wgpu::ProgrammableStageDescriptor {
						module : &vert_shader,
						entry_point: "main",
					},
					fragment_stage: Some (wgpu::ProgrammableStageDescriptor {
						module: &frag_shader,
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
					primitive_topology : wgpu::PrimitiveTopology::TriangleStrip,
					depth_stencil_state : Some( wgpu::DepthStencilStateDescriptor {
						format : Self::DEPTH_FORMAT,
						depth_write_enabled : true,
						depth_compare : wgpu::CompareFunction::LessEqual,
						stencil : wgpu::StencilStateDescriptor {
							front : wgpu::StencilStateFaceDescriptor::IGNORE,
							back : wgpu::StencilStateFaceDescriptor::IGNORE,
							read_mask : 0,
							write_mask : 0,
						}
					}),
					vertex_state : wgpu::VertexStateDescriptor {
						index_format: wgpu::IndexFormat::Uint32,
						vertex_buffers: &[Self::VERTEX_DESC],
					},
					sample_count,
					sample_mask: !0,
					alpha_to_coverage_enabled: false,

				}
			)

		};

		let msaa_texture = utils::create_multisampled_framebuffer(&device, &sc_desc, sample_count);
		let _unif_marker = std::marker::PhantomData::<UniformType>;
		let _inst_marker = std::marker::PhantomData::<InstanceType>;

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
			uniform_bgl,
			depth_buffer,
			instance_bgl,
			texture_bgl,
			msaa_texture,
			_unif_marker,
			_inst_marker,
		}

	}

	pub fn create_texture_from_image(&self, image : &image::RgbaImage) -> wgpu::Texture {
		let dimensions = image.dimensions();

		let size = wgpu::Extent3d {
			width : dimensions.0,
			height : dimensions.1,
			depth : 1,
		};

		let text = self.device.create_texture(&wgpu::TextureDescriptor {
			label : Some("reng_texture"),
			usage : wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
			mip_level_count : 1,
			sample_count : 1,
			format : wgpu::TextureFormat::Rgba8UnormSrgb,
			dimension : wgpu::TextureDimension::D2,
			size,
		});

		self.queue.write_texture(
			wgpu::TextureCopyView {
				texture : &text,
				mip_level : 0,
				origin : wgpu::Origin3d::ZERO,
			},
			image.as_raw().as_slice(),
			wgpu::TextureDataLayout {
				offset : 0,
				bytes_per_row : 4 * dimensions.0,
				rows_per_image : dimensions.1,
			},
			size
		);

		text

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

