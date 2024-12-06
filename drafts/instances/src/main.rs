use trivalibs::{
	hashmap,
	painter::{
		create_canvas_app,
		form::FormData,
		shade::ShadeProps,
		sketch::{Sketch, SketchProps},
		uniform::UniformBuffer,
		CanvasApp, Painter,
	},
	prelude::*,
	rendering::{
		camera::{CamProps, PerspectiveCamera},
		scene::SceneObject,
		transform::Transform,
	},
	wgpu::{self, include_spirv, VertexFormat},
	winit::event::{DeviceEvent, WindowEvent},
};

const VERTICES: &[Vec3] = &[vec3(0.0, 5.0, 0.0), vec3(-2.5, 0., 0.0), vec3(2.5, 0., 0.0)];

struct Triangle {
	transform: Transform,
	speed: f32,
}

struct App {
	cam: PerspectiveCamera,
	triangles: Vec<Triangle>,
}

const TRIANGLE_COUNT: usize = 100;
impl Default for App {
	fn default() -> Self {
		let mut triangles = Vec::with_capacity(TRIANGLE_COUNT);

		for _ in 0..TRIANGLE_COUNT {
			let mut t = Transform::from_translation(rand_vec3_range(-40.0, 40.0));
			t.look_at(rand_vec3_range(-40.0, 40.0), Vec3::Y);
			triangles.push(Triangle {
				transform: t,
				speed: rand_range(0.1, 1.0) * rand_sign(),
			});
		}

		Self {
			cam: PerspectiveCamera::create(CamProps {
				fov: Some(0.6),
				translation: Some(vec3(0.0, 0.0, 50.0)),
				..default()
			}),
			triangles,
		}
	}
}

struct RenderState {
	sketch: Sketch,
	model_mats: Vec<UniformBuffer<Mat4>>,
	vp_mat: UniformBuffer<Mat4>,
}

impl CanvasApp<RenderState, ()> for App {
	fn init(&mut self, p: &mut Painter) -> RenderState {
		let vert_u_layout = p.uniform_get_layout_buffered(wgpu::ShaderStages::VERTEX);
		let frag_u_layout = p.uniform_get_layout_buffered(wgpu::ShaderStages::FRAGMENT);

		let shade = p.shade_create(ShadeProps {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![VertexFormat::Float32x3],
			uniform_layout: &[&vert_u_layout, &vert_u_layout, &frag_u_layout],
		});

		let form = p.form_create(
			&FormData {
				vertex_buffer: VERTICES,
				index_buffer: None,
			},
			default(),
		);

		let uniforms = self
			.triangles
			.iter()
			.map(|t| {
				(
					p.uniform_create_mat4(&vert_u_layout, t.transform.model_mat()),
					p.uniform_create(&frag_u_layout, rand_vec3().extend(1.0)),
				)
			})
			.collect::<Vec<_>>();

		let cam = p.uniform_create_mat4(&vert_u_layout, self.cam.view_proj_mat());

		let sketch = p.sketch_create(
			form,
			shade,
			&SketchProps {
				uniforms: hashmap! {
					0 => cam.uniform,
				},
				instances: uniforms
					.iter()
					.map(|(model, color)| {
						hashmap! {
							1 => model.uniform,
							2 => color.uniform,
						}
					})
					.collect(),

				cull_mode: None,
				..default()
			},
		);

		let model_mats = uniforms.into_iter().map(|(model, _)| model).collect();

		RenderState {
			sketch,
			model_mats,
			vp_mat: cam,
		}
	}

	fn resize(&mut self, p: &mut Painter, rs: &mut RenderState) {
		let size = p.canvas_size();
		self
			.cam
			.set_aspect_ratio(size.width as f32 / size.height as f32);

		rs.vp_mat.update(p, self.cam.view_proj_mat());
	}

	fn update(&mut self, p: &mut Painter, rs: &mut RenderState, tpf: f32) {
		for (tri, model) in self.triangles.iter_mut().zip(rs.model_mats.iter_mut()) {
			tri.transform.rotate_y(tpf * tri.speed);

			model.update(p, tri.transform.model_mat());
		}
	}

	fn render(&self, p: &mut Painter, rs: &RenderState) -> Result<(), wgpu::SurfaceError> {
		p.request_redraw();
		p.draw(&rs.sketch)
	}

	fn window_event(&mut self, _e: WindowEvent, _p: &Painter) {}
	fn device_event(&mut self, _e: DeviceEvent, _p: &Painter) {}
	fn user_event(&mut self, _e: (), _p: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
