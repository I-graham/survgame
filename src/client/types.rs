#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Uniform {
	pub aspect_ratio : cgmath::Matrix4<f32>,
}