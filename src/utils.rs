#[allow(dead_code)]

use std::net;

pub const SERVER_PORT : u16 = 8778;

pub fn get_public_ip() -> net::IpAddr {
	let ip_str = reqwest::blocking::get("https://www.sfml-dev.org/ip-provider.php")
		.expect("Unable to make internet request, are you sure you're connected to the internet?")
		.text()
		.unwrap();

	use std::str::FromStr;
	net::IpAddr::from_str(&ip_str).unwrap()
}

pub fn to_char_slice<T>(array : &[T]) -> &mut [u8] {

	let size = std::mem::size_of::<T>();

	let data_ptr = array.as_ptr() as *mut u8;

	unsafe { std::slice::from_raw_parts_mut(data_ptr, array.len() * size)}

}

pub fn create_depth_texture(device : &wgpu::Device, sc_desc : &wgpu::SwapChainDescriptor, sample_count: u32, depth_format : wgpu::TextureFormat) -> (wgpu::Texture, wgpu::TextureView) {

	let size = wgpu::Extent3d {
		width  : sc_desc.width,
		height : sc_desc.height,
		depth  : 1,
	};

	let desc = wgpu::TextureDescriptor {
		label : Some("Depth"),
		size,
		mip_level_count : 1,
		sample_count,
		dimension : wgpu::TextureDimension::D2,
		format : depth_format,
		usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT
		| wgpu::TextureUsage::SAMPLED
		| wgpu::TextureUsage::COPY_SRC,
	};

	let texture = device.create_texture(&desc);
	let view = texture.create_view(
		&wgpu::TextureViewDescriptor {
			..Default::default()
		}
	);

	(texture, view)

}

pub fn create_multisampled_framebuffer(device: &wgpu::Device, sc_desc: &wgpu::SwapChainDescriptor, sample_count: u32) -> (wgpu::Texture, wgpu::TextureView) {
	let multisampled_texture_extent = wgpu::Extent3d {
		width: sc_desc.width,
		height: sc_desc.height,
		depth: 1,
	};
	let multisampled_frame_descriptor = &wgpu::TextureDescriptor {
		size: multisampled_texture_extent,
		mip_level_count: 1,
		sample_count: sample_count,
		dimension: wgpu::TextureDimension::D2,
		format: sc_desc.format,
		usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
		label: None,
	};

	let texture = device.create_texture(multisampled_frame_descriptor);

	let view = texture.create_view(
		&wgpu::TextureViewDescriptor {
			..Default::default()
		}
	);

	(texture, view)
}