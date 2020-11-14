pub mod types;
pub mod utils;
mod resources;

pub struct Renderer2D<UniformType : Copy, InstanceType> {
	resources   : resources::RenderResources2D<UniformType, InstanceType>,
	render_data : types::RenderData,
}

impl<UniformType : Copy, InstanceType> Renderer2D<UniformType, InstanceType> {

	const PREALLOCATED_INSTANCES : usize = 16;

	pub fn new(win : &winit::window::Window, sample_count : u32, vs_path : Option<&std::path::Path>, fs_path : Option<&std::path::Path>) -> Self {

		let resources = futures::executor::block_on(
			resources::RenderResources2D::<UniformType, InstanceType>::new(win, sample_count, vs_path, fs_path)
		);

		let uniform_buffer = resources.device.create_buffer(
			&wgpu::BufferDescriptor {
				label : None,
				size : std::mem::size_of::<UniformType>() as wgpu::BufferAddress,
				usage : wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
				mapped_at_creation : false,
			}
		);

		let uniform_bg = resources.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label : None,
				layout : &resources.uniform_bgl,
				entries : &[
					wgpu::BindGroupEntry {
						binding : 0,
						resource : wgpu::BindingResource::Buffer(uniform_buffer.slice(..))
					}
				]
			}
		);

		let instance_buffer = resources.device.create_buffer(
			&wgpu::BufferDescriptor {
				label : None,
				size  : (Self::PREALLOCATED_INSTANCES * std::mem::size_of::<InstanceType>()) as wgpu::BufferAddress,
				usage : wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
				mapped_at_creation : false,
			}
		);

		let instance_bg = resources.device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				label : None,
				layout : &resources.instance_bgl,
				entries : &[
					wgpu::BindGroupEntry {
						binding : 0,
						resource : wgpu::BindingResource::Buffer(instance_buffer.slice(..))
					}
				]
			}
		);

		let def_image = image::ImageBuffer::from_pixel(1, 1, image::Rgba([255,255,255,255]));

		let texture = resources.create_texture_from_image(&def_image);

		let sampler = resources.device.create_sampler(&wgpu::SamplerDescriptor {
			label : Some("nearest sampler"),
			address_mode_u: wgpu::AddressMode::MirrorRepeat,
			address_mode_v: wgpu::AddressMode::MirrorRepeat,
			address_mode_w: wgpu::AddressMode::MirrorRepeat,
			mag_filter: wgpu::FilterMode::Nearest,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		let texture_bg = resources.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label : Some("default texture"),
			layout : &resources.texture_bgl,
			entries : &[
				wgpu::BindGroupEntry {
					binding : 0,
					resource : wgpu::BindingResource::TextureView(&texture.create_view(&wgpu::TextureViewDescriptor::default())),
				},
				wgpu::BindGroupEntry {
					binding : 1,
					resource : wgpu::BindingResource::Sampler(&sampler),
				}
			]
		});

		let spawner = {
			let pool = futures::executor::LocalPool::new();
			let spawner = pool.spawner();
			(pool, spawner)
		};

		let render_data = types::RenderData {
			uniform_buffer,
			uniform_bg,
			instance_buffer,
			instance_bg,
			instance_len : 0,
			instance_cap : Self::PREALLOCATED_INSTANCES,
			encoder : None,
			staging_belt : wgpu::util::StagingBelt::new(0x100),
			spawner,
			texture_bg,
			nearest_sampler : sampler,
		};

		Self {
			resources,
			render_data,
		}

	}

	pub fn set_uniform(&mut self, uniform : &UniformType) {
		let mut encoder = self.get_encoder();
		let belt = &mut self.render_data.staging_belt;
		let unif_data = &[*uniform];
		let unif_slice = utils::to_char_slice(unif_data);
		belt.write_buffer(&mut encoder, &self.render_data.uniform_buffer, 0 as wgpu::BufferAddress, std::num::NonZeroU64::new(unif_slice.len() as u64).unwrap(), &self.resources.device).copy_from_slice(unif_slice);
		self.set_encoder(encoder);
	}

	pub fn set_instances(&mut self, instances : &[InstanceType]) {
		let mut encoder = self.get_encoder();

		if self.render_data.instance_cap < instances.len() {
			self.render_data.instance_cap = instances.len();
			self.render_data.instance_buffer = self.resources.device.create_buffer(
				&wgpu::BufferDescriptor {
					label : None,
					size  : (instances.len() * std::mem::size_of::<InstanceType>()) as wgpu::BufferAddress,
					usage : wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::STORAGE | wgpu::BufferUsage::COPY_DST,
					mapped_at_creation : false,
				}
			);

			self.render_data.instance_bg = self.resources.device.create_bind_group(
				&wgpu::BindGroupDescriptor {
					label : None,
					layout : &self.resources.instance_bgl,
					entries : &[
						wgpu::BindGroupEntry {
							binding : 0,
							resource : wgpu::BindingResource::Buffer(self.render_data.instance_buffer.slice(..))
						}
					]
				}
			);
		}

		self.render_data.instance_len = instances.len();

		let belt = &mut self.render_data.staging_belt;
		let inst_slice = utils::to_char_slice(instances);
		belt.write_buffer(&mut encoder, &self.render_data.instance_buffer, 0 as wgpu::BufferAddress, std::num::NonZeroU64::new(inst_slice.len() as u64).unwrap(), &self.resources.device).copy_from_slice(inst_slice);
		self.set_encoder(encoder);
	}

	pub fn set_texture(&mut self, texture : &wgpu::Texture) {
		self.render_data.texture_bg = self.resources.device.create_bind_group(&wgpu::BindGroupDescriptor {
			label : None,
			layout : &self.resources.texture_bgl,
			entries : &[
				wgpu::BindGroupEntry {
					binding : 0,
					resource : wgpu::BindingResource::TextureView(&texture.create_view(&wgpu::TextureViewDescriptor::default())),
				},
				wgpu::BindGroupEntry {
					binding : 1,
					resource : wgpu::BindingResource::Sampler(&self.render_data.nearest_sampler),
				}
			],
		});
	}

	pub fn submit(&mut self) {
		use futures::task::LocalSpawnExt;
		let encoder = self.get_encoder();
		self.render_data.staging_belt.finish();
		self.resources.queue.submit(Some(encoder.finish()));
		let recall_fut = self.render_data.staging_belt.recall();
		self.render_data.spawner.1.spawn_local(recall_fut).unwrap();
	}

	pub fn resize(&mut self, dims : winit::dpi::PhysicalSize<u32>) {
		self.resources.resize(dims)
	}

	pub fn draw_test(&mut self, uniform : &UniformType, instances : &[InstanceType]) {
		self.set_uniform(uniform);
		self.set_instances(instances);

		let frame = {
			let wgpu::SwapChainFrame { mut output, mut suboptimal } = self.resources.swap.get_current_frame().unwrap();
			while suboptimal {
				let frame = self.resources.swap.get_current_frame().unwrap();
				output = frame.output;
				suboptimal = frame.suboptimal;
			}
			output
		};

		let mut encoder = self.get_encoder();
		let attachment_desc = if self.resources.sample_count == 1 {
			wgpu::RenderPassColorAttachmentDescriptor {
				attachment : &frame.view,
				resolve_target : None,
				ops : wgpu::Operations {
					load  : wgpu::LoadOp::Clear(wgpu::Color{r : 0.0, g : 0.0, b : 0.0, a : 0.0}),
					store : true,
				}
			}
		} else {
			wgpu::RenderPassColorAttachmentDescriptor {
				attachment : &self.resources.msaa_texture.1,
				resolve_target : Some(&frame.view),
				ops : wgpu::Operations {
					load  : wgpu::LoadOp::Clear(wgpu::Color{r : 0.0, g : 0.0, b : 0.0, a : 0.0}),
					store : true,
				}
			}
		};

		let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
			color_attachments : &[attachment_desc],
			depth_stencil_attachment: Some( wgpu::RenderPassDepthStencilAttachmentDescriptor {
				attachment : &self.resources.depth_buffer.1,
				depth_ops : Some(wgpu::Operations {
					load  : wgpu::LoadOp::Clear(1.0),
					store : true,
				}),
				stencil_ops : None,
			}),
		});

		render_pass.set_pipeline(&self.resources.pipeline);
		render_pass.set_bind_group(0, &self.render_data.uniform_bg, &[]);
		render_pass.set_bind_group(1, &self.render_data.instance_bg, &[]);
		render_pass.set_bind_group(2, &self.render_data.texture_bg, &[]);
		render_pass.draw(0..5, 0..self.render_data.instance_len as u32);

		drop(render_pass);
		self.set_encoder(encoder);

		self.submit();
	}

	pub fn create_texture_from_image(&self, image : &image::RgbaImage) -> wgpu::Texture {
		self.resources.create_texture_from_image(image)
	}

	fn get_encoder(&mut self) -> wgpu::CommandEncoder {
		match self.render_data.encoder.take() {
			Some(encoder) => encoder,
			None => {
				self.resources.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
					label : None,
				})
			}
		}
	}

	fn set_encoder(&mut self, encoder : wgpu::CommandEncoder) {
		self.render_data.encoder = Some(encoder);
	}

}