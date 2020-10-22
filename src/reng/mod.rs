mod resources;
mod types;

pub struct Renderer2D {
	resources : resources::RenderResources2D,

}

impl Renderer2D {

	const VERTEX_DESC : wgpu::VertexBufferDescriptor<'static> = wgpu::VertexBufferDescriptor {
		stride : 4,
		step_mode : wgpu::InputStepMode::Vertex,
		attributes : &wgpu::vertex_attr_array![0 => Float]
	};

	pub fn new(win : &winit::window::Window, sample_count : u32) -> Self {

		let uniform = types::Uniform {
			number : 0.0
		};

		let resources = futures::executor::block_on(
			resources::RenderResources2D::new(win, Self::VERTEX_DESC, uniform, sample_count)
		);

		Self {
			resources,
		}

	}

	pub fn draw_test(&mut self) {
		let frame = {
			let wgpu::SwapChainFrame { mut output, mut suboptimal } = self.resources.swap.get_current_frame().unwrap();
			while suboptimal {
				let frame = self.resources.swap.get_current_frame().unwrap();
				output = frame.output;
				suboptimal = frame.suboptimal;
			}
			output
		};

		let mut encoder = self.resources.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label : None,
		});

		let attachment_desc = if self.resources.sample_count == 1 {
			wgpu::RenderPassColorAttachmentDescriptor {
				attachment : &frame.view,
				resolve_target : None,
				ops : wgpu::Operations {
					load  : wgpu::LoadOp::Clear(wgpu::Color::BLUE),
					store : true,
				}
			}
		} else {
			wgpu::RenderPassColorAttachmentDescriptor {
				attachment : &self.resources.msaa_texture.1,
				resolve_target : Some(&frame.view),
				ops : wgpu::Operations {
					load  : wgpu::LoadOp::Clear(wgpu::Color::BLUE),
					store : true,
				}
			}
		};

		let render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

		drop(render_pass);

		self.resources.queue.submit(Some(encoder.finish()));

	}

}