/*use paste::paste;

macro_rules! uniform_field {
	($field:ident : $type:ty) => { paste! {
		$field : $type,
		[<__ $field>] : [i8; 16 - std::mem::size_of::<$type>()],
	} };
}*/

#[repr(C)]
pub struct Uniform {
	pub number : f32,
}

#[repr(C)]
pub struct Instance2D {
	pub texture_id : i32,
	pub color_tint : [f32; 4],
	pub scale      : f32,
	pub translate  : [f32; 2],
}
