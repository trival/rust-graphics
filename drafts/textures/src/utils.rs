use noise::{NoiseFn, Simplex};
use trivalibs::{
	painter::{texture::Texture, wgpu, Painter},
	prelude::*,
	rendering::texture::f64_to_u8,
};

pub fn rand_rgba_f32(width: u32, height: u32) -> Vec<f32> {
	let mut rgba = vec![0.0; (width * height * 4) as usize];
	for i in 0..rgba.len() {
		rgba[i] = rand_range(0.0, 1.0);
	}
	rgba
}

pub fn rand_rgba_u8(width: u32, height: u32) -> Vec<u8> {
	let mut rgba = vec![0; (width * height * 4) as usize];
	for i in 0..rgba.len() {
		rgba[i] = rand_range(0., 255.) as u8;
	}
	rgba
}

pub fn tiled_noise(u: f64, v: f64, scale: f64, seed: u32) -> f64 {
	let simplex = Simplex::new(seed);

	// Map coordinates to circle for seamless wrapping
	let angle_x = u * std::f64::consts::TAU;
	let angle_y = v * std::f64::consts::TAU;

	let nx = angle_x.cos() * scale;
	let ny = angle_y.cos() * scale;
	let nz = angle_x.sin() * scale;
	let nw = angle_y.sin() * scale;

	// Get 4D noise value
	let value = simplex.get([nx, ny, nz, nw]);

	// Map from [-1, 1] to [0, 1]
	value * 0.5 + 0.5
}

pub fn tiled_noise_rgba_u8(width: u32, height: u32, initial_scale: f64) -> Vec<u8> {
	let size = (width * height) as usize;
	let mut rgba = vec![0; size * 4];
	let seed_r = rand_f32().floor() as u32;
	let seed_g = rand_f32().floor() as u32;
	let seed_b = rand_f32().floor() as u32;
	let seed_a = rand_f32().floor() as u32;

	for i in 0..size {
		let u = (i % width as usize) as f64 / width as f64;
		let v = (i / width as usize) as f64 / height as f64;
		let i = i * 4;
		rgba[i] = f64_to_u8(tiled_noise(u, v, initial_scale, seed_r));
		rgba[i + 1] = f64_to_u8(tiled_noise(u, v, initial_scale * 2.025, seed_g));
		rgba[i + 2] = f64_to_u8(tiled_noise(u, v, initial_scale * 4.05, seed_b));
		rgba[i + 3] = f64_to_u8(tiled_noise(u, v, initial_scale * 8.1, seed_a));
	}

	rgba
}

pub fn tiled_noise_rgba_f32(width: u32, height: u32, initial_scale: f64) -> Vec<f32> {
	let size = (width * height) as usize;
	let mut rgba = vec![0.0; size * 4];
	let seed_r = rand_f32().floor() as u32;
	let seed_g = rand_f32().floor() as u32;
	let seed_b = rand_f32().floor() as u32;
	let seed_a = rand_f32().floor() as u32;

	for i in 0..size {
		let u = (i % width as usize) as f64 / width as f64;
		let v = (i / width as usize) as f64 / height as f64;
		let i = i * 4;
		rgba[i] = tiled_noise(u, v, initial_scale, seed_r) as f32;
		rgba[i + 1] = tiled_noise(u, v, initial_scale * 2.0, seed_g) as f32;
		rgba[i + 2] = tiled_noise(u, v, initial_scale * 4.0, seed_b) as f32;
		rgba[i + 3] = tiled_noise(u, v, initial_scale * 8.0, seed_a) as f32;
	}

	rgba
}

pub fn textures_u8(
	p: &mut Painter,
	width: u32,
	height: u32,
	noise_scale: f64,
) -> (Texture, Texture) {
	let texture_random = p.texture_2d(width, height).create();
	texture_random.fill_2d(p, &rand_rgba_u8(width, height));

	let texture_simplex = p.texture_2d(width, height).create();
	texture_simplex.fill_2d(p, &tiled_noise_rgba_u8(width, height, noise_scale));

	(texture_random, texture_simplex)
}

pub fn textures_f32(
	p: &mut Painter,
	width: u32,
	height: u32,
	noise_scale: f64,
) -> (Texture, Texture) {
	let texture_random_f32 = p
		.texture_2d(width, height)
		.with_format(wgpu::TextureFormat::Rgba32Float)
		.create();

	texture_random_f32.fill_2d(p, bytemuck::cast_slice(&rand_rgba_f32(width, height)));

	let texture_simplex_f32 = p
		.texture_2d(width, height)
		.with_format(wgpu::TextureFormat::Rgba32Float)
		.create();

	texture_simplex_f32.fill_2d(
		p,
		bytemuck::cast_slice(&tiled_noise_rgba_f32(width, height, noise_scale)),
	);

	(texture_random_f32, texture_simplex_f32)
}
