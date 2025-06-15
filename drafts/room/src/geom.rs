use trivalibs::{
	prelude::*,
	rendering::{
		mesh_geometry::{
			face_normal,
			utils::{vert_pos_uv, Vert3dUv},
			FaceDataProps, MeshBufferType, MeshGeometry,
		},
		shapes::{cuboid::Cuboid, quad::Quad3D},
		BufferedGeometry,
	},
};

pub fn create_plane(width: f32, height: f32, normal: Vec3, center: Vec3) -> BufferedGeometry {
	let plane: Quad3D<Vert3dUv> =
		Quad3D::from_dimensions_center_f(width, height, normal, center, vert_pos_uv).into();

	let mut geom = MeshGeometry::new();
	geom.add_face4_data(plane.to_ccw_verts(), face_normal(plane.normal));

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}

pub struct GridProps {
	pub grid_width: f32,
	pub grid_height: f32,
	pub count: usize,
	pub strip_width: f32,
	pub strip_height: f32,
	pub center: Vec3,
}

pub struct GridData {
	pub form: BufferedGeometry,
	pub texture_size: (f32, f32),
}

pub fn create_grid_rows_form(props: GridProps) -> GridData {
	let face_data = |normal: Vec3, section: usize| FaceDataProps {
		normal: Some(normal),
		section: Some(section),
		data: None,
	};

	let mut geom = MeshGeometry::new();

	let step = props.grid_height / props.count as f32;

	let h_half = props.grid_height / 2.0 - step;

	let count = props.count - 1;
	let v_full = count as f32 * (props.strip_height * 2.0 + props.strip_width);
	let v_height = props.strip_height / v_full;
	let v_bottom = props.strip_width / v_full;

	let start_pos = props.center - vec3(0., 0., h_half);
	let mut v_start = 0.;

	for i in 0..count {
		let bbox = Cuboid::box_at(
			start_pos + i as f32 * vec3(0., 0., step),
			props.grid_width,
			props.strip_height,
			props.strip_width,
		);

		let front =
			bbox.front_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.y * v_height + v_start)));
		geom.add_face4_data(front.to_ccw_verts(), face_data(front.normal, 0));
		v_start += v_height;

		let back =
			bbox.back_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.x, uvw.y * v_height + v_start)));
		geom.add_face4_data(back.to_ccw_verts(), face_data(back.normal, 1));
		v_start += v_height;

		let bottom =
			bbox.bottom_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.x, uvw.z * v_bottom + v_start)));
		geom.add_face4_data(bottom.to_ccw_verts(), face_data(bottom.normal, 2));
		v_start += v_bottom;
	}

	GridData {
		form: geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals),
		texture_size: (props.grid_width, v_full),
	}
}

pub fn create_grid_columns_form(props: GridProps) -> GridData {
	let face_data = |normal: Vec3, section: usize| FaceDataProps {
		normal: Some(normal),
		section: Some(section),
		data: None,
	};

	let mut geom = MeshGeometry::new();

	let step = props.grid_width / props.count as f32;

	let w_half = props.grid_width / 2.0 - step;

	let count = props.count - 1;
	let v_full = count as f32 * (props.strip_height * 2.0 + props.strip_width);
	let v_height = props.strip_height / v_full;
	let v_bottom = props.strip_width / v_full;

	let start_pos = props.center - vec3(w_half, 0., 0.);
	let mut v_start = 0.;

	for i in 0..count {
		let bbox = Cuboid::box_at(
			start_pos + i as f32 * vec3(step, 0., 0.),
			props.strip_width,
			props.strip_height,
			props.grid_height,
		);

		let left =
			bbox.left_face_f(|pos, uvw| vert_pos_uv(pos, vec2(1.0 - uvw.z, uvw.y * v_height + v_start)));
		geom.add_face4_data(left.to_ccw_verts(), face_data(left.normal, 0));
		v_start += v_height;

		let right =
			bbox.right_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.z, uvw.y * v_height + v_start)));
		geom.add_face4_data(right.to_ccw_verts(), face_data(right.normal, 1));
		v_start += v_height;

		let bottom =
			bbox.bottom_face_f(|pos, uvw| vert_pos_uv(pos, vec2(uvw.z, uvw.x * v_bottom + v_start)));
		geom.add_face4_data(bottom.to_ccw_verts(), face_data(bottom.normal, 2));
		v_start += v_bottom;
	}

	GridData {
		form: geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals),
		texture_size: (props.grid_height, v_full),
	}
}
