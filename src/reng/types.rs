#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct GLint(pub i32);

#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct GLfloat(pub f32);

#[repr(C, align(8))]
#[derive(Clone, Copy, Debug)]
pub struct GLvec2(pub f32, pub f32);

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct GLvec3(pub f32, pub f32, pub f32);

#[repr(C, align(16))]
#[derive(Clone, Copy, Debug)]
pub struct GLvec4(pub f32, pub f32, pub f32, pub f32);

use futures::executor::{LocalPool, LocalSpawner};
pub struct RenderData {
	pub uniform_buffer  : wgpu::Buffer,
	pub uniform_bg      : wgpu::BindGroup,
	pub instance_buffer : wgpu::Buffer,
	pub instance_bg     : wgpu::BindGroup,
	pub instance_len    : usize,
	pub instance_cap    : usize,
	pub encoder         : Option<wgpu::CommandEncoder>,
	pub staging_belt    : wgpu::util::StagingBelt,
	pub spawner         : (LocalPool, LocalSpawner),
	pub texture_bg      : wgpu::BindGroup,
	pub nearest_sampler : wgpu::Sampler,
}