use trivalibs::{
	prelude::*,
	rendering::{
		mesh_geometry::{
			utils::{vert_pos_uv, Vert3dUv},
			MeshBufferType, MeshGeometry,
		},
		shapes::quad::Quad3D,
		BufferedGeometry,
	},
};

pub fn create_plane(width: f32, height: f32, normal: Vec3, center: Vec3) -> BufferedGeometry {
	let plane: Quad3D<Vert3dUv> =
		Quad3D::from_dimensions_center_f(width, height, normal, center, vert_pos_uv).into();

	let mut geom = MeshGeometry::new();
	geom.add_face4(plane.to_ccw_verts());

	geom.to_buffered_geometry_by_type(MeshBufferType::FaceNormals)
}
