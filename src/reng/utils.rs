#[allow(dead_code)]

pub fn to_char_slice_mut<T>(array : &mut [T]) -> &mut [u8] {

	let size = std::mem::size_of::<T>();

	let data_ptr = array.as_ptr() as *mut u8;

	unsafe { std::slice::from_raw_parts_mut(data_ptr, array.len() * size)}

}

pub fn to_char_slice<T>(array : &[T]) -> &[u8] {

	let size = std::mem::size_of::<T>();

	let data_ptr = array.as_ptr() as *const u8;

	unsafe { std::slice::from_raw_parts(data_ptr, array.len() * size)}

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
		sample_count,
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

pub fn create_spritesheet(mut images : Vec<image::RgbaImage>) -> (image::RgbaImage, Vec<((u32,u32),(u32,u32))>) {

	use image::GenericImage;
	use image::GenericImageView;
	let mut dyn_image = image::DynamicImage::new_rgba8(1, 1);

	let mut corners = vec![(0u32, 0u32)];
	let mut placed_images : Vec<((u32,u32),(u32,u32))> = vec![];
	let mut final_coords = vec![];

	images.sort_by_key(|e| -((e.height() * e.width()) as i32) );

	for image in &images {
		let mut best_extension : Option<(usize, (u32, u32), bool)> = None;
		'corner_loop: for corner in corners.iter().enumerate() {

			for rotated in &[false, true] {
				let x_coord = corner.1.0 + if *rotated { image.height() } else { image.width() };
				let y_coord = corner.1.1 + if *rotated { image.width() } else { image.height() };

				let size_dims = dyn_image.dimensions();
				let x_ext = x_coord as i32 - size_dims.0 as i32;
				let y_ext = y_coord as i32 - size_dims.1 as i32;

				for placed_image in &placed_images {
					let placed_ul = placed_image.0;
					let placed_lr = placed_image.1;

					let corner_ul = corner.1;
					let corner_lr = (x_coord, y_coord);

					if corner_ul.0 < placed_lr.0 && corner_lr.0 > placed_ul.0 &&
					   corner_ul.1 < placed_lr.1 && corner_lr.1 > placed_ul.1 {
						continue 'corner_loop;
					}

				}

				let extension = (
					if x_ext > 0 { x_ext as u32 } else { 0 },
					if y_ext > 0 { y_ext as u32 } else { 0 },
				);

				if let Some(best_ext) = best_extension {
					let dims = dyn_image.dimensions();
					let compute_ext_size = |ext : (u32, u32)| {
						ext.0 * dims.1 + ext.1 * dims.0 + ext.0 * ext.1
					};
					if compute_ext_size(best_ext.1) > compute_ext_size(extension) {
						best_extension = Some((corner.0, extension, *rotated));
					}
				} else {
					best_extension = Some((corner.0, extension, *rotated));
				}
			}
		}

		let position = best_extension.unwrap();
		let corner = corners.remove(position.0);

		if position.1 != (0u32,0u32) {
			let mut grown = image::DynamicImage::new_rgba8(
				dyn_image.dimensions().0 + position.1.0,
				dyn_image.dimensions().1 + position.1.1
			);

			grown.copy_from(&dyn_image, 0, 0).unwrap();
			dyn_image = grown;
		}


		let dims;
		if position.2 {
			let flipped = image::imageops::rotate90(image);
			dyn_image.copy_from(&flipped, corner.0, corner.1).unwrap();
			dims = (
				corner.0 + flipped.dimensions().0,
				corner.1 + flipped.dimensions().1
			);
			final_coords.push((
				(dims.0, corner.1),
				(corner.0, dims.1)
			));
		} else {
			dyn_image.copy_from(image, corner.0, corner.1).unwrap();
			dims = (
				corner.0 + image.dimensions().0,
				corner.1 + image.dimensions().1
			);
			final_coords.push((
				corner,
				dims
			));
		}

		placed_images.push((
			corner,
			dims
		));
		corners.push((
			dims.0,
			corner.1,
		));
		corners.push((
			corner.0,
			dims.1,
		));

	}


	(dyn_image.into_rgba(), final_coords)

}