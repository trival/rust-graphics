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
				speed: rand_range(0.1, 1.0),
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
	fn init(&mut self, painter: &mut Painter) -> RenderState {
		let vert_u_layout = painter.uniform_get_layout_buffered(wgpu::ShaderStages::VERTEX);
		let frag_u_layout = painter.uniform_get_layout_buffered(wgpu::ShaderStages::FRAGMENT);

		let shade = painter.shade_create(ShadeProps {
			vertex_shader: include_spirv!("../shader/vertex.spv"),
			fragment_shader: include_spirv!("../shader/fragment.spv"),
			vertex_format: vec![VertexFormat::Float32x3],
			uniform_layout: &[&vert_u_layout, &vert_u_layout, &frag_u_layout],
		});

		let form = painter.form_create(
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
					painter.uniform_create_mat4(&vert_u_layout, t.transform.model_mat()),
					painter.uniform_create_buffered(&frag_u_layout, rand_vec3().extend(1.0)),
				)
			})
			.collect::<Vec<_>>();

		let cam = painter.uniform_create_mat4(&vert_u_layout, self.cam.view_proj_mat());

		let sketch = painter.sketch_create(
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

	fn resize(&mut self, painter: &mut Painter, render_state: &mut RenderState) {
		let size = painter.canvas_size();
		self
			.cam
			.set_aspect_ratio(size.width as f32 / size.height as f32);
		painter.uniform_update_mat4(&render_state.vp_mat, self.cam.view_proj_mat());
	}

	fn update(&mut self, painter: &mut Painter, render_state: &mut RenderState, tpf: f32) {
		for (tri, model) in self
			.triangles
			.iter_mut()
			.zip(render_state.model_mats.iter_mut())
		{
			tri.transform.rotate_y(tpf * tri.speed);
			painter.uniform_update_mat4(model, tri.transform.model_mat());
		}
	}

	fn render(
		&self,
		painter: &mut Painter,
		render_state: &RenderState,
	) -> Result<(), wgpu::SurfaceError> {
		painter.request_redraw();
		painter.draw(&render_state.sketch)
	}

	fn window_event(&mut self, _event: WindowEvent, _painter: &Painter) {}
	fn device_event(&mut self, _event: DeviceEvent, _painter: &Painter) {}
	fn user_event(&mut self, _event: (), _painter: &Painter) {}
}

pub fn main() {
	create_canvas_app(App::default()).start();
}
